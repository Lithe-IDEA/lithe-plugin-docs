# Lithe plugin backend SDK for Rust

`lithe-plugin-sdk` is the official way to write a
[plugin backend](../../contract/v1/README.md#plugin-backend) in Rust: the one
long-lived process a plugin declares under `backend` in `plugin.json`, which
Lithe starts when one of the plugin's own web pages first calls a function.
The backend is a packaged executable, so a plugin ships one archive per
platform. It speaks the LSP base protocol on stdio and implements, from
[plugin contract v1](../../contract/v1/README.md):

- **`litheFunctions` 1 and 2**: plugin functions with progress,
  cancellation, and registration at runtime. The SDK answers the newest
  version the host offers.
- **Host services** of version 2: the plugin's own credentials, kept by the
  host in the platform's secret store (the Keychain on macOS).

The SDK owns the wire format, negotiation, cancellation, and error mapping.
Your code supplies only handlers. It depends only on `serde_json`, runs
every call on a thread of its own, and needs Rust 1.87 or newer.

Build providers and VCS providers are not part of this SDK yet; write those
with the [Node.js](../node/README.md) or [Java](../java/README.md) SDK.

## A minimal backend

```json
"pluginAPIVersion": 3,
"requires": { "backend": 1, "serverProcess": 1, "ui": 2 },
"permissions": ["backend.launch", "credentials.store", "ui.contribute", "ui.web"],
"backend": { "launch": { "kind": "packaged", "path": "bin/backend" }, "arguments": ["--state", "${stateDir}"] }
```

```rust
use lithe_plugin_sdk::{Backend, PluginError};
use serde_json::json;

fn main() {
    let mut backend = Backend::new("example-backend").version(env!("CARGO_PKG_VERSION"));
    backend
        .function("connection.test", |params, context| {
            context.progress("Connecting", Some(10));
            let key = params["connection"].as_str().ok_or_else(|| {
                PluginError::new("example.invalidParams", "\"connection\" must be a string.")
            })?;
            let password = context.credentials().get(&format!("connection:{key}"))?;
            context.check_cancelled()?;
            Ok(json!({ "hasPassword": password.is_some() }))
        })
        .expect("a valid function ID");
    std::process::exit(backend.listen());
}
```

The page calls it through the host bridge, without an `instance`:

```js
const { value } = await lithe.request('functions.invoke', { function: 'connection.test', params: { connection: 'main' } });
```

[`examples/sample-backend`](examples/sample-backend) is a complete plugin:
the manifest, a web page, notes kept in `${stateDir}` with progress, a
function registered at runtime, a secret in the credential store, and
cancellation. `node scripts/sdk-e2e.mjs` drives it through a real Lithe Core
session with the scenario in
[`sdk/e2e/plugin_sdk/backend.rs`](../e2e/plugin_sdk/backend.rs).

## Getting the SDK

Releases are published on
[this repository's Releases page](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases)
and tagged `sdk-rust-v<version>`. The SDK is not published to crates.io.
Either way below fetches it at build time, never at runtime, and pins its
exact contents:

- **Git dependency.** Name the tag; `Cargo.lock` records the commit, and
  `cargo build --locked` refuses anything else:

  ```toml
  [dependencies]
  lithe-plugin-sdk = { git = "https://github.com/Lithe-IDEA/lithe-plugin-docs", tag = "sdk-rust-v0.1.0" }
  ```

- **Vendored archive.** A release ships `lithe-plugin-sdk-rust-<version>.crate`
  (a gzipped tarball, as `cargo package` makes it) and a `.sha256` file.
  Check the archive against the SHA-256 you pinned, unpack it into your
  repository, and use it as a `path` dependency.

## API

| API | Purpose |
| --- | --- |
| `Backend::new(name).version(version)` | Creates the backend; `name` and `version` are reported as `serverInfo`, and `name` prefixes log lines. |
| `backend.function(id, handler)` | Declares a function offered in `initialize`. |
| `backend.on_initialize(hook)` | Runs before the `initialize` answer with an `Initialize`: the host's `params()`, the negotiated `functions_version()`, `root_path()` (the plugin's `${stateDir}`), and `function(id, handler)` to add functions that depend on state found at startup. An error refuses `initialize`. |
| `backend.on_shutdown(hook)` | Runs on `shutdown`, after running calls were cancelled. |
| `backend.handle()` | A clonable `BackendHandle` for other threads. |
| `backend.listen()` | Serves stdio until `exit` or the end of input and returns the exit code: `0` after `shutdown` and `exit`, `1` otherwise. |
| `backend.serve(input, output)` | The same on other streams, for tests. |
| `handle.register_function(id, handler)`, `handle.unregister_function(id)` | Adds or withdraws a function after `initialize` and waits until the host accepted it. Withdrawing revokes and cancels running calls; a handler must not withdraw its own function. |
| `handle.function_ids()`, `handle.functions_version()` | What the backend serves and negotiated. |
| `handle.credentials()`, `context.credentials()` | The credentials service: `get(key)`, `set(key, secret)`, `delete(key)`. |
| `PluginError::new(code, message)` | A failure meant for your own UI. |

A handler is `Fn(Value, &Context) -> Result<Value, FunctionError>`. Its
context offers:

| Member | Meaning |
| --- | --- |
| `cancellation()` | A `Cancellation`, fired when Lithe cancels the call, its deadline passes, the function is withdrawn, or the session ends: `is_cancelled()`, `check()`, `wait_timeout(duration)` (use it instead of `thread::sleep`), and `on_cancel(callback)`, for example to cancel a running database query from the driver. |
| `check_cancelled()` | `Err(Cancelled)` once cancelled, for `?`. |
| `progress(message, percentage)` | The message is cut to 1024 bytes and the percentage to 100; reports after the call ended are dropped. |
| `credentials()` | Credentials bound to the call: a request still waiting for the host when the call is cancelled ends at once. |
| `backend()`, `function_id()`, `log(message)` | The handle, the function's ID, and a line on stderr. |

## Credentials

A key is 1–128 ASCII letters, digits, `.`, `_`, `:`, or `-`; a secret is at
most 16 KiB. Credentials are private to your plugin ID, survive disabling and
updating the plugin, and are deleted when it is uninstalled. They are never
written to `${stateDir}` or the package.

| Failure (`SdkError::code()`) | Meaning |
| --- | --- |
| `sdk.protocol.notNegotiated` | The host offered only protocol 1, so it has no host services. Nothing was sent. |
| `plugin.host.permissionDenied` | The plugin lacks the `credentials.store` permission. |
| `plugin.host.invalidParams`, `sdk.credentials.invalidKey`, `sdk.credentials.invalidSecret` | A malformed key or secret. |
| `plugin.host.failed` | The host's secret store failed, for example a locked Keychain; the message says why and never contains the secret. |
| `sdk.call.cancelled` | The call was cancelled while waiting; `?` answers as cancelled. |
| `sdk.session.closed` | The session ended while waiting. |

## Results and errors

| Handler outcome | What Lithe sees |
| --- | --- |
| `Ok(value)` (including `Value::Null`) | `result.value` |
| `Err(PluginError)` | `plugin.function.failed` with your message and `pluginErrorCode`. The code is for your plugin's own UI; Lithe never branches on it. |
| Any other error, or a panic | `plugin.function.failed` with a generic message, so paths, secrets, and stack traces never reach the UI. The details go to stderr. A panic ends only its call. |
| `Err(Cancelled)`, or any outcome after cancellation | `requestCancelled` (or Lithe's timeout or revocation outcome) |

`?` converts `PluginError`, `Cancelled`, `SdkError`, `std::io::Error`, and
`serde_json::Error` into `FunctionError`; `FunctionError::internal(details)`
wraps anything else. A result must serialize to at most 1 MiB of JSON. A
function ID is dot-separated lowerCamelCase segments (`snapshot.refresh`), at
most 128 bytes, and a session holds at most 256 functions. A `PluginError`
code matches `[A-Za-z][A-Za-z0-9._-]*`, at most 64 bytes; `PluginError::new`
panics on any other code.

## Rules that keep Lithe in charge

1. **stdout belongs to the protocol.** On Unix, `listen()` moves the
   protocol to a duplicate of the stdout descriptor and points stdout at
   stderr, so a stray `println!` in your code or a dependency lands in the
   host's log. Elsewhere, never print to stdout.
2. **The package is read-only.** Keep caches, databases, and logs under
   `${stateDir}`, which Lithe also passes as the root and working directory.
   Writing to the package breaks code signing and update deltas.
3. **Secrets go to the credentials service.** Never write a password or
   token to `${stateDir}`, to a log line, or to a `PluginError` message.
4. **Work ends with its call.** Check the cancellation in long loops, use
   `wait_timeout` between retries, and register an `on_cancel` callback for
   blocking driver calls. Lithe has already ended the call when it cancels;
   the SDK answers `requestCancelled` whatever the handler returns.
5. **No calls back into Lithe's dispatch.** The SDK never sends
   `lithe/function/invoke` or `lithe/build/*`; Lithe would refuse them as
   `plugin.function.recursiveCall`.

## Testing

```sh
cargo test --locked --manifest-path sdk/rust/Cargo.toml
cargo clippy --locked --manifest-path sdk/rust/Cargo.toml --all-targets -- -D warnings
node scripts/sdk-e2e.mjs
```

The unit tests drive a session through in-process pipes with a scripted
host. The end-to-end check builds the sample, installs it as a package, and
runs it through a real Core session.
