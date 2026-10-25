//! The plugin backend session: one process the host starts from the plugin's
//! manifest and talks to over stdio with the LSP base protocol. The SDK owns
//! everything Lithe manages uniformly -- the lifecycle, protocol negotiation,
//! cancellation, error codes, and the `lithe/*` namespace -- and leaves the
//! plugin free to decide what its handlers do.
//!
//! Contract: `contract/v1/README.md` ("Plugin backend", "Plugin functions",
//! and "Host services").

use crate::error::{
    cancelled_response, internal_response, response_error, sdk_codes, Cancelled, FunctionError,
    SdkError,
};
use crate::framing::{read_frame, write_frame, Frame};
use crate::protocol::{
    bounded_text, error_codes, is_credential_key, is_function_id, methods, FUNCTIONS_VERSIONS,
    HOST_SERVICES_VERSION, MAX_CREDENTIAL_SECRET_BYTES, MAX_FUNCTIONS, MAX_MESSAGE_BYTES,
    MAX_RESULT_BYTES,
};
use serde_json::{json, Map, Value};
use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::io::{self, BufReader, Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

type Handler = Arc<dyn Fn(Value, &Context) -> Result<Value, FunctionError> + Send + Sync>;
type InitializeHook = Box<dyn FnMut(&mut Initialize) -> Result<(), FunctionError> + Send>;
type ShutdownHook = Box<dyn FnMut() + Send>;
type Answer = Result<Value, Value>;
type Outcome = Result<Value, SdkError>;

/// `Idle` → `Listening` → `Initializing` → `Ready` → `ShuttingDown` → `Exited`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Listening,
    Initializing,
    Ready,
    ShuttingDown,
    Exited,
}

