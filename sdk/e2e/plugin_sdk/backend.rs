//! End-to-end scenario of a plugin backend (`backend` in `plugin.json`),
//! run against Rust Core through the same JSON commands a host sends.
//!
//! The test plays the host: it expands the backend launch plan, starts the
//! sample under the plugin's backend provider ID with the granted host
//! services, and answers Core's `hostRequest` events from an in-memory
//! credential store the way the macOS host answers them from the Keychain.
//! The scenario drives functions with progress, a plugin failure, an
//! unexpected failure, cancellation, runtime registration, credentials, and
//! state across a restart, where the backend is granted no host service.

use super::{core, listing, Fixture, Session};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Runs the scenario against the installed package at `package`, whose
/// `plugin.json` declares the sample backend.
pub fn run_backend_scenario(package: &Path) {
    let manifest: Value =
        serde_json::from_slice(&fs::read(package.join("plugin.json")).expect("sample manifest"))
            .expect("sample manifest JSON");
    let plugin_id = manifest["id"].as_str().expect("plugin ID").to_string();
    let package_before = listing(package);
    let validated = core(
        "plugins.declarative.validate",
        json!({ "manifest": manifest }),
    );
    assert_eq!(validated["status"], "ok", "{validated}");

    let fixture = Fixture::new("backend");
    let state = fixture.root.join("state");
    let plan = launch_plan(&manifest, package, &state);
    assert_eq!(plan["backend"]["hostServices"], json!(["credentials"]));
    assert_eq!(plan["requiredDirectories"], json!(["state"]));
    fs::create_dir_all(&state).expect("host state directory");

    let mut session = start(&plugin_id, package, &plan, &state);
    assert_eq!(
        session.functions(),
        [
            "fail.unexpectedly",
            "note.read",
            "note.save",
            "secret.check",
            "secret.forget",
            "secret.remember",
            "work.wait"
        ]
    );

    // A function with progress, which registers note.clear on first use.
    let saved = session.call(
        "save",
        "note.save",
        json!({ "name": "first", "text": "héllo" }),
    );
    assert_eq!(
        saved["result"]["value"],
        json!({ "name": "first", "bytes": 6 }),
        "{saved}"
    );
    let progress: Vec<_> = session
        .events
        .iter()
        .filter(|event| event["type"] == "requestProgress" && event["operationId"] == "save")
        .map(|event| event["result"]["message"].clone())
        .collect();
    assert_eq!(progress, ["Saving", "Saved"]);
    session.await_event("note.clear to be registered", |event| {
        event["type"] == "functionsChanged"
            && event["result"]["functions"]
                .as_array()
                .is_some_and(|functions| functions.contains(&json!("note.clear")))
    });
    let read = session.call("read", "note.read", json!({ "name": "first" }));
    assert_eq!(read["result"]["value"], json!({ "text": "héllo" }));

    // A plugin failure keeps Lithe's stable code and forwards the plugin's own.
    let missing = session.call("missing", "note.read", json!({ "name": "other" }));
    assert_eq!(
        missing["error"]["code"], "plugin.function.failed",
        "{missing}"
    );
    assert_eq!(missing["error"]["pluginErrorCode"], "sample.noteMissing");

    // An unexpected failure reaches the host without its details.
    let failed = session.call("fail", "fail.unexpectedly", json!({}));
    assert_eq!(
        failed["error"]["code"], "plugin.function.failed",
        "{failed}"
    );
    assert!(failed["error"].get("pluginErrorCode").is_none(), "{failed}");
    assert!(
        !failed.to_string().contains(&*state.to_string_lossy()),
        "a machine path reached the host: {failed}"
    );

    // Credentials travel as host requests and never touch the state directory.
    let mut store = BTreeMap::new();
    let remembered = serve_host(
        &mut session,
        &mut store,
        "remember",
        "secret.remember",
        json!({ "key": "sample:password", "secret": "s3cr3t" }),
    );
    assert_eq!(remembered["result"]["value"], json!({}), "{remembered}");
    assert_eq!(
        store,
        BTreeMap::from([("sample:password".to_string(), "s3cr3t".to_string())])
    );
    for (operation, secret, matches) in [("check", "s3cr3t", true), ("check-wrong", "nope", false)]
    {
        let checked = serve_host(
            &mut session,
            &mut store,
            operation,
            "secret.check",
            json!({ "key": "sample:password", "secret": secret }),
        );
        assert_eq!(
            checked["result"]["value"],
            json!({ "stored": true, "matches": matches }),
            "{checked}"
        );
    }
    let forgotten = serve_host(
        &mut session,
        &mut store,
        "forget",
        "secret.forget",
        json!({ "key": "sample:password" }),
    );
    assert_eq!(forgotten["result"]["value"], Value::Null, "{forgotten}");
    assert!(store.is_empty());
    let requests: Vec<_> = session
        .events
        .iter()
        .filter(|event| event["type"] == "hostRequest")
        .map(|event| (event["method"].clone(), event["result"]["service"].clone()))
        .collect();
    assert_eq!(requests.len(), 4, "{requests:?}");
    assert!(requests.iter().all(|(_, service)| service == "credentials"));

    // Cancelling a running call reaches the handler.
    session.invoke("wait", "work.wait", json!({}));
    wait_for_file(&mut session, &state.join("waiting"));
    core(
        "lsp.cancelOperation",
        json!({ "sessionId": session.id, "operationId": "wait" }),
    );
    assert_eq!(
        session.completion("wait")["error"]["code"],
        "requestCancelled"
    );
    wait_for_file(&mut session, &state.join("cancelled"));

    // State survives a restart. Without credentials.store the host grants no
    // host service, and Core refuses the request before the host sees it.
    session.stop();
    drop(session);
    let mut ungranted = manifest.clone();
    ungranted["permissions"] = json!(["backend.launch", "ui.contribute", "ui.web"]);
    let plan = launch_plan(&ungranted, package, &state);
    assert_eq!(plan["backend"]["hostServices"], json!([]));
    let mut restarted = start(&plugin_id, package, &plan, &state);
    assert!(restarted.functions().contains(&"note.clear".to_string()));
    let read = restarted.call("read-again", "note.read", json!({ "name": "first" }));
    assert_eq!(read["result"]["value"], json!({ "text": "héllo" }));
    let refused = restarted.call(
        "remember-ungranted",
        "secret.remember",
        json!({ "key": "sample:password", "secret": "s3cr3t" }),
    );
    assert_eq!(
        refused["error"]["pluginErrorCode"], "sample.credentialsUnavailable",
        "{refused}"
    );
    assert!(
        !restarted
            .events
            .iter()
            .any(|event| event["type"] == "hostRequest"),
        "an ungranted request reached the host"
    );
    assert_eq!(
        restarted.call("clear", "note.clear", json!({}))["result"]["value"],
        Value::Null
    );
    let cleared = restarted.call("read-cleared", "note.read", json!({ "name": "first" }));
    assert_eq!(cleared["error"]["pluginErrorCode"], "sample.noteMissing");
    restarted.stop();
    drop(restarted);

    assert!(
        listing(package) == package_before,
        "the plugin package changed at runtime"
    );
    let left: BTreeSet<_> = fs::read_dir(&state)
        .expect("state directory")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        left,
        BTreeSet::from(["cancelled".to_string(), "waiting".to_string()])
    );
}

