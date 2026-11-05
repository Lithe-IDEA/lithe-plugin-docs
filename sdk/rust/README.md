# Lithe plugin backend SDK for Rust

`lithe-plugin-sdk` is the official way to write a
[plugin backend](../../contract/v1/README.md#plugin-backend) in Rust: the one
long-lived process a plugin declares under `backend` in `plugin.json`, which
Lithe starts when one of the plugin's own web pages first calls a function.
The backend is a packaged executable, so a plugin ships one archive per
platform. It speaks the LSP base protocol on stdio and implements, from
[plugin contract v1](../../contract/v1/README.md):

- **`litheFunctions` 1 to 4**: plugin functions with progress,
  cancellation, and registration at runtime. The SDK answers the newest
  version the host offers.
- **Host services** of versions 2 to 4: the plugin's own credentials, kept
  by the host in the platform's secret store (the Keychain on macOS), from
  version 3 workspace contexts, files, documents, editors, preview,
  the browser, window messages, commands, and subscriptions, and from
  version 4 [calls to the functions other plugins export](#calling-other-plugins),
  relayed by Core, with the `Caller` a function called that way sees.
- **`litheKeymapImport` 1** (from 0.2.0): independently negotiated shortcut
  import providers reading only explicitly selected source snapshots.

The SDK owns the wire format, negotiation, cancellation, and error mapping.
Your code supplies only handlers. It depends on `serde_json` and `base64`, runs
every call on a thread of its own, and needs Rust 1.87 or newer.

Build providers and VCS providers are not part of this SDK yet; write build
providers with the [Node.js](../node/README.md) or [Java](../java/README.md)
SDK, and VCS providers with the Node.js SDK.

## Shortcut import providers

Available from SDK 0.2.0. Declare `keymap_import_provider(id, handler)` before listening. The host
negotiates IDs independently of plugin functions; a handler receives a
`KeymapImportRequest` and a call-scoped `Context`. Use
`request.list_inputs(context.host())` and `request.read_chunk(context.host(),
file_id, offset, maximum_bytes)` to read declared files in bounded chunks.
`KeymapSourceFile.path` is a relative label; the file ID is the authority.
Return `{ "bindings": [...] }` in the normalized contract format. The host
cancels the call on import cancellation and revokes the snapshot when it ends.
Do not convert conditional or multi-stroke rules into global bindings. Product
discovery and the import workflow are not available yet.

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
function registered at runtime, a secret in the credential store,
cancellation, a watch on the page's workspace, a command answered with a
message in its window, and a handler for workspace changes.
`node scripts/sdk-e2e.mjs` drives it through a real Lithe Core session with
the scenario in
[`sdk/e2e/plugin_sdk/backend.rs`](../e2e/plugin_sdk/backend.rs).

## Getting the SDK

Releases are published on
[this repository's Releases page](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases)
and tagged `sdk-rust-v<version>`. Version 0.2.0 adds shortcut import providers, native terminals, and typed
diagnostics through `litheLogging` 1, and removes the free-text `log` API.
Version 0.3.0 adds plugin-function protocol 4: `host().functions()` to list
and call the functions other plugins export, `Context::caller()` with the
`Caller` type, and the `Functions` and `RELAY_ERROR_CODES` exports. Under a
host that offers protocol 3 or older the two relay methods are refused
locally and nothing is sent. The table that gives a refusal carrying only a
JSON-RPC number its code now covers the 5 host error codes -32022 to -32026
(`HOST_ERROR_CODES` has 21 entries) and the 11 `plugin.function.*` codes of
`RELAY_ERROR_CODES`.
The SDK is not published to crates.io.
Either way below fetches it at build time, never at runtime, and pins its
exact contents:

- **Git dependency.** Name the tag; `Cargo.lock` records the commit, and
  `cargo build --locked` refuses anything else:

  ```toml
  [dependencies]
  lithe-plugin-sdk = { git = "https://github.com/Lithe-IDEA/lithe-plugin-docs", tag = "sdk-rust-v0.3.0" }
  ```

- **Vendored archive.** A release ships `lithe-plugin-sdk-rust-<version>.crate`
  (a gzipped tarball, as `cargo package` makes it) and a `.sha256` file.
  Check the archive against the SHA-256 you pinned, unpack it into your
  repository, and use it as a `path` dependency.

## Logging

From version 0.2.0, the SDK requires `experimental.litheLogging` version 1
at initialization and refuses a host that does not offer it, so a backend
built on 0.2.0 or later starts only in a Lithe release that offers it.
See [plugin logging](../../guides/plugin-logging.md) for the typed diagnostics
API, host-owned catalog, exact levels, operation correlation and `${logDir}`.
Use `diagnostics.degraded/recovered` (Rust: `diagnostics().degraded/recovered`)
for handled fallback or recovery facts. Function and build/VCS terminal facts
are automatic; do not log a failure before returning it to the SDK. Suppress
successful high-frequency reads with `suppressSuccessDiagnostics(operation)`
(Rust: `suppress_success_diagnostics`). The old free-text `log` API is removed;
there is no stderr fallback or plugin-owned file writer.

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
| `backend.on_command(id, handler)` | Runs `Fn(&str, Option<WorkspaceContext>)` when the user runs one of the plugin's commands (protocol 3). |
| `backend.on_contexts_changed(handler)`, `backend.on_host_notification(method, handler)` | Handle `lithe/host/workspace/didChangeContexts` and any other `lithe/host/*` notification a later host adds. |
| `handle.host()`, `context.host()` | The [host services](#host-services). `handle.host()` is bound to no call. |
| `handle.credentials()`, `context.credentials()` | Shorthand for `host().credentials()`. |
| `PluginError::new(code, message)` | A failure meant for your own UI. |

A handler is `Fn(Value, &Context) -> Result<Value, FunctionError>`. Its
context offers:

| Member | Meaning |
| --- | --- |
| `cancellation()` | A `Cancellation`, fired when Lithe cancels the call, its deadline passes, the function is withdrawn, or the session ends: `is_cancelled()`, `check()`, `wait_timeout(duration)` (use it instead of `thread::sleep`), and `on_cancel(callback)`, for example to cancel a running database query from the driver. |
| `check_cancelled()` | `Err(Cancelled)` once cancelled, for `?`. |
| `progress(message, percentage)` | The message is cut to 1024 bytes and the percentage to 100; reports after the call ended are dropped. |
| `host()` | Host services bound to the call: a request still waiting for the host when the call is cancelled ends at once. `credentials()` is `host().credentials()`. |
| `workspace_context()` | From protocol 3, the `WorkspaceContext` of the window whose page made the call; `None` under older protocols or when no window made it. |
| `caller()` | From protocol 4, the `Caller` (`plugin_id()`) of the plugin whose backend called this function through Core's relay; `None` when the host started the call, under older protocols, and when the host's `caller` is not exactly `{ pluginID }` with a non-empty string. |
| `backend()`, `function_id()`, `diagnostics()` | The handle, the function's ID, and typed diagnostics correlated with this call. |

## Host services

From plugin-function protocol 2 a backend may ask the host for services;
each is granted by a manifest permission and refused otherwise. The SDK
negotiates the highest version both sides offer, so a backend written for
protocol 1 keeps working unchanged. Every method blocks its thread until the
host answers and returns `Result<_, SdkError>`; wrong arguments fail locally
and send nothing.

```rust
backend
    .function("query.run", |params, context| {
        // Protocol 2, permission credentials.store: the platform's secret store, per plugin.
        let password = context
            .host()
            .credentials()
            .get("connection.prod:password")?
            .ok_or_else(|| PluginError::new("query.noPassword", "Sign in first."))?;
        // Protocol 3: the window the page runs in.
        let mut folders = None;
        if let Some(workspace) = context.workspace_context() {
            if context.host().supports("lithe/host/workspace/getFolders")? {
                folders = Some(context.host().workspace().get_folders(workspace)?);
            }
        }
        run_query(&params, &password, folders)
    })
    .expect("a valid function ID");

let host = backend.handle().host();
backend
    .on_command("db.refresh", move |_command_id, workspace| {
        if let Some(workspace) = workspace {
            host.window()
                .show_information(&workspace, json!({ "text": "Refreshed." }))?;
        }
        Ok(())
    })
    .expect("a valid command ID");
```

| `Host` member | Version | Purpose |
| --- | --- | --- |
| `credentials().get(key)`, `.set(key, secret)`, `.delete(key)` | 2 | Secrets in the platform's store; see [Credentials](#credentials). |
| `capabilities()`, `supports(method)` | 3 | What this session may call, from `lithe/host/getCapabilities` (asked once). Under protocol 2, which has no discovery, `supports` reports the credential methods. |
| `workspace().list_contexts()`, `.get_context(&context)`, `.get_folders(&context)` | 3 | Workspace contexts; only `get_context` returns, and creates, the context's `stateDirectory`. |
| `workspace().read_directory`, `.stat`, `.read_file`, `.find_files`, `.find_text`, `.watch` | 3 | Files of a local context, addressed as `{ rootID, path }` from `get_folders`. |
| `documents().list`, `.get_snapshot`; `editor().get_active`, `.get_selections`, `.apply_edits`, `.replace_selections`, `.insert_text`, `.open`, `.reveal`, `.register_actions` | 3 | Live documents and editors. Edits name the `expectedVersion` they were computed against. |
| `preview().open_html`, `browser().open_external`, `window().show_information`, `.show_warning`, `.show_error` | 3 | Preview, browser, and messages. |
| `functions().providers(function_id)`, `.invoke(&context, call)` | 4 | The plugins that export a function, and a call to one of them; see [Calling other plugins](#calling-other-plugins). |
| `request(method, params)`, `subscribe(method, params)` | 2 / 3 | Any `lithe/host/*` method, including ones a later host adds. Under protocol 3 or older the two relay methods are refused locally. |
| `subscriptions().dispose(id)` | 3 | Ends a subscription; `Subscription::dispose()` does the same. |
| `with_cancellation(&cancellation)` | 2 | The same services, cancelled together with `cancellation`. |
| `version()` | | The negotiated plugin-function version; `None` before `initialize`. |

Methods taking a workspace context take a `&WorkspaceContext`: one from
`context.workspace_context()`, `WorkspaceContext::new(id, generation)`, or
`WorkspaceContext::from_value` on a whole context from `list_contexts`. Their
`params` are the contract's, without `context`, and they return the
contract's result as a `Value`. The full params and results are in
[the contract](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3).

What the SDK does for you, and what it leaves to you:

- **Refusals keep the host's code.** Branch on `code()`, such as
  `plugin.host.contextExpired`; `rpc_code()` is the JSON-RPC number and
  `details()` the host's object, if any. A code this SDK does not know is
  kept as sent. On `versionConflict` reread and recompute; on
  `contextExpired` resolve the original target again, never the focused
  window; retry `busy` and `rateLimited` (after the `retryAfterMilliseconds` detail) with a
  bound; `permissionDenied` and `readOnly` need the user. From protocol 3 an
  unknown param is `plugin.host.unsupportedOption` naming it in the
  `option` detail, so you can retry without it.
- **Cancellation has no rollback.** Cancelling the token given to
  `with_cancellation`, or ending the call whose `context.host()` sent a
  request, fails it at once with `sdk.call.cancelled`; from protocol 3 the
  SDK also sends `$/cancelRequest` so the host stops. An edit or an external
  open may already have taken effect, and the SDK never sends a request
  again.
- **At most 16 host requests are unanswered.** Further ones wait in the SDK
  for a free slot instead of being refused.
- **Subscriptions route by ID.** `workspace().watch`,
  `editor().register_actions`, and `subscribe` return a `Subscription`;
  `recv()` and `recv_timeout(duration)` give each `Event` in order, with
  `sequence()`, `event()`, and `gap()`, which is true when events were
  missed, so rescan. Events the host sends before its answer are kept. No
  event is delivered after `dispose()` or the end of the session; once the
  queued ones are read, `recv()` returns `None`.
- **Commands are events.** `on_command` runs on a thread of its own when the
  user runs one of the plugin's commands, with the window's
  `WorkspaceContext` when the host sent one. The host waits for no result,
  and a failure is only logged. A stopped backend is started for it only
  when the manifest declares `onCommand:<commandID>`. Host notifications
  without a handler are ignored.

### Credentials

A key is 1–128 ASCII letters, digits, `.`, `_`, `:`, or `-`; a secret is at
most 16 KiB. Credentials are private to your plugin ID, survive disabling and
updating the plugin, and are deleted when it is uninstalled. They are never
written to `${stateDir}` or the package. `get` returns `Ok(None)` when
nothing is stored.

| Failure (`SdkError::code()`) | Meaning |
| --- | --- |
| `plugin.host.unavailable` | The host offered only protocol 1, so it has no host services. Nothing was sent. |
| `plugin.host.permissionDenied` | The plugin lacks the `credentials.store` permission. |
| `plugin.host.invalidParams`, `sdk.credentials.invalidKey`, `sdk.credentials.invalidSecret` | A malformed key or secret. |
| `plugin.host.failed` | The host's secret store failed, for example a locked Keychain; the message says why and never contains the secret. |
| `sdk.call.cancelled` | The call was cancelled while waiting; `?` answers as cancelled. |
| `sdk.session.notReady`, `sdk.session.closed` | The session has not finished `initialize`, or it ended while waiting. |

### Calling other plugins

From plugin-function protocol 4 a backend may call a function another plugin
exports; Core relays the call and the provider's backend never learns more
than the call it serves. The manifest declares both sides: the caller lists
`{ id, version }` under `pluginFunctions.calls`, and the provider lists
`{ id, version, context: "workspace" }` under `pluginFunctions.exports` and
registers the function with `backend.function(id, handler)`. See
[cross-plugin calls](../../contract/v1/README.md#cross-plugin-calls-protocol-version-4)
for the manifest, the order of Core's checks, and the call chain.

```rust
backend
    .function("project.options", |_params, context| {
        let workspace = context
            .workspace_context()
            .ok_or_else(|| PluginError::new("example.noWorkspace", "Open a folder first."))?;
        let host = context.host();
        if !host.supports("lithe/host/functions/invoke")? {
            return Ok(json!({ "options": [] }));
        }
        let providers = host.functions().providers("project.describe")?;
        let Some(provider) = providers["providers"][0]["pluginID"].as_str() else {
            return Ok(json!({ "options": [] }));
        };
        let answer = host.functions().invoke(
            workspace,
            json!({
                "pluginID": provider,
                "functionID": "project.describe",
                "params": { "project": "app" },
                "timeoutMilliseconds": 5000,
            }),
        )?;
        Ok(json!({ "options": answer["value"] }))
    })
    .expect("a valid function ID");

backend
    .function("project.describe", |params, context| {
        // Core wrote the caller from the calling session; it cannot be forged.
        let caller = context.caller().map(|caller| caller.plugin_id());
        Ok(json!({ "project": params["project"], "askedBy": caller }))
    })
    .expect("a valid function ID");
```

- **Check before calling.** `supports("lithe/host/functions/invoke")` and
  `supports("lithe/host/functions/providers")` tell whether this session may
  call. Protocol 4 alone is not enough: a plugin that declares no `calls` gets
  `plugin.host.permissionDenied`. Under protocol 3 or older, `providers`,
  `invoke`, and `host().request` with either method fail locally with
  `plugin.host.unavailable` and send nothing.
- **`providers(function_id)`** answers `{ "providers": [{ "pluginID",
  "version" }] }`: the enabled plugins other than this one that export the
  function, sorted by plugin ID. This plugin must have declared the function,
  or the host answers `plugin.function.notDeclared`. A provider's backend need
  not be running.
- **`invoke(&context, call)`** answers `{ "value": ... }`, where `value` is the
  provider's result and may be `null`. `context` is a `WorkspaceContext` this
  backend learned from the workspace service or `context.workspace_context()`;
  the relay accepts local workspaces only. `call` is an object with
  `pluginID` and `functionID`, and optionally `params` (an object, `{}` by
  default), `timeoutMilliseconds` (a positive integer; the host defaults it to
  60000 and caps it at 600000, and the SDK does not clamp it), and
  `parentOperationID`. Any other key, such as `context` or `caller`, is not
  sent; the host would refuse it as `plugin.host.unsupportedOption`. A call
  that is not an object, lacks `pluginID` or `functionID`, or has a field of
  the wrong type fails locally with `sdk.functions.invalidCall`, and a
  `function_id` that is not a function ID does the same for `providers`.
- **Limits are the host's.** A chain has at most 4 calls and does not return
  to a plugin already on it (`plugin.function.callCycle`); a caller has at
  most 8 relayed calls running; params and a result are at most 256 KiB. A
  provider that is asked to apply values must not call the plugin that asked
  inside that call.
- **`parentOperationID` is filled in for you inside a handler.** A call made
  with `context.host().functions().invoke(...)` while a function runs names
  that call's `operationID`, so Core makes it a child: cancelling or ending
  the handler's call cancels the relayed call. An explicit
  `parentOperationID` in `call` overrides it (1 to 128 bytes). A
  `handle.host()` serves no call and a generic `request` is sent as written,
  so neither fills it in. An ID that names a call this backend is no longer
  serving fails with `plugin.host.cancelled`.
- **A relayed call occupies a request slot.** It counts among the 16
  unanswered host requests until the provider answers, which can take up to
  600 seconds; further requests wait in the SDK. Keep relayed calls bounded
  with `timeoutMilliseconds`.
- **`context.caller()` names the calling plugin.** It is `Some(Caller)` only
  for a call Core relayed under protocol 4, and `None` for a call the host
  started, which includes every call before protocol 4. The SDK accepts only
  a `caller` that is exactly `{ "pluginID": "<non-empty string>" }`; any other
  shape is read as no caller. It names who called, not what they may do:
  authorize with the plugin's own rules.

| Failure (`SdkError::code()`) | Meaning |
| --- | --- |
| `sdk.functions.invalidCall` | The `call` or the function ID was malformed. Nothing was sent. |
| `plugin.host.unavailable` | The host negotiated protocol 3 or older. Nothing was sent. |
| `plugin.host.permissionDenied` | The plugin declares no `pluginFunctions.calls`. |
| `plugin.function.notDeclared`, `.providerDisabled`, `.notExported`, `.versionMismatch` | The call is not declared, the provider is missing or disabled, does not export the function now, or exports another version. |
| `plugin.function.callCycle`, `.depthExceeded`, `.tooManyCalls`, `.payloadTooLarge` | The chain, the caller's running calls, or a payload exceeded a limit. |
| `plugin.function.providerUnavailable`, `.invalidResult` | The provider's backend stopped or did not start in time, or answered with no result or one over 1 MiB. |
| `plugin.function.failed` | The provider's function returned an error. The `pluginErrorCode` entry of `details()` is the provider's own `PluginError` code, and `message()` is its message. |
| `plugin.host.contextExpired`, `plugin.host.authorityUnsupported`, `plugin.host.timeout`, `plugin.host.cancelled` | The context was closed or replaced, is not a local workspace, the deadline passed, or the call (or its parent) was cancelled. |
| `sdk.host.invalidResult` | The host answered with something other than an object. |

## Results and errors

| Handler outcome | What Lithe sees |
| --- | --- |
| `Ok(value)` (including `Value::Null`) | `result.value` |
| `Err(PluginError)` | `plugin.function.failed` with your message and `pluginErrorCode`. The code is for your plugin's own UI; Lithe never branches on it. |
| Any other error, or a panic | `plugin.function.failed` with a generic message, so paths, secrets, and stack traces never reach the UI. Automatic SDK diagnostics record only a fixed failure category. A panic ends only its call; SDK catch boundaries suppress raw payload printing. |
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
2. **The package is read-only.** Keep caches and databases under `${stateDir}`,
   which Lithe also passes as the root and working directory. Diagnostics use
   the SDK and host; native log files require an explicitly supplied `${logDir}`
   on a compatible local host. Writing to the package breaks code signing and
   update deltas.
3. **Secrets go to the credentials service.** Never write a password or
   token to `${stateDir}`, to a log line, or to a `PluginError` message.
4. **Work ends with its call.** Check the cancellation in long loops, use
   `wait_timeout` between retries, and register an `on_cancel` callback for
   blocking driver calls. Lithe has already ended the call when it cancels;
   the SDK answers `requestCancelled` whatever the handler returns.
5. **No calls back into Lithe's dispatch.** The SDK never sends
   `lithe/function/invoke` or `lithe/build/*`; Lithe would refuse them as
   `plugin.function.recursiveCall`. `Host::request` sends only
   `lithe/host/*` and refuses anything else as `sdk.host.method`. To call
   another plugin, use [`host().functions()`](#calling-other-plugins), which
   sends `lithe/host/functions/invoke` and lets Core relay it.

## Testing

```sh
cargo test --locked --manifest-path sdk/rust/Cargo.toml
cargo fmt --manifest-path sdk/rust/Cargo.toml --all -- --check
cargo clippy --locked --manifest-path sdk/rust/Cargo.toml --all-targets -- -D warnings
node scripts/sdk-e2e.mjs
```

The unit tests drive a session through in-process pipes with a scripted
host; `tests/host_services.rs` checks the host services against the
contract's fixture. The end-to-end check builds the sample, installs it as a
package, and runs it through a real Core session.

## Native terminal API

From version 0.2.0, the SDK includes wrappers for plugin-owned native terminal create/show/close,
reopen with bounded display history, disposal and exit subscriptions. These
methods are discovered within protocol 3; they require a newer host and
`terminal.create`, not a new protocol version. See
[the native terminal guide](../../guides/native-terminals.md).