struct State {
    phase: Phase,
    functions: BTreeMap<String, Handler>,
    /// The negotiated plugin-function protocol version.
    functions_version: Option<u64>,
    /// Host requests being handled, by JSON-RPC ID.
    incoming: HashMap<String, Running>,
    /// Requests this backend sent to the host, by JSON-RPC ID.
    outgoing: HashMap<String, (&'static str, mpsc::Sender<Outcome>)>,
}

struct Running {
    function_id: String,
    cancellation: Cancellation,
}

/// Everything handler threads share with the session. Locks are taken in the
/// order `registrations`, `state`, `output`, never the other way round.
struct Shared {
    name: String,
    state: Mutex<State>,
    output: Mutex<Option<Box<dyn Write + Send>>>,
    /// Serializes register/unregister so the host sees them in call order.
    registrations: Mutex<()>,
    next_request: AtomicU64,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The JSON-RPC ID as a map key; numbers and strings never collide.
fn request_key(id: &Value) -> String {
    id.to_string()
}

fn rpc_error(code: i64, message: impl Into<String>) -> Value {
    json!({ "code": code, "message": message.into() })
}

/// A plugin backend. Declare functions and hooks, then call
/// [`listen`](Self::listen).
///
/// ```no_run
/// use lithe_plugin_sdk::{Backend, PluginError};
/// use serde_json::json;
///
/// let mut backend = Backend::new("example-backend").version("1.0.0");
/// backend
///     .function("snapshot.refresh", |params, context| {
///         context.progress("Refreshing", Some(10));
///         let name = params["name"].as_str().ok_or_else(|| {
///             PluginError::new("example.invalidName", "\"name\" must be a string.")
///         })?;
///         Ok(json!({ "refreshed": name }))
///     })
///     .expect("a valid function ID");
/// std::process::exit(backend.listen());
/// ```
pub struct Backend {
    shared: Arc<Shared>,
    version: Option<String>,
    initialize_hooks: Vec<InitializeHook>,
    shutdown_hooks: Vec<ShutdownHook>,
}

impl Backend {
    /// `name` is reported as `serverInfo.name` and prefixes log lines.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            shared: Arc::new(Shared {
                name: name.into(),
                state: Mutex::new(State {
                    phase: Phase::Idle,
                    functions: BTreeMap::new(),
                    functions_version: None,
                    incoming: HashMap::new(),
                    outgoing: HashMap::new(),
                }),
                output: Mutex::new(None),
                registrations: Mutex::new(()),
                next_request: AtomicU64::new(1),
            }),
            version: None,
            initialize_hooks: Vec::new(),
            shutdown_hooks: Vec::new(),
        }
    }

    /// Reported as `serverInfo.version`.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Declares a function offered from the start, in the `initialize`
    /// answer. To add one later, use [`BackendHandle::register_function`].
    ///
    /// Each call runs on a thread of its own, so a handler may block; it
    /// should stop early when its [`Context::cancellation`] fires.
    pub fn function<F>(&mut self, id: &str, handler: F) -> Result<&mut Self, SdkError>
    where
        F: Fn(Value, &Context) -> Result<Value, FunctionError> + Send + Sync + 'static,
    {
        let mut state = lock(&self.shared.state);
        check_new_function(id, &state.functions)?;
        state.functions.insert(id.to_string(), Arc::new(handler));
        drop(state);
        Ok(self)
    }

    /// Runs before the `initialize` answer. A hook may read the host's
    /// params and add functions; an error refuses `initialize`, mapped like
    /// a handler's.
    pub fn on_initialize<F>(&mut self, hook: F) -> &mut Self
    where
        F: FnMut(&mut Initialize) -> Result<(), FunctionError> + Send + 'static,
    {
        self.initialize_hooks.push(Box::new(hook));
        self
    }

    /// Runs on `shutdown`, after running calls were cancelled.
    pub fn on_shutdown<F>(&mut self, hook: F) -> &mut Self
    where
        F: FnMut() + Send + 'static,
    {
        self.shutdown_hooks.push(Box::new(hook));
        self
    }

    /// A handle for other threads: runtime registration, host services, and
    /// logging.
    pub fn handle(&self) -> BackendHandle {
        BackendHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Serves the host on stdin and stdout until it sends `exit` or closes
    /// stdin, and returns the process exit code. Stdout carries only
    /// protocol frames: on Unix the SDK moves the protocol to a duplicate of
    /// the stdout descriptor and points stdout at stderr, so a stray
    /// `println!`, in the plugin or a dependency, lands in the host's log
    /// instead of corrupting the stream.
    pub fn listen(self) -> i32 {
        let output = crate::stdio::protocol_output();
        self.serve(io::stdin().lock(), output)
    }

    /// Serves the host on the given streams; [`listen`](Self::listen) uses
    /// stdin and stdout.
    pub fn serve(mut self, input: impl Read, output: impl Write + Send + 'static) -> i32 {
        lock(&self.shared.state).phase = Phase::Listening;
        *lock(&self.shared.output) = Some(Box::new(output));
        let mut reader = BufReader::new(input);
        loop {
            match read_frame(&mut reader) {
                Ok(None) => {
                    let code = i32::from(self.shared.phase() != Phase::ShuttingDown);
                    return self.shared.finish(code);
                }
                Err(error) => {
                    self.shared.log(format_args!("Ending the session: {error}"));
                    return self.shared.finish(1);
                }
                Ok(Some(Frame::Malformed)) => self.shared.respond(
                    &Value::Null,
                    Err(rpc_error(
                        error_codes::PARSE_ERROR,
                        "The message is not valid JSON.",
                    )),
                ),
                Ok(Some(Frame::Message(message))) => {
                    if let Some(code) = self.dispatch(message) {
                        return self.shared.finish(code);
                    }
                }
            }
        }
    }

    /// Handles one message; returns the exit code once the host sent `exit`.
    fn dispatch(&mut self, message: Value) -> Option<i32> {
        let Value::Object(message) = message else {
            // Valid JSON that is not a message object, such as `null` or a batch.
            self.shared.respond(
                &Value::Null,
                Err(rpc_error(
                    error_codes::INVALID_REQUEST,
                    "The message is not a JSON-RPC object.",
                )),
            );
            return None;
        };
        let id = message.get("id").filter(|id| !id.is_null()).cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        match (message.get("method").and_then(Value::as_str), id) {
            (None, Some(id)) => {
                self.shared.settle_outgoing(&id, &message);
                None
            }
            (None, None) => None,
            (Some(method), Some(id)) => {
                self.handle_request(id, method, params);
                None
            }
            (Some(method), None) => self.handle_notification(method, &params),
        }
    }

    fn handle_request(&mut self, id: Value, method: &str, params: Value) {
        match method {
            methods::INITIALIZE => self.initialize(id, params),
            methods::SHUTDOWN => self.shutdown(id),
            _ => match self.shared.phase() {
                Phase::Ready if method == methods::FUNCTION_INVOKE => {
                    Shared::invoke(&self.shared, id, &params)
                }
                Phase::Ready => self.shared.respond(
                    &id,
                    Err(rpc_error(
                        error_codes::METHOD_NOT_FOUND,
                        format!("Unsupported method: {method}"),
                    )),
                ),
                Phase::Idle | Phase::Listening | Phase::Initializing => self.shared.respond(
                    &id,
                    Err(rpc_error(
                        error_codes::SERVER_NOT_INITIALIZED,
                        "The backend is not initialized.",
                    )),
                ),
                Phase::ShuttingDown | Phase::Exited => self.shared.respond(
                    &id,
                    Err(rpc_error(
                        error_codes::INVALID_REQUEST,
                        "The backend is shutting down.",
                    )),
                ),
            },
        }
    }

    fn handle_notification(&mut self, method: &str, params: &Value) -> Option<i32> {
        match method {
            methods::EXIT => Some(i32::from(self.shared.phase() != Phase::ShuttingDown)),
            methods::CANCEL_REQUEST => {
                let key = request_key(&params["id"]);
                let running = lock(&self.shared.state)
                    .incoming
                    .get(&key)
                    .map(|running| running.cancellation.clone());
                if let Some(cancellation) = running {
                    cancellation.cancel();
                }
                None
            }
            // `initialized` and anything else a plugin backend is not sent.
            _ => None,
        }
    }

    fn initialize(&mut self, id: Value, params: Value) {
        {
            let mut state = lock(&self.shared.state);
            if state.phase != Phase::Listening {
                drop(state);
                return self.shared.respond(
                    &id,
                    Err(rpc_error(
                        error_codes::INVALID_REQUEST,
                        "initialize was already received.",
                    )),
                );
            }
            state.phase = Phase::Initializing;
        }
        let offered = params
            .pointer("/capabilities/experimental/litheFunctions/versions")
            .and_then(Value::as_array);
        let functions_version = FUNCTIONS_VERSIONS.into_iter().find(|version| {
            offered
                .is_some_and(|offered| offered.iter().any(|entry| entry.as_u64() == Some(*version)))
        });
        let mut initialize = Initialize {
            params,
            functions_version,
            existing: lock(&self.shared.state).functions.keys().cloned().collect(),
            added: BTreeMap::new(),
        };
        for hook in &mut self.initialize_hooks {
            let outcome = catch_unwind(AssertUnwindSafe(|| hook(&mut initialize)));
            let error = match outcome {
                Ok(Ok(())) => continue,
                Ok(Err(error)) => {
                    self.shared.log(format_args!("initialize failed: {error}"));
                    response_error(&error)
                }
                Err(panic) => {
                    self.shared
                        .log(format_args!("initialize failed: {}", panic_message(&panic)));
                    internal_response()
                }
            };
            lock(&self.shared.state).phase = Phase::Listening;
            return self.shared.respond(&id, Err(error));
        }

        // The state stays locked until the answer is written, so a function
        // registered from another thread cannot overtake it.
        let mut state = lock(&self.shared.state);
        state.functions.extend(initialize.added);
        state.functions_version = functions_version;
        let mut experimental = Map::new();
        match functions_version {
            Some(version) => {
                let functions: Vec<_> = state
                    .functions
                    .keys()
                    .map(|id| json!({ "id": id }))
                    .collect();
                experimental.insert(
                    "litheFunctions".into(),
                    json!({ "version": version, "functions": functions }),
                );
            }
            None if !state.functions.is_empty() => self.shared.log(
                "The host did not offer litheFunctions version 1 or 2; functions stay unavailable.",
            ),
            None => {}
        }
        let mut server_info = json!({ "name": self.shared.name });
        if let Some(version) = &self.version {
            server_info["version"] = json!(version);
        }
        state.phase = Phase::Ready;
        self.shared.respond(
            &id,
            Ok(json!({ "capabilities": { "experimental": experimental }, "serverInfo": server_info })),
        );
    }

    fn shutdown(&mut self, id: Value) {
        let running = {
            let mut state = lock(&self.shared.state);
            if !matches!(
                state.phase,
                Phase::Listening | Phase::Initializing | Phase::Ready
            ) {
                drop(state);
                return self.shared.respond(
                    &id,
                    Err(rpc_error(
                        error_codes::INVALID_REQUEST,
                        "shutdown was already received.",
                    )),
                );
            }
            state.phase = Phase::ShuttingDown;
            state.running_cancellations()
        };
        running.iter().for_each(Cancellation::cancel);
        for hook in &mut self.shutdown_hooks {
            if let Err(panic) = catch_unwind(AssertUnwindSafe(&mut *hook)) {
                self.shared.log(format_args!(
                    "A shutdown hook failed: {}",
                    panic_message(&panic)
                ));
            }
        }
        self.shared.respond(&id, Ok(Value::Null));
    }
}