fn launch_plan(manifest: &Value, package: &Path, state: &Path) -> Value {
    let plan = core(
        "plugins.declarative.languageServerLaunchPlan",
        json!({
            "manifest": manifest,
            "backend": true,
            "request": {
                "pluginDirectory": package,
                "stateDirectory": state,
                "architecture": if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" },
            }
        }),
    );
    assert_eq!(plan["status"], "ok", "{plan}");
    assert_eq!(plan["plan"]["launch"]["kind"], "packaged");
    plan["plan"].clone()
}

/// Starts the backend as the host does: the packaged executable, under the
/// plugin's backend provider ID, with the state directory as root and
/// working directory and the plan's host services.
fn start(plugin_id: &str, package: &Path, plan: &Value, state: &Path) -> Session {
    let executable = package.join(plan["launch"]["path"].as_str().expect("packaged path"));
    let started = core(
        "lsp.startServer",
        json!({
            "providerId": format!("pluginBackend:{plugin_id}"),
            "pluginId": plugin_id,
            "executablePath": executable,
            "arguments": plan["arguments"],
            "rootUri": format!("file://{}/", state.display()),
            "workingDirectory": state,
            "hostServices": plan["backend"]["hostServices"],
            "initializeTimeoutMilliseconds": super::WAIT.as_millis() as u64,
            "requestTimeoutMilliseconds": super::WAIT.as_millis() as u64,
            "shutdownTimeoutMilliseconds": 5000,
        }),
    );
    let mut session = Session {
        id: started["sessionId"]
            .as_str()
            .expect("session ID")
            .to_string(),
        plugin_id: plugin_id.to_string(),
        events: Vec::new(),
    };
    session.await_event("the backend to become ready", |event| {
        event["type"] == "stateChanged" && event["state"] == "ready"
    });
    session
}

