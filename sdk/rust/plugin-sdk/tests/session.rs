//! The backend session driven by a scripted host over in-process pipes.
//! Every wait is bounded, so a broken session fails here with the frames it
//! sent instead of hanging.

use lithe_plugin_sdk::{Backend, FunctionError, PluginError, SdkError};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(10);

/// The host side of one session.
struct Host {
    input: Option<io::PipeWriter>,
    frames: mpsc::Receiver<Value>,
    exit: mpsc::Receiver<i32>,
}

impl Host {
    fn start(backend: Backend) -> Self {
        let (backend_input, input) = io::pipe().expect("input pipe");
        let (output, backend_output) = io::pipe().expect("output pipe");
        let (exit_sender, exit) = mpsc::channel();
        thread::spawn(move || {
            let _ = exit_sender.send(backend.serve(backend_input, backend_output));
        });
        let (frame_sender, frames) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(output);
            while let Some(frame) = read_frame(&mut reader) {
                if frame_sender.send(frame).is_err() {
                    return;
                }
            }
        });
        Self {
            input: Some(input),
            frames,
            exit,
        }
    }

    fn send_raw(&mut self, bytes: &[u8]) {
        let input = self.input.as_mut().expect("open input");
        input.write_all(bytes).expect("write to the backend");
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        self.send_raw(format!("Content-Length: {}\r\n\r\n{body}", body.len()).as_bytes());
    }

    fn request(&mut self, id: impl Into<Value>, method: &str, params: Value) {
        self.send(json!({ "jsonrpc": "2.0", "id": id.into(), "method": method, "params": params }));
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn next(&self) -> Value {
        self.frames
            .recv_timeout(WAIT)
            .expect("a frame from the backend in time")
    }

    /// The answer to request `id`, after any frames before it.
    fn answer(&self, id: impl Into<Value>) -> (Value, Vec<Value>) {
        let id = id.into();
        let mut before = Vec::new();
        loop {
            let frame = self.next();
            if frame["id"] == id && frame.get("method").is_none() {
                return (frame, before);
            }
            before.push(frame);
        }
    }

    fn initialize(&mut self, versions: Value) -> Value {
        let capabilities = if versions.is_null() {
            json!({})
        } else {
            json!({ "experimental": { "litheFunctions": { "versions": versions } } })
        };
        self.request(
            0,
            "initialize",
            json!({ "rootUri": "file:///tmp/state/", "capabilities": capabilities }),
        );
        self.answer(0).0["result"].clone()
    }

    fn invoke(&mut self, id: i64, function: &str, params: Value) {
        self.request(
            id,
            "lithe/function/invoke",
            json!({ "functionID": function, "operationID": format!("op-{id}"), "params": params }),
        );
    }

    /// Ends the input stream and returns the backend's exit code.
    fn close(mut self) -> i32 {
        self.input = None;
        self.exit.recv_timeout(WAIT).expect("the backend to end")
    }

    fn exit_code(&self) -> i32 {
        self.exit.recv_timeout(WAIT).expect("the backend to end")
    }
}

fn read_frame(reader: &mut impl BufRead) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            length = value.parse().ok();
        }
    }
    let mut body = vec![0; length?];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn echo_backend() -> Backend {
    let mut backend = Backend::new("test-backend").version("1.2.3");
    backend
        .function("echo.value", |params, _| Ok(params))
        .expect("valid function");
    backend
}

#[test]
fn negotiates_the_newest_offered_version_and_lists_the_functions() {
    for (offered, version) in [(json!([1, 2]), json!(2)), (json!([1]), json!(1))] {
        let mut host = Host::start(echo_backend());
        let result = host.initialize(offered);
        assert_eq!(
            result,
            json!({
                "capabilities": { "experimental": {
                    "litheFunctions": { "version": version, "functions": [{ "id": "echo.value" }] }
                } },
                "serverInfo": { "name": "test-backend", "version": "1.2.3" }
            })
        );
        host.invoke(1, "echo.value", json!({ "a": 1 }));
        assert_eq!(host.answer(1).0["result"], json!({ "a": 1 }));
        assert_eq!(host.close(), 1);
    }
}

#[test]
fn a_host_without_plugin_functions_gets_no_functions() {
    let mut host = Host::start(echo_backend());
    assert_eq!(
        host.initialize(Value::Null)["capabilities"],
        json!({ "experimental": {} })
    );
    host.invoke(1, "echo.value", json!({}));
    assert_eq!(host.answer(1).0["error"]["code"], -32602);
    host.close();
}