impl State {
    fn running_cancellations(&self) -> Vec<Cancellation> {
        self.incoming
            .values()
            .map(|running| running.cancellation.clone())
            .collect()
    }

    fn require_functions(&self) -> Result<(), SdkError> {
        if self.phase == Phase::Exited {
            Err(SdkError::new(
                sdk_codes::SESSION_CLOSED,
                "The session ended.",
            ))
        } else if self.phase == Phase::Ready && self.functions_version.is_some() {
            Ok(())
        } else {
            Err(SdkError::new(
                "sdk.functions.unavailable",
                "The host has not negotiated plugin functions.",
            ))
        }
    }
}

impl Shared {
    fn phase(&self) -> Phase {
        lock(&self.state).phase
    }

    fn functions_version(&self) -> Option<u64> {
        lock(&self.state).functions_version
    }

    fn log(&self, message: impl Display) {
        let _ = writeln!(io::stderr(), "[{}] {message}", self.name);
    }

    fn finish(&self, code: i32) -> i32 {
        let running = {
            let mut state = lock(&self.state);
            state.phase = Phase::Exited;
            // Dropping the senders ends every host request still waiting.
            state.outgoing.clear();
            state.running_cancellations()
        };
        running.iter().for_each(Cancellation::cancel);
        *lock(&self.output) = None;
        code
    }