/// Invokes `function` and answers every host request of the call from
/// `store` until the call ends.
fn serve_host(
    session: &mut Session,
    store: &mut BTreeMap<String, String>,
    operation: &str,
    function: &str,
    params: Value,
) -> Value {
    let request_id = |event: &Value| {
        event["result"]["hostRequestID"]
            .as_str()
            .map(str::to_string)
    };
    let mut answered: BTreeSet<_> = session
        .events
        .iter()
        .filter(|event| event["type"] == "hostRequest")
        .filter_map(request_id)
        .collect();
    session.invoke(operation, function, params);
    let deadline = Instant::now() + super::WAIT;
    loop {
        let pending: Vec<_> = session
            .events
            .iter()
            .filter(|event| {
                event["type"] == "hostRequest"
                    && request_id(event).is_some_and(|id| !answered.contains(&id))
            })
            .cloned()
            .collect();
        for request in pending {
            let id = request_id(&request).expect("host request ID");
            let params = &request["result"]["params"];
            let key = params["key"].as_str().expect("credential key").to_string();
            let result = match request["method"].as_str() {
                Some("lithe/host/credentials/get") => {
                    json!({ "secret": store.get(&key) })
                }
                Some("lithe/host/credentials/set") => {
                    store.insert(key, params["secret"].as_str().expect("secret").to_string());
                    Value::Null
                }
                Some("lithe/host/credentials/delete") => {
                    store.remove(&key);
                    Value::Null
                }
                other => panic!("unexpected host request {other:?}"),
            };
            core(
                "lsp.hostResponse",
                json!({ "sessionId": session.id, "hostRequestID": id, "result": result }),
            );
            answered.insert(id);
        }
        if let Some(completed) = session
            .events
            .iter()
            .find(|event| event["operationId"] == operation && event["type"] == "requestCompleted")
        {
            return completed.clone();
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "timed out waiting for {operation}; events: {:#?}",
            session.events
        );
        let drained = core(
            "lsp.waitEvents",
            json!({ "sessionId": session.id, "timeoutMilliseconds": remaining.as_millis().min(200) as u64 }),
        );
        session
            .events
            .extend(drained["events"].as_array().cloned().unwrap_or_default());
    }
}

/// Waits for the backend to create `path`, draining events meanwhile. The
/// file is the only signal a handler blocked in the backend process gives.
fn wait_for_file(session: &mut Session, path: &Path) {
    let deadline = Instant::now() + super::WAIT;
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "the backend never created {}",
            path.display()
        );
        let drained = core(
            "lsp.waitEvents",
            json!({ "sessionId": session.id, "timeoutMilliseconds": 50 }),
        );
        session
            .events
            .extend(drained["events"].as_array().cloned().unwrap_or_default());
    }
}
