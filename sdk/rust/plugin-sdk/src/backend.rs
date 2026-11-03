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
use crate::host::{Channel, Credentials, Host, Outcome, Outgoing, WorkspaceContext};
use crate::protocol::{
    bounded_text, error_codes, is_command_id, is_function_id, methods, CONTEXT_VERSION,
    FUNCTIONS_VERSIONS, MAX_FUNCTIONS, MAX_MESSAGE_BYTES, MAX_RESULT_BYTES,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Map, Value};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Display;
use std::io::{self, BufReader, Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

type Handler = Arc<dyn Fn(Value, &Context) -> Result<Value, FunctionError> + Send + Sync>;
type InitializeHook = Box<dyn FnMut(&mut Initialize) -> Result<(), FunctionError> + Send>;
type ShutdownHook = Box<dyn FnMut() + Send>;
type CommandHandler =
    Arc<dyn Fn(&str, Option<WorkspaceContext>) -> Result<(), FunctionError> + Send + Sync>;
type NotificationHandler = Arc<dyn Fn(Value) -> Result<(), FunctionError> + Send + Sync>;
type Answer = Result<Value, Value>;

/// `Idle` → `Listening` → `Initializing` → `Ready` → `ShuttingDown` → `Exited`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Idle,
    Listening,
    Initializing,
    Ready,
    ShuttingDown,
    Exited,
}

pub(crate) struct State {
    pub(crate) phase: Phase,
    functions: BTreeMap<String, Handler>,
    keymap_providers: BTreeMap<String, Handler>,
    selected_keymap_providers: BTreeSet<String>,
    /// The negotiated plugin-function protocol version.
    functions_version: Option<u64>,
    /// Host requests being handled, by JSON-RPC ID.
    incoming: HashMap<String, Running>,
    /// Requests this backend sent to the host, by JSON-RPC ID.
    pub(crate) outgoing: HashMap<String, Outgoing>,
    pub(crate) channel: Channel,
}

struct Running {
    function_id: String,
    cancellation: Cancellation,
}