    fn write(&self, message: &Value) {
        let mut output = lock(&self.output);
        let Some(writer) = output.as_mut() else {
            return;
        };
        if let Err(error) = write_frame(writer.as_mut(), message) {
            // Every later write would fail the same way.
            *output = None;
            drop(output);
            self.log(format_args!("The host stream failed: {error}"));
        }
    }

    fn respond(&self, id: &Value, answer: Answer) {
        self.write(&match answer {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
        });
    }

    fn invoke(shared: &Arc<Self>, id: Value, params: &Value) {
        let function_id = params["functionID"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let operation_id = params["operationID"].clone();
        let call_params = match params.get("params") {
            None | Some(Value::Null) => json!({}),
            Some(params) => params.clone(),
        };
        let key = request_key(&id);
        let cancellation = Cancellation::new();
        let handler = {
            let mut state = lock(&shared.state);
            let handler = state
                .functions_version
                .and_then(|_| state.functions.get(&function_id).cloned());
            if handler.is_some() {
                state.incoming.insert(
                    key.clone(),
                    Running {
                        function_id: function_id.clone(),
                        cancellation: cancellation.clone(),
                    },
                );
            }
            handler
        };
        let Some(handler) = handler else {
            return shared.respond(
                &id,
                Err(rpc_error(
                    error_codes::INVALID_PARAMS,
                    format!("Function {function_id} is not registered."),
                )),
            );
        };
        let worker = Arc::clone(shared);
        let thread_key = key.clone();
        let thread_id = id.clone();
        let spawned = thread::Builder::new()
            .name(format!("lithe-function-{function_id}"))
            .spawn(move || {
                let context = Context {
                    shared: Arc::clone(&worker),
                    function_id,
                    operation_id,
                    cancellation,
                    reporting: AtomicBool::new(true),
                };
                let outcome = catch_unwind(AssertUnwindSafe(|| handler(call_params, &context)));
                context.reporting.store(false, Ordering::SeqCst);
                let answer = worker.function_answer(&context, outcome);
                lock(&worker.state).incoming.remove(&thread_key);
                worker.respond(&thread_id, answer);
            });
        if let Err(error) = spawned {
            lock(&shared.state).incoming.remove(&key);
            shared.log(format_args!("The call could not start: {error}"));
            shared.respond(&id, Err(internal_response()));
        }
    }

    /// The answer to one call. A cancelled call answers as cancelled whatever
    /// the handler returned, and only failures the host never sees in full
    /// are logged with their details.
    fn function_answer(
        &self,
        context: &Context,
        outcome: Result<Result<Value, FunctionError>, Box<dyn Any + Send>>,
    ) -> Answer {
        let cancelled = context.cancellation.is_cancelled();
        let function = &context.function_id;
        match outcome {
            _ if cancelled => Err(cancelled_response()),
            Ok(Ok(value)) => {
                let size = serde_json::to_vec(&value).map_or(usize::MAX, |bytes| bytes.len());
                if size > MAX_RESULT_BYTES {
                    self.log(format_args!(
                        "Function {function} returned {size} bytes; results are limited to {MAX_RESULT_BYTES}."
                    ));
                    Err(internal_response())
                } else {
                    Ok(value)
                }
            }
            Ok(Err(error)) => {
                match &error {
                    FunctionError::Plugin(error) => {
                        self.log(format_args!("Function {function} failed: {error}"))
                    }
                    FunctionError::Internal(details) => self.log(format_args!(
                        "Function {function} failed unexpectedly: {details}"
                    )),
                    FunctionError::Cancelled => {}
                }
                Err(response_error(&error))
            }
            Err(panic) => {
                self.log(format_args!(
                    "Function {function} panicked: {}",
                    panic_message(&panic)
                ));
                Err(internal_response())
            }
        }
    }

    /// Sends a request to the host and waits for its answer, the end of the
    /// session, or the cancellation of the call that sent it.
    fn request(
        &self,
        method: &'static str,
        params: Value,
        cancellation: Option<&Cancellation>,
    ) -> Outcome {
        let id = format!("sdk-{}", self.next_request.fetch_add(1, Ordering::Relaxed));
        let (sender, receiver) = mpsc::channel();
        {
            let mut state = lock(&self.state);
            match state.phase {
                Phase::Ready => {}
                Phase::Exited => {
                    return Err(SdkError::new(
                        sdk_codes::SESSION_CLOSED,
                        "The session ended.",
                    ))
                }
                _ => {
                    return Err(SdkError::new(
                        "sdk.session.notReady",
                        "The session is not ready for requests.",
                    ))
                }
            }
            state.outgoing.insert(id.clone(), (method, sender.clone()));
        }
        if let Some(cancellation) = cancellation {
            cancellation.on_cancel(move || {
                let _ = sender.send(Err(SdkError::new(
                    sdk_codes::CALL_CANCELLED,
                    "The call was cancelled.",
                )));
            });
        } else {
            drop(sender);
        }
        self.write(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let outcome = receiver.recv().unwrap_or_else(|_| {
            Err(SdkError::new(
                sdk_codes::SESSION_CLOSED,
                "The session ended.",
            ))
        });
        lock(&self.state).outgoing.remove(&id);
        outcome
    }

    fn settle_outgoing(&self, id: &Value, message: &Map<String, Value>) {
        let Some(id) = id.as_str() else {
            return;
        };
        let Some((method, sender)) = lock(&self.state).outgoing.remove(id) else {
            return;
        };
        let outcome = match message.get("error").filter(|error| !error.is_null()) {
            Some(error) => Err(SdkError::from_response(method, error)),
            None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
        };
        let _ = sender.send(outcome);
    }
}

fn check_new_function(id: &str, functions: &BTreeMap<String, Handler>) -> Result<(), SdkError> {
    if !is_function_id(id) {
        return Err(SdkError::new(
            "sdk.function.invalidID",
            format!("Function ID {id:?} must be dot-separated lowerCamelCase segments of at most 128 bytes."),
        ));
    }
    if functions.contains_key(id) {
        return Err(SdkError::new(
            "sdk.function.duplicate",
            format!("Function {id} is already declared."),
        ));
    }
    if functions.len() >= MAX_FUNCTIONS {
        return Err(SdkError::new(
            "sdk.function.limit",
            format!("A backend serves at most {MAX_FUNCTIONS} functions."),
        ));
    }
    Ok(())
}

fn panic_message(panic: &Box<dyn Any + Send>) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|message| message.to_string())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic without a message".into())
}