#[test]
fn follows_the_lsp_lifecycle() {
    let mut host = Host::start(echo_backend());
    host.invoke(1, "echo.value", json!({}));
    assert_eq!(host.answer(1).0["error"]["code"], -32002);
    host.send_raw(b"Content-Length: 3\r\n\r\nnot");
    let malformed = host.next();
    assert_eq!(
        (malformed["id"].clone(), malformed["error"]["code"].clone()),
        (Value::Null, json!(-32700))
    );
    host.send(json!([1, 2]));
    assert_eq!(host.next()["error"]["code"], -32600);

    host.initialize(json!([1, 2]));
    host.request(2, "initialize", json!({}));
    assert_eq!(host.answer(2).0["error"]["code"], -32600);
    host.request(3, "textDocument/hover", json!({}));
    assert_eq!(host.answer(3).0["error"]["code"], -32601);
    host.request(4, "shutdown", Value::Null);
    assert_eq!(
        host.answer(4).0,
        json!({ "jsonrpc": "2.0", "id": 4, "result": null })
    );
    host.invoke(5, "echo.value", json!({}));
    assert_eq!(host.answer(5).0["error"]["code"], -32600);
    host.notify("exit", Value::Null);
    assert_eq!(host.exit_code(), 0);
}

#[test]
fn exit_without_shutdown_and_a_broken_frame_end_with_a_failure() {
    let mut host = Host::start(echo_backend());
    host.notify("exit", Value::Null);
    assert_eq!(host.exit_code(), 1);

    let mut host = Host::start(echo_backend());
    host.send_raw(b"Content-Length: nope\r\n\r\n");
    assert_eq!(host.exit_code(), 1);
}

#[test]
fn maps_results_failures_and_panics() {
    let mut backend = Backend::new("test-backend");
    backend
        .function("work.run", |params, context| {
            context.progress("Halfway", Some(150));
            match params["mode"].as_str() {
                Some("plugin") => Err(PluginError::new("work.failed", "The work failed.").into()),
                Some("internal") => Err(FunctionError::internal("/private/path: denied")),
                Some("panic") => panic!("secret detail"),
                Some("large") => Ok(json!("x".repeat(1 << 20))),
                _ => Ok(json!({ "done": true })),
            }
        })
        .unwrap();
    let mut host = Host::start(backend);
    host.initialize(json!([1]));

    host.invoke(1, "work.run", json!({}));
    let (answer, before) = host.answer(1);
    assert_eq!(answer["result"], json!({ "done": true }));
    assert_eq!(
        before,
        [json!({
            "jsonrpc": "2.0", "method": "lithe/function/progress",
            "params": { "operationID": "op-1", "message": "Halfway", "percentage": 100 }
        })]
    );

    host.invoke(2, "work.run", json!({ "mode": "plugin" }));
    assert_eq!(
        host.answer(2).0["error"],
        json!({ "code": -32000, "message": "The work failed.", "data": { "code": "work.failed" } })
    );
    for (id, mode) in [(3, "internal"), (4, "panic"), (5, "large")] {
        host.invoke(id, "work.run", json!({ "mode": mode }));
        assert_eq!(
            host.answer(id).0["error"],
            json!({ "code": -32603, "message": "The plugin backend failed unexpectedly." }),
            "{mode}"
        );
    }
    // The session survives a panicking call.
    host.invoke(6, "work.run", json!({}));
    assert_eq!(host.answer(6).0["result"], json!({ "done": true }));
    host.close();
}

#[test]
fn cancel_request_cancels_the_running_call() {
    let mut backend = Backend::new("test-backend");
    backend
        .function("work.wait", |_, context| {
            context.progress("Waiting", None);
            if context.cancellation().wait_timeout(Duration::from_secs(30)) {
                context.check_cancelled()?;
            }
            Ok(json!("not cancelled"))
        })
        .unwrap();
    let mut host = Host::start(backend);
    host.initialize(json!([1]));
    host.invoke(7, "work.wait", json!({}));
    assert_eq!(host.next()["method"], "lithe/function/progress");
    host.notify("$/cancelRequest", json!({ "id": 7 }));
    assert_eq!(
        host.answer(7).0["error"],
        json!({ "code": -32800, "message": "The request was cancelled." })
    );
    host.close();
}