/// Everything handler threads share with the session. Locks are taken in the
/// order `registrations`, `capabilities`, `state`, `output`, never the other
/// way round.
pub(crate) struct Shared {
    name: String,
    pub(crate) state: Mutex<State>,
    output: Mutex<Option<Box<dyn Write + Send>>>,
    /// Serializes register/unregister so the host sees them in call order.
    registrations: Mutex<()>,
    pub(crate) next_request: AtomicU64,
    /// Signalled when a host-request slot frees or a waiting request must
    /// give up; waits on `state`.
    pub(crate) slots: Condvar,
    /// The session's `getCapabilities` answer.
    pub(crate) capabilities: Mutex<Option<Value>>,
}

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
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
    commands: HashMap<String, CommandHandler>,
    contexts_changed: Option<NotificationHandler>,
    host_notifications: HashMap<String, NotificationHandler>,
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
                    keymap_providers: BTreeMap::new(),
                    selected_keymap_providers: BTreeSet::new(),
                    functions_version: None,
                    incoming: HashMap::new(),
                    outgoing: HashMap::new(),
                    channel: Channel::default(),
                }),
                output: Mutex::new(None),
                registrations: Mutex::new(()),
                next_request: AtomicU64::new(1),
                slots: Condvar::new(),
                capabilities: Mutex::new(None),
            }),
            version: None,
            initialize_hooks: Vec::new(),
            shutdown_hooks: Vec::new(),
            commands: HashMap::new(),
            contexts_changed: None,
            host_notifications: HashMap::new(),
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

    /// Declares an independent typed shortcut import provider. Its handler
    /// receives an opaque snapshot, never a filesystem path; it can use the
    /// call-scoped host to list and read the explicitly selected inputs.
    pub fn keymap_import_provider<F>(&mut self, id: &str, handler: F) -> Result<&mut Self, SdkError>
    where
        F: Fn(KeymapImportRequest, &Context) -> Result<Value, FunctionError>
            + Send
            + Sync
            + 'static,
    {
        if !valid_keymap_provider_id(id) {
            return Err(SdkError::new(
                "sdk.keymap.invalidID",
                "Invalid shortcut import provider ID.",
            ));
        }
        let mut state = lock(&self.shared.state);
        if state.keymap_providers.contains_key(id) || state.keymap_providers.len() >= 8 {
            return Err(SdkError::new(
                "sdk.keymap.invalidRegistration",
                "Duplicate provider or provider limit exceeded.",
            ));
        }
        state.keymap_providers.insert(
            id.to_string(),
            Arc::new(move |params, context| {
                handler(
                    KeymapImportRequest {
                        provider_id: params["providerID"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        snapshot_id: params["snapshotID"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    },
                    context,
                )
            }),
        );
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

    /// Handles `lithe/host/commands/didRun` for one of the plugin's commands
    /// (protocol 3), with the workspace context the command ran in. It is an
    /// event: the host waits for no result, and a failure is only logged.
    /// Each run has a thread of its own. A later handler for the same ID
    /// replaces the earlier one.
    ///
    /// `command_id` is the command's ID from the manifest, 1-128 of
    /// `[a-z0-9.-]`.
    pub fn on_command<F>(&mut self, command_id: &str, handler: F) -> Result<&mut Self, SdkError>
    where
        F: Fn(&str, Option<WorkspaceContext>) -> Result<(), FunctionError> + Send + Sync + 'static,
    {
        if !is_command_id(command_id) {
            return Err(SdkError::new(
                "sdk.command.invalidID",
                format!("Command ID {command_id:?} must be 1-128 of [a-z0-9.-]."),
            ));
        }
        self.commands
            .insert(command_id.to_string(), Arc::new(handler));
        Ok(self)
    }

    /// Handles `lithe/host/workspace/didChangeContexts` (protocol 3) with
    /// `{ revision }`; call `host().workspace().list_contexts()` for the
    /// contexts.
    pub fn on_contexts_changed<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(Value) -> Result<(), FunctionError> + Send + Sync + 'static,
    {
        self.contexts_changed = Some(Arc::new(handler));
        self
    }

    /// Handles a `lithe/host/*` notification a later host adds. Commands,
    /// context changes, and subscription events have their own APIs.
    pub fn on_host_notification<F>(
        &mut self,
        method: &str,
        handler: F,
    ) -> Result<&mut Self, SdkError>
    where
        F: Fn(Value) -> Result<(), FunctionError> + Send + Sync + 'static,
    {
        if !method.starts_with(methods::HOST_PREFIX) {
            return Err(SdkError::new(
                "sdk.method.invalid",
                format!("{method} is not a host notification."),
            ));
        }
        if [
            methods::DID_CHANGE_CONTEXTS,
            methods::SUBSCRIPTION_NOTIFY,
            methods::COMMAND_DID_RUN,
        ]
        .contains(&method)
        {
            return Err(SdkError::new(
                "sdk.method.reserved",
                format!("{method} is routed by the SDK; use on_command, on_contexts_changed, or Host::subscribe."),
            ));
        }
        self.host_notifications
            .insert(method.to_string(), Arc::new(handler));
        Ok(self)
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
                Phase::Ready if method == "lithe/keymap/import" => {
                    Shared::invoke_keymap(&self.shared, id, &params)
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
            method if method.starts_with(methods::HOST_PREFIX) => {
                self.host_notification(method, params);
                None
            }
            // `initialized` and anything else a plugin backend is not sent.
            _ => None,
        }
    }

    /// Routes a host notification. Core checked its shape and the session's
    /// grant; one this SDK does not know is ignored, as the contract asks.
    fn host_notification(&self, method: &str, params: &Value) {
        if self.shared.functions_version() < Some(CONTEXT_VERSION) || !params.is_object() {
            return;
        }
        match method {
            methods::SUBSCRIPTION_NOTIFY => self.shared.deliver(params),
            methods::COMMAND_DID_RUN => {
                let command_id = params["commandID"].as_str().unwrap_or_default();
                let Some(handler) = self.commands.get(command_id).cloned() else {
                    return self
                        .shared
                        .log(format_args!("No handler for command {command_id}."));
                };
                let command_id = command_id.to_string();
                let context = WorkspaceContext::from_reference(&params["context"]);
                self.shared
                    .notify_handler(method, move || handler(&command_id, context));
            }
            _ => {
                let handler = if method == methods::DID_CHANGE_CONTEXTS {
                    self.contexts_changed.as_ref()
                } else {
                    self.host_notifications.get(method)
                };
                if let Some(handler) = handler.cloned() {
                    let params = params.clone();
                    self.shared.notify_handler(method, move || handler(params));
                }
            }
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
        let keymap_offer = params
            .pointer("/capabilities/experimental/litheKeymapImport")
            .cloned();
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
        if let Some(offer) = keymap_offer {
            let version = offer
                .get("versions")
                .and_then(Value::as_array)
                .is_some_and(|versions| versions.iter().any(|version| version.as_u64() == Some(1)));
            if version {
                if let Some(providers) = offer
                    .get("providers")
                    .and_then(Value::as_array)
                    .filter(|ids| ids.len() <= 8)
                {
                    state.selected_keymap_providers = providers
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|id| state.keymap_providers.contains_key(*id))
                        .map(str::to_string)
                        .collect();
                    if !state.selected_keymap_providers.is_empty() {
                        experimental.insert(
                            "litheKeymapImport".into(),
                            json!({"version":1,"providers":state.selected_keymap_providers}),
                        );
                    }
                }
            }
        }
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
                "The host did not offer a litheFunctions version from 1 to 3; functions stay unavailable.",
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

    pub(crate) fn functions_version(&self) -> Option<u64> {
        lock(&self.state).functions_version
    }

    pub(crate) fn log(&self, message: impl Display) {
        let _ = writeln!(io::stderr(), "[{}] {message}", self.name);
    }

    fn finish(&self, code: i32) -> i32 {
        let running = {
            let mut state = lock(&self.state);
            state.phase = Phase::Exited;
            self.close_channel(&mut state);
            state.running_cancellations()
        };
        running.iter().for_each(Cancellation::cancel);
        *lock(&self.output) = None;
        code
    }

    /// Runs a notification handler off the reader thread; a failure is
    /// logged.
    fn notify_handler(
        self: &Arc<Self>,
        method: &str,
        handler: impl FnOnce() -> Result<(), FunctionError> + Send + 'static,
    ) {
        let shared = Arc::clone(self);
        let method = method.to_string();
        let spawned = thread::Builder::new()
            .name(format!("lithe-notification-{method}"))
            .spawn(move || match catch_unwind(AssertUnwindSafe(handler)) {
                Ok(Ok(())) => {}
                Ok(Err(error)) => shared.log(format_args!("Notification {method} failed: {error}")),
                Err(panic) => shared.log(format_args!(
                    "Notification {method} panicked: {}",
                    panic_message(&panic)
                )),
            });
        if let Err(error) = spawned {
            self.log(format_args!(
                "A notification handler could not start: {error}"
            ));
        }
    }

    pub(crate) fn write(&self, message: &Value) {
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

    fn invoke_keymap(shared: &Arc<Self>, id: Value, params: &Value) {
        let provider = params
            .get("providerID")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let snapshot = params
            .get("snapshotID")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !params.as_object().is_some_and(|object| object.len() == 2)
            || !valid_keymap_provider_id(provider)
            || !valid_snapshot_id(snapshot)
        {
            return shared.respond(
                &id,
                Err(rpc_error(
                    error_codes::INVALID_PARAMS,
                    "Invalid shortcut import request.",
                )),
            );
        }
        let call_id = format!("keymap:{provider}");
        let key = request_key(&id);
        let cancellation = Cancellation::new();
        let handler = {
            let mut state = lock(&shared.state);
            let handler = state
                .selected_keymap_providers
                .contains(provider)
                .then(|| state.keymap_providers.get(provider).cloned())
                .flatten();
            if handler.is_some() {
                state.incoming.insert(
                    key.clone(),
                    Running {
                        function_id: call_id.clone(),
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
                    "Shortcut import provider was not negotiated.",
                )),
            );
        };
        let worker = Arc::clone(shared);
        let call_params = params.clone();
        let thread_key = key.clone();
        let thread_id = id.clone();
        let spawned = thread::Builder::new()
            .name("lithe-keymap-import".into())
            .spawn(move || {
                let ended = Cancellation::new();
                let context = Context {
                    shared: Arc::clone(&worker),
                    function_id: call_id,
                    operation_id: Value::Null,
                    host: Host::new(
                        Arc::clone(&worker),
                        vec![cancellation.clone(), ended.clone()],
                    ),
                    cancellation,
                    workspace_context: None,
                    reporting: AtomicBool::new(false),
                };
                let outcome = catch_unwind(AssertUnwindSafe(|| handler(call_params, &context)));
                ended.cancel();
                let answer = worker.function_answer(&context, outcome);
                lock(&worker.state).incoming.remove(&thread_key);
                worker.respond(&thread_id, answer);
            });
        if let Err(error) = spawned {
            lock(&shared.state).incoming.remove(&key);
            shared.log(format_args!("The import could not start: {error}"));
            shared.respond(&id, Err(internal_response()));
        }
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
            let workspace_context = state
                .functions_version
                .filter(|version| *version >= CONTEXT_VERSION)
                .and_then(|_| WorkspaceContext::from_reference(&params["context"]));
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
            handler.map(|handler| (handler, workspace_context))
        };
        let Some((handler, workspace_context)) = handler else {
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
                // Ends host requests the call left behind, on clones of its host.
                let ended = Cancellation::new();
                let context = Context {
                    shared: Arc::clone(&worker),
                    function_id,
                    operation_id,
                    host: Host::new(
                        Arc::clone(&worker),
                        vec![cancellation.clone(), ended.clone()],
                    ),
                    cancellation,
                    workspace_context,
                    reporting: AtomicBool::new(true),
                };
                let outcome = catch_unwind(AssertUnwindSafe(|| handler(call_params, &context)));
                context.reporting.store(false, Ordering::SeqCst);
                ended.cancel();
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

    /// A `lithe/function/*` request; it takes no host-request slot.
    fn request(self: &Arc<Self>, method: &str, params: Value) -> Outcome {
        self.exchange(method, params, false, &[], None)
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
    host: Host,
    workspace_context: Option<WorkspaceContext>,
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

    /// Host services for this call. A request still waiting for the host
    /// when the call is cancelled or ends fails with `sdk.call.cancelled`,
    /// which `?` turns into a cancelled answer, and from protocol 3 the SDK
    /// cancels it on the wire.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// The `{ contextID, generation }` of the window the call came from
    /// (protocol 3); `None` for calls without one, such as from the plugin's
    /// own UI, and before protocol 3.
    pub fn workspace_context(&self) -> Option<&WorkspaceContext> {
        self.workspace_context.as_ref()
    }

    /// `host().credentials()`.
    pub fn credentials(&self) -> Credentials {
        self.host.credentials()
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

/// A single host-authorized source snapshot for a shortcut import.
pub struct KeymapImportRequest {
    provider_id: String,
    snapshot_id: String,
}

/// A source file in the host's selected, read-only snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapSourceFile {
    pub id: String,
    pub path: String,
    pub byte_count: u64,
}

impl KeymapImportRequest {
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Lists the declared files captured for this request. The path is
    /// relative to the selected source and is only a label, never authority.
    pub fn list_inputs(&self, host: &Host) -> Result<Vec<KeymapSourceFile>, SdkError> {
        let result = host.request(
            "lithe/host/keymapImport/listInputs",
            json!({
                "providerID": self.provider_id, "snapshotID": self.snapshot_id
            }),
        )?;
        let invalid = || {
            SdkError::new(
                "sdk.keymap.invalidResponse",
                "Invalid shortcut source list.",
            )
        };
        let files = result
            .get("files")
            .and_then(Value::as_array)
            .filter(|files| files.len() <= 128)
            .ok_or_else(invalid)?;
        files
            .iter()
            .map(|file| {
                let id = file
                    .get("id")
                    .and_then(Value::as_str)
                    .filter(|id| valid_snapshot_id(id))
                    .ok_or_else(invalid)?;
                let path = file
                    .get("path")
                    .and_then(Value::as_str)
                    .filter(|path| !path.is_empty() && path.len() <= 4096)
                    .ok_or_else(invalid)?;
                let byte_count = file
                    .get("byteCount")
                    .and_then(Value::as_u64)
                    .filter(|size| *size <= 1_048_576)
                    .ok_or_else(invalid)?;
                Ok(KeymapSourceFile {
                    id: id.into(),
                    path: path.into(),
                    byte_count,
                })
            })
            .collect()
    }

    /// Reads one bounded chunk from a file ID returned by `list_inputs`.
    pub fn read_chunk(
        &self,
        host: &Host,
        file_id: &str,
        offset: u64,
        maximum_bytes: u32,
    ) -> Result<Vec<u8>, SdkError> {
        let invalid = || SdkError::new("sdk.keymap.invalidChunk", "Invalid shortcut source chunk.");
        if !valid_snapshot_id(file_id)
            || offset > 1_048_576
            || maximum_bytes == 0
            || maximum_bytes > 131_072
        {
            return Err(invalid());
        }
        let result = host.request(
            "lithe/host/keymapImport/readChunk",
            json!({
                "providerID": self.provider_id, "snapshotID": self.snapshot_id,
                "fileID": file_id, "offset": offset, "maximumBytes": maximum_bytes
            }),
        )?;
        let encoded = result
            .get("data")
            .and_then(Value::as_str)
            .filter(|data| data.len() <= 174_764)
            .ok_or_else(invalid)?;
        let bytes = BASE64.decode(encoded).map_err(|_| invalid())?;
        if bytes.len() > maximum_bytes as usize {
            return Err(invalid());
        }
        Ok(bytes)
    }
}

fn valid_keymap_provider_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.as_bytes()[0].is_ascii_lowercase()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_snapshot_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
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
            .request(methods::FUNCTION_UNREGISTER, json!({ "ids": [id] }))?;
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

    /// Host services, for work outside a function call; usable once the
    /// session is ready.
    pub fn host(&self) -> Host {
        Host::new(Arc::clone(&self.shared), Vec::new())
    }

    /// `host().credentials()`.
    pub fn credentials(&self) -> Credentials {
        self.host().credentials()
    }

    /// Writes a diagnostic line to stderr, which the host keeps in its log.
    pub fn log(&self, message: impl Display) {
        self.shared.log(message);
    }
}

/// The cancellation of one call, or of work a plugin may stop for its own
/// reasons (see [`Host::with_cancellation`]). Clones observe the same token.
#[derive(Clone)]
pub struct Cancellation(Arc<CancellationInner>);

struct CancellationInner {
    state: Mutex<CancellationState>,
    changed: Condvar,
}

struct CancellationState {
    cancelled: bool,
    next_watch: u64,
    callbacks: Vec<(u64, Box<dyn FnOnce() + Send>)>,
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

impl Cancellation {
    pub fn new() -> Self {
        Self(Arc::new(CancellationInner {
            state: Mutex::new(CancellationState {
                cancelled: false,
                next_watch: 0,
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
        let _ = self.watch(callback);
    }

    /// Cancels: wakes waiters and runs the callbacks, once.
    pub fn cancel(&self) {
        let callbacks = {
            let mut state = lock(&self.0.state);
            if state.cancelled {
                return;
            }
            state.cancelled = true;
            std::mem::take(&mut state.callbacks)
        };
        self.0.changed.notify_all();
        for (_, callback) in callbacks {
            callback();
        }
    }

    /// Like [`on_cancel`](Self::on_cancel), but removable with
    /// [`unwatch`](Self::unwatch); `None` when already cancelled, after
    /// running `callback`.
    pub(crate) fn watch(&self, callback: impl FnOnce() + Send + 'static) -> Option<u64> {
        let mut state = lock(&self.0.state);
        if state.cancelled {
            drop(state);
            callback();
            return None;
        }
        let watch = state.next_watch;
        state.next_watch += 1;
        state.callbacks.push((watch, Box::new(callback)));
        Some(watch)
    }

    pub(crate) fn unwatch(&self, watch: u64) {
        lock(&self.0.state)
            .callbacks
            .retain(|(other, _)| *other != watch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

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