/// What a hook sees of `initialize`, before the backend answers it.
pub struct Initialize {
    params: Value,
    functions_version: Option<u64>,
    existing: Vec<String>,
    added: BTreeMap<String, Handler>,
}

impl Initialize {
    /// The host's `initialize` params.
    pub fn params(&self) -> &Value {
        &self.params
    }

    /// The plugin-function protocol version the backend will answer, or
    /// `None` when the host offered none this SDK speaks.
    pub fn functions_version(&self) -> Option<u64> {
        self.functions_version
    }

    /// The `rootUri` as a path. For a plugin backend this is the plugin's
    /// `${stateDir}`, also the working directory.
    pub fn root_path(&self) -> Option<PathBuf> {
        self.params["rootUri"].as_str().and_then(file_uri_path)
    }

    /// Adds a function to the initial list, for example one that depends on
    /// state found at startup.
    pub fn function<F>(&mut self, id: &str, handler: F) -> Result<(), SdkError>
    where
        F: Fn(Value, &Context) -> Result<Value, FunctionError> + Send + Sync + 'static,
    {
        if self.existing.iter().any(|existing| existing == id) {
            return Err(SdkError::new(
                "sdk.function.duplicate",
                format!("Function {id} is already declared."),
            ));
        }
        if self.existing.len() + self.added.len() >= MAX_FUNCTIONS {
            return Err(SdkError::new(
                "sdk.function.limit",
                format!("A backend serves at most {MAX_FUNCTIONS} functions."),
            ));
        }
        check_new_function(id, &self.added)?;
        self.added.insert(id.to_string(), Arc::new(handler));
        Ok(())
    }
}