#[test]
fn shutdown_cancels_running_calls() {
    let mut backend = Backend::new("test-backend");
    let (sender, receiver) = mpsc::channel();
    backend
        .function("work.wait", |_, context| {
            context.progress("Waiting", None);
            context.cancellation().wait_timeout(Duration::from_secs(30));
            context.check_cancelled()?;
            Ok(Value::Null)
        })
        .unwrap();
    backend.on_shutdown(move || sender.send("shutdown hook").unwrap());
    let mut host = Host::start(backend);
    host.initialize(json!([1]));
    host.invoke(1, "work.wait", json!({}));
    assert_eq!(host.next()["method"], "lithe/function/progress");
    host.request(2, "shutdown", Value::Null);
    let mut answers: Vec<_> = (0..2).map(|_| host.next()).collect();
    answers.sort_by_key(|answer| answer["id"].as_i64());
    assert_eq!(answers[0]["error"]["code"], -32800);
    assert_eq!(answers[1]["result"], Value::Null);
    assert_eq!(receiver.recv_timeout(WAIT), Ok("shutdown hook"));
    host.close();
}

/// A function that stores, reads, and deletes a credential.
fn credentials_backend() -> Backend {
    let mut backend = Backend::new("test-backend");
    backend
        .function("secret.roundTrip", |params, context| {
            let credentials = context.credentials();
            let key = params["key"].as_str().unwrap_or_default();
            if let Err(error) = credentials.set(key, "s3cr3t") {
                return Err(PluginError::new("test.credentials", error.code().to_string()).into());
            }
            let stored = credentials.get(key)?;
            credentials.delete(key)?;
            Ok(json!({ "stored": stored }))
        })
        .unwrap();
    backend
}

#[test]
fn credentials_are_host_requests_of_protocol_two() {
    let mut host = Host::start(credentials_backend());
    host.initialize(json!([1, 2]));
    host.invoke(1, "secret.roundTrip", json!({ "key": "db:main" }));
    let set = host.next();
    assert_eq!(set["method"], "lithe/host/credentials/set");
    assert_eq!(
        set["params"],
        json!({ "key": "db:main", "secret": "s3cr3t" })
    );
    host.send(json!({ "jsonrpc": "2.0", "id": set["id"], "result": null }));
    let get = host.next();
    assert_eq!(get["method"], "lithe/host/credentials/get");
    assert_eq!(get["params"], json!({ "key": "db:main" }));
    host.send(json!({ "jsonrpc": "2.0", "id": get["id"], "result": { "secret": "s3cr3t" } }));
    let delete = host.next();
    assert_eq!(delete["method"], "lithe/host/credentials/delete");
    host.send(json!({ "jsonrpc": "2.0", "id": delete["id"], "result": null }));
    assert_eq!(host.answer(1).0["result"], json!({ "stored": "s3cr3t" }));

    // Core's refusal reaches the handler with its code.
    host.invoke(2, "secret.roundTrip", json!({ "key": "db:main" }));
    let set = host.next();
    host.send(json!({
        "jsonrpc": "2.0", "id": set["id"],
        "error": { "code": -32601, "message": "Not granted.", "data": { "code": "plugin.host.permissionDenied" } }
    }));
    assert_eq!(
        host.answer(2).0["error"]["message"],
        "plugin.host.permissionDenied"
    );

    // A malformed key never reaches the host.
    host.invoke(3, "secret.roundTrip", json!({ "key": "has space" }));
    assert_eq!(
        host.answer(3).0["error"]["message"],
        "sdk.credentials.invalidKey"
    );
    host.close();
}

#[test]
fn credentials_need_protocol_two() {
    let mut host = Host::start(credentials_backend());
    host.initialize(json!([1]));
    host.invoke(1, "secret.roundTrip", json!({ "key": "db:main" }));
    let (answer, before) = host.answer(1);
    assert!(before.is_empty(), "{before:?}");
    assert_eq!(answer["error"]["message"], "sdk.protocol.notNegotiated");
    host.close();
}

#[test]
fn cancelling_a_call_ends_its_waiting_host_request() {
    let mut host = Host::start(credentials_backend());
    host.initialize(json!([2]));
    host.invoke(1, "secret.roundTrip", json!({ "key": "db:main" }));
    assert_eq!(host.next()["method"], "lithe/host/credentials/set");
    host.notify("$/cancelRequest", json!({ "id": 1 }));
    assert_eq!(host.answer(1).0["error"]["code"], -32800);
    host.close();
}

