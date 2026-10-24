//! Sample plugin backend for the Rust SDK: the `backend` process of the
//! sample plugin in this directory. Lithe starts it when the plugin's web
//! page first calls a function, with `--state ${stateDir}`. It:
//!
//! - keeps notes in its state directory, with progress (`note.save`,
//!   `note.read`), and registers `note.clear` once there is a note to clear;
//! - keeps a secret in the host's credential store, never in its state
//!   (`secret.remember`, `secret.check`, `secret.forget`);
//! - waits until it is cancelled (`work.wait`), and fails unexpectedly
//!   without leaking details to the page (`fail.unexpectedly`).

use lithe_plugin_sdk::{
    sdk_codes, BackendHandle, Cancelled, Context, FunctionError, PluginError, SdkError, Value,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let state = match state_directory() {
        Ok(state) => Arc::new(state),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let mut backend =
        lithe_plugin_sdk::Backend::new("sample-backend").version(env!("CARGO_PKG_VERSION"));

    let notes = Arc::clone(&state);
    backend
        .function("note.save", move |params, context| {
            let name = note_name(&params)?;
            let text = params["text"].as_str().ok_or_else(|| {
                PluginError::new("sample.invalidParams", "\"text\" must be a string.")
            })?;
            context.progress("Saving", Some(0));
            let directory = notes.join("notes");
            fs::create_dir_all(&directory)?;
            // Written beside the note and renamed, so a reader never sees half a note.
            let temporary = directory.join(format!(".{name}.tmp"));
            fs::write(&temporary, text)?;
            fs::rename(&temporary, directory.join(format!("{name}.txt")))?;
            context.progress("Saved", Some(100));
            offer_clear(&context.backend(), &notes);
            Ok(json!({ "name": name, "bytes": text.len() }))
        })
        .expect("valid function");

    let notes = Arc::clone(&state);
    backend
        .function("note.read", move |params, _| {
            let name = note_name(&params)?;
            match fs::read_to_string(notes.join("notes").join(format!("{name}.txt"))) {
                Ok(text) => Ok(json!({ "text": text })),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    Err(PluginError::new(
                        "sample.noteMissing",
                        format!("There is no note named {name}."),
                    )
                    .into())
                }
                Err(error) => Err(error.into()),
            }
        })
        .expect("valid function");

    backend
        .function("secret.remember", |params, context| {
            let (key, secret) = (credential_key(&params)?, secret(&params)?);
            context
                .credentials()
                .set(key, secret)
                .map_err(credentials_failure)?;
            Ok(json!({}))
        })
        .expect("valid function");

    backend
        .function("secret.check", |params, context| {
            let (key, secret) = (credential_key(&params)?, secret(&params)?);
            let stored = context
                .credentials()
                .get(key)
                .map_err(credentials_failure)?;
            // The page learns whether its input matches, never the secret.
            Ok(json!({ "stored": stored.is_some(), "matches": stored.as_deref() == Some(secret) }))
        })
        .expect("valid function");

    backend
        .function("secret.forget", |params, context| {
            let key = credential_key(&params)?;
            context
                .credentials()
                .delete(key)
                .map_err(credentials_failure)?;
            Ok(Value::Null)
        })
        .expect("valid function");

    let markers = Arc::clone(&state);
    backend
        .function("work.wait", move |_, context| {
            fs::write(markers.join("waiting"), "")?;
            // Blocks without spinning until the host cancels the call.
            if context
                .cancellation()
                .wait_timeout(Duration::from_secs(600))
            {
                fs::write(markers.join("cancelled"), "")?;
                return Err(Cancelled.into());
            }
            Ok(json!("finished"))
        })
        .expect("valid function");

    let hidden = Arc::clone(&state);
    backend
        .function("fail.unexpectedly", move |_, _| {
            Err(FunctionError::internal(format!(
                "could not open {}",
                hidden.join("missing").display()
            )))
        })
        .expect("valid function");

    let notes = Arc::clone(&state);
    backend.on_initialize(move |initialize| {
        if has_notes(&notes) {
            let notes = Arc::clone(&notes);
            initialize.function("note.clear", move |_, context| clear(&notes, context))?;
        }
        Ok(())
    });

    std::process::exit(backend.listen());
}

fn state_directory() -> Result<PathBuf, String> {
    let mut arguments = std::env::args().skip(1);
    match (arguments.next().as_deref(), arguments.next()) {
        (Some("--state"), Some(path)) if arguments.next().is_none() => Ok(PathBuf::from(path)),
        _ => Err("usage: sample-backend --state <directory>".into()),
    }
}

/// Registers `note.clear` the first time there is a note to clear.
fn offer_clear(backend: &BackendHandle, state: &Arc<PathBuf>) {
    if backend.function_ids().iter().any(|id| id == "note.clear") {
        return;
    }
    let notes = Arc::clone(state);
    if let Err(error) =
        backend.register_function("note.clear", move |_, context| clear(&notes, context))
    {
        backend.log(format_args!("note.clear stays unavailable: {error}"));
    }
}

fn clear(state: &Path, context: &Context) -> Result<Value, FunctionError> {
    context.check_cancelled()?;
    match fs::remove_dir_all(state.join("notes")) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(Value::Null),
    }
}

fn has_notes(state: &Path) -> bool {
    fs::read_dir(state.join("notes")).is_ok_and(|mut entries| entries.next().is_some())
}

/// A note name is a file name the sample chooses the alphabet of, so a page
/// can never name a path outside the notes directory.
fn note_name(params: &Value) -> Result<&str, PluginError> {
    params["name"]
        .as_str()
        .filter(|name| {
            (1..=64).contains(&name.len())
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
        .ok_or_else(|| {
            PluginError::new(
                "sample.invalidParams",
                "\"name\" must be 1-64 lowercase letters, digits, or \"-\".",
            )
        })
}

fn credential_key(params: &Value) -> Result<&str, PluginError> {
    params["key"]
        .as_str()
        .filter(|key| lithe_plugin_sdk::is_credential_key(key))
        .ok_or_else(|| PluginError::new("sample.invalidParams", "\"key\" is not a credential key."))
}

fn secret(params: &Value) -> Result<&str, PluginError> {
    params["secret"]
        .as_str()
        .ok_or_else(|| PluginError::new("sample.invalidParams", "\"secret\" must be a string."))
}

/// A host that offers no credential store, or a plugin installed without the
/// `credentials.store` permission, is a state the page can explain; other
/// failures stay internal.
fn credentials_failure(error: SdkError) -> FunctionError {
    if error.code() == sdk_codes::NOT_NEGOTIATED || error.code().starts_with("plugin.host.") {
        PluginError::new(
            "sample.credentialsUnavailable",
            "Lithe cannot store credentials for this plugin.",
        )
        .into()
    } else {
        error.into()
    }
}