/// The path of a `file:` URI, percent-decoded.
fn file_uri_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    if !path.starts_with('/') {
        return None;
    }
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let mut path = String::from_utf8(decoded).ok()?;
    // `/C:/Users` on Windows.
    if cfg!(windows) && path.as_bytes().get(2) == Some(&b':') {
        path.remove(0);
    }
    while path.len() > 1 && path.ends_with('/') {
        path.pop();
    }
    Some(PathBuf::from(path))
}

/// One running function call.
pub struct Context {
    shared: Arc<Shared>,
    function_id: String,
    operation_id: Value,
    cancellation: Cancellation,
    reporting: AtomicBool,
}

impl Context {
    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    /// Fires when the host cancels the call, withdraws its function, or
    /// shuts the backend down. Core has already ended the call by then, so a
    /// handler should stop its work and return promptly.
    pub fn cancellation(&self) -> &Cancellation {
        &self.cancellation
    }

    /// `Err(Cancelled)` once the call is cancelled, for use with `?`.
    pub fn check_cancelled(&self) -> Result<(), Cancelled> {
        self.cancellation.check()
    }

    /// Reports progress of this call; the host shows the latest report.
    /// `message` is cut to 1024 bytes and `percentage` to 100. Reports after
    /// the call ended are dropped.
    pub fn progress(&self, message: &str, percentage: Option<u8>) {
        if !self.reporting.load(Ordering::SeqCst) {
            return;
        }
        let mut report = json!({
            "operationID": self.operation_id,
            "message": bounded_text(message, MAX_MESSAGE_BYTES),
        });
        if let Some(percentage) = percentage {
            report["percentage"] = json!(percentage.min(100));
        }
        self.shared.write(
            &json!({ "jsonrpc": "2.0", "method": methods::FUNCTION_PROGRESS, "params": report }),
        );
    }

    /// The plugin's credentials. A request still waiting for the host when
    /// the call is cancelled fails with `sdk.call.cancelled`, which `?`
    /// turns into a cancelled answer.
    pub fn credentials(&self) -> Credentials {
        Credentials {
            shared: Arc::clone(&self.shared),
            cancellation: Some(self.cancellation.clone()),
        }
    }

    pub fn backend(&self) -> BackendHandle {
        BackendHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Writes a diagnostic line to stderr, which the host keeps in its log.
    pub fn log(&self, message: impl Display) {
        self.shared.log(message);
    }
}

/// A clonable handle to a backend for any thread.
#[derive(Clone)]
pub struct BackendHandle {
    shared: Arc<Shared>,
}

impl BackendHandle {
    /// The negotiated plugin-function protocol version; `None` before
    /// `initialize` or when the host offered none this SDK speaks.
    pub fn functions_version(&self) -> Option<u64> {
        self.shared.functions_version()
    }