#[test]
fn functions_are_registered_and_withdrawn_at_runtime() {
    let mut backend = Backend::new("test-backend");
    backend
        .function("extra.add", |_, context| {
            context
                .backend()
                .register_function("extra.value", |_, _| Ok(json!(42)))?;
            Ok(json!(context.backend().function_ids()))
        })
        .unwrap();
    backend
        .function("extra.remove", |_, context| {
            context.backend().unregister_function("extra.value")?;
            Ok(Value::Null)
        })
        .unwrap();
    let handle = backend.handle();
    let mut host = Host::start(backend);
    host.initialize(json!([1]));

    host.invoke(1, "extra.add", json!({}));
    let register = host.next();
    assert_eq!(register["method"], "lithe/function/register");
    assert_eq!(
        register["params"],
        json!({ "functions": [{ "id": "extra.value" }] })
    );
    host.send(json!({ "jsonrpc": "2.0", "id": register["id"], "result": null }));
    assert_eq!(
        host.answer(1).0["result"],
        json!(["extra.add", "extra.remove", "extra.value"])
    );
    host.invoke(2, "extra.value", json!({}));
    assert_eq!(host.answer(2).0["result"], 42);

    host.invoke(3, "extra.remove", json!({}));
    let unregister = host.next();
    assert_eq!(unregister["params"], json!({ "ids": ["extra.value"] }));
    host.send(json!({ "jsonrpc": "2.0", "id": unregister["id"], "result": null }));
    assert_eq!(host.answer(3).0["result"], Value::Null);
    host.invoke(4, "extra.value", json!({}));
    assert_eq!(host.answer(4).0["error"]["code"], -32602);

    // A refused registration leaves the function out.
    let registering = {
        let handle = handle.clone();
        thread::spawn(move || handle.register_function("extra.refused", |_, _| Ok(Value::Null)))
    };
    let register = host.next();
    host.send(json!({
        "jsonrpc": "2.0", "id": register["id"],
        "error": { "code": -32602, "message": "No.", "data": { "code": "plugin.function.invalidRegistration" } }
    }));
    let refused: Result<(), SdkError> = registering.join().unwrap();
    assert_eq!(
        refused.unwrap_err().code(),
        "plugin.function.invalidRegistration"
    );
    assert!(!handle.function_ids().contains(&"extra.refused".to_string()));
    host.close();
    // After the session, requests fail instead of waiting.
    assert_eq!(
        handle
            .register_function("extra.late", |_, _| Ok(Value::Null))
            .unwrap_err()
            .code(),
        "sdk.session.closed"
    );
}

#[test]
fn function_declarations_are_checked() {
    let mut backend = Backend::new("test-backend");
    let invalid = backend
        .function("Bad", |_, _| Ok(Value::Null))
        .err()
        .unwrap();
    assert_eq!(invalid.code(), "sdk.function.invalidID");
    backend.function("a.b", |_, _| Ok(Value::Null)).unwrap();
    let duplicate = backend
        .function("a.b", |_, _| Ok(Value::Null))
        .err()
        .unwrap();
    assert_eq!(duplicate.code(), "sdk.function.duplicate");
    assert_eq!(
        backend
            .handle()
            .register_function("c.d", |_, _| Ok(Value::Null))
            .unwrap_err()
            .code(),
        "sdk.functions.unavailable"
    );
}

#[test]
fn initialize_hooks_see_the_root_and_add_functions() {
    let mut backend = Backend::new("test-backend");
    let (sender, receiver) = mpsc::channel();
    backend.on_initialize(move |initialize| {
        sender.send(initialize.root_path()).unwrap();
        initialize.function("late.added", |_, _| Ok(json!("added")))?;
        Ok(())
    });
    let mut host = Host::start(backend);
    let result = host.initialize(json!([1, 2]));
    assert_eq!(
        result["capabilities"]["experimental"]["litheFunctions"]["functions"],
        json!([{ "id": "late.added" }])
    );
    assert_eq!(
        receiver.recv_timeout(WAIT).unwrap(),
        Some(std::path::PathBuf::from("/tmp/state"))
    );

    let mut refusing = Backend::new("test-backend");
    refusing.on_initialize(|_| Err(PluginError::new("test.notReady", "Not ready.").into()));
    let mut host = Host::start(refusing);
    host.request(0, "initialize", json!({ "capabilities": {} }));
    assert_eq!(host.answer(0).0["error"]["data"]["code"], "test.notReady");
    host.close();
}