    /// IDs of the functions the backend currently serves, sorted.
    pub fn function_ids(&self) -> Vec<String> {
        lock(&self.shared.state).functions.keys().cloned().collect()
    }

    /// Adds a function after `initialize` and waits until the host accepted
    /// it. The handler is installed first, because the host may invoke it as
    /// soon as its registry changed.
    pub fn register_function<F>(&self, id: &str, handler: F) -> Result<(), SdkError>
    where
        F: Fn(Value, &Context) -> Result<Value, FunctionError> + Send + Sync + 'static,
    {
        let _order = lock(&self.shared.registrations);
        {
            let mut state = lock(&self.shared.state);
            state.require_functions()?;
            check_new_function(id, &state.functions)?;
            state.functions.insert(id.to_string(), Arc::new(handler));
        }
        let registered = self.shared.request(
            methods::FUNCTION_REGISTER,
            json!({ "functions": [{ "id": id }] }),
            None,
        );
        if registered.is_err() {
            lock(&self.shared.state).functions.remove(id);
        }
        registered.map(drop)
    }

    /// Withdraws a function and waits until the host accepted it. The host
    /// ends the function's running calls as revoked, and the SDK cancels
    /// them. A handler must not withdraw its own function: that revokes the
    /// call it runs in.
    pub fn unregister_function(&self, id: &str) -> Result<(), SdkError> {
        let _order = lock(&self.shared.registrations);
        {
            let state = lock(&self.shared.state);
            state.require_functions()?;
            if !state.functions.contains_key(id) {
                return Err(SdkError::new(
                    "sdk.function.unknown",
                    format!("Function {id} is not registered."),
                ));
            }
        }
        self.shared
            .request(methods::FUNCTION_UNREGISTER, json!({ "ids": [id] }), None)?;
        let running: Vec<_> = {
            let mut state = lock(&self.shared.state);
            state.functions.remove(id);
            state
                .incoming
                .values()
                .filter(|running| running.function_id == id)
                .map(|running| running.cancellation.clone())
                .collect()
        };
        running.iter().for_each(Cancellation::cancel);
        Ok(())
    }

    /// The plugin's credentials, for work outside a function call.
    pub fn credentials(&self) -> Credentials {
        Credentials {
            shared: Arc::clone(&self.shared),
            cancellation: None,
        }
    }

    /// Writes a diagnostic line to stderr, which the host keeps in its log.
    pub fn log(&self, message: impl Display) {
        self.shared.log(message);
    }
}

/// The host's `credentials` service (plugin-function protocol 2): secrets
/// private to this plugin, kept in the platform's secret store, never in
/// `${stateDir}`. It needs the `credentials.store` permission; without it
/// every request fails with `plugin.host.permissionDenied`.
pub struct Credentials {
    shared: Arc<Shared>,
    cancellation: Option<Cancellation>,
}

impl Credentials {
    /// The secret stored under `key`, or `None`.
    pub fn get(&self, key: &str) -> Result<Option<String>, SdkError> {
        let result = self.request(methods::CREDENTIALS_GET, key, json!({ "key": key }))?;
        match &result["secret"] {
            Value::String(secret) => Ok(Some(secret.clone())),
            Value::Null => Ok(None),
            _ => Err(SdkError::new(
                "sdk.host.invalidResult",
                "The host answered without a secret.",
            )),
        }
    }

    /// Stores `secret` (at most 16 KiB) under `key`, replacing any value.
    pub fn set(&self, key: &str, secret: &str) -> Result<(), SdkError> {
        if secret.len() > MAX_CREDENTIAL_SECRET_BYTES {
            return Err(SdkError::new(
                "sdk.credentials.invalidSecret",
                format!("A secret is at most {MAX_CREDENTIAL_SECRET_BYTES} bytes."),
            ));
        }
        self.request(
            methods::CREDENTIALS_SET,
            key,
            json!({ "key": key, "secret": secret }),
        )
        .map(drop)
    }

    /// Removes the secret under `key`; succeeds when nothing was stored.
    pub fn delete(&self, key: &str) -> Result<(), SdkError> {
        self.request(methods::CREDENTIALS_DELETE, key, json!({ "key": key }))
            .map(drop)
    }

    fn request(&self, method: &'static str, key: &str, params: Value) -> Outcome {
        if !is_credential_key(key) {
            return Err(SdkError::new(
                "sdk.credentials.invalidKey",
                "A credential key is 1-128 ASCII letters, digits, \".\", \"_\", \":\", or \"-\".",
            ));
        }
        if self
            .shared
            .functions_version()
            .is_none_or(|version| version < HOST_SERVICES_VERSION)
        {
            return Err(SdkError::new(
                sdk_codes::NOT_NEGOTIATED,
                "The host did not negotiate plugin-function protocol 2, which offers credentials.",
            ));
        }
        self.shared
            .request(method, params, self.cancellation.as_ref())
    }
}

/// The cancellation of one call. Clones observe the same call.
#[derive(Clone)]
pub struct Cancellation(Arc<CancellationInner>);

struct CancellationInner {
    state: Mutex<CancellationState>,
    changed: Condvar,
}

struct CancellationState {
    cancelled: bool,
    callbacks: Vec<Box<dyn FnOnce() + Send>>,
}

impl Cancellation {
    fn new() -> Self {
        Self(Arc::new(CancellationInner {
            state: Mutex::new(CancellationState {
                cancelled: false,
                callbacks: Vec::new(),
            }),
            changed: Condvar::new(),
        }))
    }

    pub fn is_cancelled(&self) -> bool {
        lock(&self.0.state).cancelled
    }

    /// `Err(Cancelled)` once cancelled, for use with `?`.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }

    /// Blocks for at most `timeout`, and returns whether the call was
    /// cancelled. Use it instead of `thread::sleep` between retries.
    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        let state = lock(&self.0.state);
        let (state, _) = self
            .0
            .changed
            .wait_timeout_while(state, timeout, |state| !state.cancelled)
            .unwrap_or_else(PoisonError::into_inner);
        state.cancelled
    }

    /// Runs `callback` once, when the call is cancelled, or right away when
    /// it already is: for example to cancel a query a driver is running.
    /// The callback runs on the thread that cancels, so it must not block.
    pub fn on_cancel(&self, callback: impl FnOnce() + Send + 'static) {
        let mut state = lock(&self.0.state);
        if state.cancelled {
            drop(state);
            callback();
        } else {
            state.callbacks.push(Box::new(callback));
        }
    }

    pub(crate) fn cancel(&self) {
        let callbacks = {
            let mut state = lock(&self.0.state);
            if state.cancelled {
                return;
            }
            state.cancelled = true;
            std::mem::take(&mut state.callbacks)
        };
        self.0.changed.notify_all();
        for callback in callbacks {
            callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris_become_paths() {
        assert_eq!(
            file_uri_path("file:///var/state/"),
            Some(PathBuf::from("/var/state"))
        );
        assert_eq!(
            file_uri_path("file://localhost/a%20b/%C3%A9"),
            Some(PathBuf::from("/a b/é"))
        );
        assert_eq!(file_uri_path("file:///"), Some(PathBuf::from("/")));
        assert_eq!(file_uri_path("https://example.com/"), None);
        assert_eq!(file_uri_path("file:///bad%zz"), None);
    }

    #[test]
    fn cancellation_wakes_waiters_and_runs_callbacks_once() {
        let cancellation = Cancellation::new();
        assert!(!cancellation.wait_timeout(Duration::ZERO));
        let (sender, receiver) = mpsc::channel();
        let early = sender.clone();
        cancellation.on_cancel(move || early.send("registered").unwrap());
        let waiter = {
            let cancellation = cancellation.clone();
            thread::spawn(move || cancellation.wait_timeout(Duration::from_secs(10)))
        };
        cancellation.cancel();
        cancellation.cancel();
        assert!(waiter.join().unwrap());
        cancellation.on_cancel(move || sender.send("late").unwrap());
        let received: Vec<_> = receiver.try_iter().collect();
        assert_eq!(received, ["registered", "late"]);
        assert_eq!(cancellation.check(), Err(Cancelled));
    }
}
