# Lithe plugin backend SDK for Node.js

`@lithe/plugin-sdk` is the official way to write a plugin backend in
JavaScript: the process a plugin starts through a `runtime` launch
(`{ "kind": "runtime", "tool": "node", … }`). It speaks the LSP base protocol
on stdio and implements Lithe's backend protocols from
[plugin contract v1](../../contract/v1/README.md):

- **`litheBuild` 1–6**: typed `lithe/build/*` requests, for a build system
  whose `model` or `dependencies` source is `provider`: models and dependency
  trees (1), bounded task execution (2), tests (3), run (4) and debug (5)
  preparation, and read-only model inputs (6).
- **`litheFunctions` 1–4**: plugin functions, which are plugin-private actions
  that Lithe invokes for the same plugin, with progress, cancellation, and
  registration at runtime (1); host services such as credentials (2);
  workspace contexts, capability discovery, host notifications, and the
  public document, file, editor, and window methods (3); and calls between
  plugins, relayed by Core (4). See [Host services](#host-services) and
  [Calling other plugins](#calling-other-plugins-protocol-4).
- **`litheVcs` 1**: typed `lithe/vcs/*` requests, for a plugin's
  `vcsProviders` entry, from the
  [VCS provider contract](https://github.com/1lck/Lithe-IDEA/blob/preview/shared/contracts/vcs-provider-v1.md).
- **`litheKeymapImport` 1** (from 0.3.0): independently negotiated shortcut
  import providers reading only explicitly selected source snapshots.

The SDK owns the wire format, negotiation, cancellation, process cleanup, and
error mapping. Your code supplies only handlers. Node 20.3 or newer is
required; declare it as the runtime's `minimumVersion`.

## Shortcut import providers

Available from SDK 0.3.0. Declare `backend.keymapImportProvider(id, handler)` before listening. Its
handler receives a snapshot with `providerID`, `snapshotID`, `listInputs()`
and `readChunk(fileID, offset, maximumBytes)`, plus the normal call context.
Paths returned by `listInputs` are relative labels; reads use opaque file IDs.
Return `{ bindings: [...] }` in the normalized contract format and observe the
call's cancellation signal. The host revokes the snapshot when the call ends.
Do not turn conditional or multi-stroke rules into global bindings. Product
discovery and the import workflow are not available yet.

## A minimal backend

```js
import { createBackend, PluginError } from './vendor/lithe-plugin-sdk/src/index.mjs';

const backend = createBackend({ name: 'example-backend', version: '1.0.0' });

backend.build.model(async ({ root }) => ({
  buildSystemID: 'example',
  root,
  projects: [{ name: 'app', path: root }],
}));

backend.function('snapshot.refresh', async (params, context) => {
  context.progress({ message: 'Refreshing', percentage: 10 });
  const result = await context.run('example-tool', ['refresh'], { cwd: params.root });
  if (result.exitCode !== 0) throw new PluginError('example.refreshFailed', 'The refresh failed.');
  return { refreshedAt: Date.now() };
});

backend.listen();
```

[`examples/sample-build`](examples/sample-build) is a complete plugin with a
build provider, functions, persistent state, a function registered at runtime,
and cancellation. `node scripts/sdk-e2e.mjs` drives it through a real Lithe
Core session with the scenario in [`sdk/e2e`](../e2e/plugin_sdk/mod.rs).

## A VCS provider

```js
import { createBackend, PluginError, VCS_ERRORS } from './vendor/lithe-plugin-sdk/src/index.mjs';

createBackend({ name: 'example-vcs', version: '1.0.0' })
  .vcs('example', {
    status: async ({ root, tool }, context) => {
      if (!tool) throw new PluginError(VCS_ERRORS.toolUnavailable, 'The example client was not found.');
      const result = await context.run(tool.executable, ['status', '--json'], { cwd: root.path });
      return { changes: JSON.parse(result.stdout).changes };
    },
    baseContent: ({ path }) => ({ content: readBase(path), binary: false }),
    checkin: ({ paths, message }) => ({ revision: commit(paths, message) }),
    rollback: ({ paths }) => revert(paths),
  })
  .listen();
```

`backend.vcs(id, handlers, traits?)` names the VCS of the manifest's
`vcsProviders` entry and its handlers, keyed by operation: `status`,
`baseContent`, `checkin`, and `rollback` are required; `history`,
`fileHistory`, `annotate`, `update`, `fileOperation`, `resolve`, and
`cleanup` are optional. Each handler you declare negotiates its capability,
so the host offers only what you implement. `traits` describes the VCS
itself: `{ push: true }` when a check-in is local and publishing is a
separate step, `{ staging: true }` when changes have a staged dimension.
`git` is reserved for Lithe.

A handler receives the operation's arguments plus `root: { path }` and
`tool: { executable } | null` (the client the host resolved), and returns the
reply of the contract; returning nothing answers `{}`. Core validates every
reply, sorts paths, and refuses one that leaves the root. Throw
`PluginError` with a `VCS_ERRORS` code (`conflict`, `outOfDate`,
`authenticationFailed`, `workingCopyLocked`, `workingCopyUnsupported`,
`toolUnavailable`) so the host can explain the failure; other codes reach it
as `vcs.provider.failed`. A VCS handler's `context.run` also reports each
finished command (command line, exit code, duration, and at most 16 KiB of
stderr) as `lithe/vcs/execution` for the project console; pass
`{ report: false }` to skip one. Arguments are reported as given, so never
put a credential in them.

[`examples/sample-vcs`](examples/sample-vcs) is a complete provider for a
toy VCS that keeps revisions in a JSON file. Rust Core's
`sdk/e2e/plugin_sdk_node.rs` drives every operation through a
real Core session.

## Getting the SDK

Releases are published on [this repository's Releases page](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases) and
tagged `sdk-node-v<version>`. Version 0.3.0 adds shortcut import providers, native terminals, and typed
diagnostics through `litheLogging` 1, and removes the free-text `log` API.
Version 0.4.0 adds calls between plugins (`litheFunctions` 4):
`host.functions.providers` and `host.functions.invoke`, `context.caller` in the
function that is called, and the `plugin.function.*` relay error codes in
`HOST_ERRORS`. The number-only fallback table of `HostError` is complete: it
now also names the five host error codes -32022 to -32026
(`plugin.host.languageServerUnavailable`, `languageServerError`,
`methodNotDeclared`, `toolUnresolved` and `foreignContribution`).
A release ships
`lithe-plugin-sdk-node-<version>.tgz` and a `.sha256` file holding the
archive's SHA-256. The SDK is not published to the npm registry. Use it like
this:

1. Download the archive during your plugin's build, never at runtime.
2. Check the archive against the SHA-256 you pinned in your repository.
3. Unpack it and copy its `package/` directory into your plugin package, for
   example as `backend/vendor/lithe-plugin-sdk/`, then import
   `src/index.mjs` from there by relative path.

A plugin package is installed read-only, has no `node_modules` install step,
and must run offline, so everything the backend imports has to be inside the
package. The SDK has no dependencies.

## Logging

From version 0.3.0, the SDK requires `experimental.litheLogging` version 1
at initialization and refuses a host that does not offer it, so a backend
built on 0.3.0 or later starts only in a Lithe release that offers it.
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
| `createBackend({ name?, version?, capabilities? })` | Creates the backend. `capabilities` are standard LSP server capabilities; the SDK negotiates its experimental protocols. |
| `backend.build.model(handler)`, `backend.build.dependencies(handler)` | Implements the typed build requests. The SDK stamps `format` and `version`; a handler returning another format or a version other than 1 fails. Core validates the document. `litheBuild` is answered only when a handler exists. |
| `backend.build.execute(handler)` | Protocol 2+: provider-owned bounded task, returning `{ exitCode, output }`. |
| `backend.build.test(handler)`, `backend.build.discoverTests(handler)` | Protocol 3+: exact test execution and runnable selectors; discovery gets `format: "litheTestDiscovery"`, `version: 1`. |
| `backend.build.prepareRun(handler)` | Protocol 4: returns a portable launch plan after upstream synchronization/build; the host owns the application process. |
| `backend.buildVersion` | Highest common negotiated build version (1–6), or `null`; methods newer than that version are refused before their handler runs. |
| `backend.functionsVersion` | Highest common negotiated plugin-function version (1–4), or `null`. |
| `backend.host` | [Host services](#host-services) (`lithe/host/*`), usable once the session is ready. |
| `backend.onCommand(commandID, handler)` | Protocol 3: one of the plugin's commands ran; the handler gets `{ commandID, workspaceContext }`. |
| `backend.onContextsChanged(handler)` | Protocol 3: workspace contexts opened, closed, or changed; the handler gets `{ revision }`. |
| `backend.onHostNotification(method, handler)` | Protocol 3: a `lithe/host/*` notification a later host adds. Unknown ones are ignored. |
| `backend.vcs(id, handlers, traits?)` | Implements the typed VCS requests; see [A VCS provider](#a-vcs-provider). `litheVcs` is answered only when the host offers it. |
| `backend.function(id, handler)` | Declares a function offered in `initialize`. |
| `backend.registerFunction(id, handler)`, `backend.unregisterFunction(id)` | Adds or withdraws a function. Called before the `initialize` answer (for example from `onInitialize`), they change the initial list; later they send `lithe/function/register` / `unregister`. Withdrawing revokes running calls. |
| `backend.onInitialize(hook)`, `backend.onShutdown(hook)` | Lifecycle hooks. `onInitialize` receives the `initialize` params and what was negotiated. |
| `backend.onRequest(method, handler)`, `backend.onNotification(method, handler)` | Ordinary LSP methods, such as `textDocument/hover` for a language server. Lifecycle and `lithe/*` methods are reserved. |
| `backend.sendRequest`, `backend.sendNotification` | Ordinary LSP messages to the host; `lithe/*` methods are reserved. |
| `backend.diagnostics.degraded(operation, code, count?)`, `.recovered(...)` | Typed handled fallback/recovery; no payload text. |
| `backend.listen()` | Serves stdio until `exit` or the end of input. |
| `PluginError(code, message)` | A failure meant for your own UI, or with a `VCS_ERRORS` code, for the host's. |
| `HostError` | The host refused or failed a host request: `code` (`error.data.code`), `rpcCode`, and protocol 3 `details`. `HOST_ERRORS` maps each code, including the `plugin.function.*` codes of the relay, to its number. |
| `runProcess(command, args, options)` | Runs a process without a shell, as one cancellable process tree. |
| `openStateStore(directory)` | A small atomic JSON store in a host-provided directory. |

Every handler receives a context:

| Member | Meaning |
| --- | --- |
| `signal` | Aborted when Lithe cancels the call, its deadline passes, the function is withdrawn, or the session ends; also once the handler has settled, so work it left running stops. |
| `throwIfCancelled()` | Throws the cancellation if `signal` is aborted. |
| `run(command, args, options)` | `runProcess` bound to the call: cancelling or ending the call ends the whole process tree. In VCS handlers it also reports the finished command unless `options.report` is `false`. |
| `diagnostics` | Typed diagnostics correlated with this call. |
| `host` | `backend.host` bound to the call: host requests it left unanswered are cancelled when the call ends. |
| `functionID` | Functions only. |
| `workspaceContext` | Functions only, protocol 3: `{ contextID, generation }` of the page's window, or `null`. |
| `caller` | Functions only, protocol 4: `{ pluginID }` of the plugin whose relayed call this is, or `null` for a call Lithe started, under protocol 3 or earlier, or when the field is not exactly `{ pluginID }`. Core writes it; it is read-only. |
| `operationID` | Functions and bounded build/test/run-preparation calls. Opaque wire identity chosen by Core; never derive anything from it. |
| `progress({ message?, percentage?, output? })` | Functions use messages up to 1024 UTF-8 bytes. Build/test/run preparation use messages up to 4096 bytes and output up to 64 KiB per report. Percentages are clamped to 0–100; cancelled or settled build calls cannot report. Model/dependency/discovery contexts have no progress. |

## Native build workflows

The typed handlers of build protocols 2–6 ship from SDK 0.2.0. They
require the corresponding host capability (`buildProviders` 2/3/4); the manifest
requirement and runtime negotiation are both checked. A version 1 agreement
never permits execute/test/prepareRun, even when those handlers are registered.

See [the native workflow guide](../../guides/native-build-workflows.md) and the
[no-page sample](examples/sample-native-build/backend/main.mjs). Core validates
workspace containment and upstream selections; the SDK does not implement a
second build engine. Build tool failures return their exit code, while a thrown
error fails preparation and cannot start an application.

## Host services

From plugin-function protocol 2 a backend may ask the host for services;
each is granted by a manifest permission and refused otherwise. The SDK
negotiates the highest version both sides offer, so a backend written for
protocol 1 keeps working unchanged.

```js
backend.function('query.run', async (params, context) => {
  // Protocol 2, permission credentials.store: the platform's secret store, per plugin.
  const password = await context.host.credentials.get('connection.prod:password');
  if (password === null) throw new PluginError('query.noPassword', 'Sign in first.');

  // Protocol 3: the window the page runs in.
  const workspace = context.workspaceContext;
  if (workspace && await context.host.supports('lithe/host/workspace/getFolders')) {
    const { folders } = await context.host.workspace.getFolders(workspace);
    // ...
  }
  return runQuery(params, password);
});

backend.onCommand('db.refresh', async ({ workspaceContext }) => {
  if (!workspaceContext) return;
  await backend.host.window.showInformation(workspaceContext, { text: 'Refreshed.' });
});
```

| `backend.host` member | Version | Purpose |
| --- | --- | --- |
| `credentials.get(key)`, `.set(key, secret)`, `.delete(key)` | 2 | Secrets in the platform's store. `get` returns `null` when nothing is stored. A key is 1–128 of `[A-Za-z0-9._:-]`, a secret at most 16 KiB; others fail locally. |
| `capabilities()`, `supports(method)` | 3 | What this session may call, from `lithe/host/getCapabilities` (asked once). Under protocol 2, which has no discovery, `supports` reports the credential methods. |
| `workspace.listContexts()`, `.getContext(context)`, `.getFolders(context)` | 3 | Workspace contexts; only `getContext` returns, and creates, the context's `stateDirectory`. |
| `workspace.readDirectory`, `.stat`, `.readFile`, `.findFiles`, `.findText`, `.watch` | 3 | Files of a local context, addressed as `{ rootID, path }` from `getFolders`. |
| `documents.list`, `.getSnapshot`; `editor.getActive`, `.getSelections`, `.applyEdits`, `.replaceSelections`, `.insertText`, `.open`, `.reveal`, `.registerActions` | 3 | Live documents and editors. Edits name the `expectedVersion` they were computed against. |
| `preview.openHTML`, `browser.openExternal`, `window.showInformation`, `.showWarning`, `.showError` | 3 | Preview, browser, and messages. |
| `functions.providers(functionID)` | 4 | The enabled plugins that export a function: `{ providers: [{ pluginID, version }] }`. See [Calling other plugins](#calling-other-plugins-protocol-4). |
| `functions.invoke(context, call)` | 4 | Calls one provider's exported function in a workspace context: `{ value }`. `call` is `{ pluginID, functionID, params?, timeoutMilliseconds?, parentOperationID? }`. |
| `request(method, params?)`, `subscribe(method, params, onEvent?)` | 2 / 3 | Any `lithe/host/*` method, including ones a later host adds. |
| `subscriptions.dispose(id)` | 3 | Ends a subscription; `Subscription.dispose()` does the same. |

Methods taking a workspace context accept `{ contextID, generation }` or a
whole context from `listContexts`; their params are the contract's, without
`context`. Every method takes `{ signal }` as its last argument. The full
params and results are in
[the contract](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3)
and in `src/index.d.ts`.

What the SDK does for you, and what it leaves to you:

- **Refusals are `HostError`.** Branch on `error.code`, such as
  `plugin.host.contextExpired`; a code this SDK does not know is still a
  `HostError`. On `versionConflict` reread and recompute; on `contextExpired`
  resolve the original target again, never the focused window; retry `busy`
  and `rateLimited` (after `details.retryAfterMilliseconds`) with a bound;
  `permissionDenied` and `readOnly` need the user. From protocol 3 an
  unknown param is `plugin.host.unsupportedOption` naming it in
  `details.option`, so you can retry without it.
- **Cancellation has no rollback.** Aborting a request's signal, or ending
  the call that sent it, rejects it at once; from protocol 3 the SDK also
  sends `$/cancelRequest` so the host stops. An edit or an external open may
  already have taken effect, and the SDK never sends a request again.
- **At most 16 host requests are unanswered.** Further ones wait in the SDK
  for a free slot instead of being refused.
- **Subscriptions route by ID.** `workspace.watch` and `subscribe` call
  `onEvent(event, { subscriptionID, sequence, gap })` in order; `gap` is
  true when events were missed, so rescan. Nothing arrives after `dispose()`
  or the end of the session.
- **Commands are events.** `onCommand` runs when the user runs one of the
  plugin's commands; the host waits for no result. A stopped backend is
  started for it only when the manifest declares `onCommand:<commandID>`.

### Calling other plugins (protocol 4)

A plugin can call a function another enabled plugin exports. Core relays
every call: your backend never reaches the provider's process, and the
provider learns only the call it serves. The rules of the relay are in
[the contract](../../contract/v1/README.md#cross-plugin-calls-protocol-version-4).

- **Declare the call in the manifest.** The caller lists
  `pluginFunctions.calls` (`{ id, version }`, the function and the contract
  version it was written against) and `requires.pluginFunctions: 1`, with
  `backend: 2` and `serverProcess: 1`. The provider lists
  `pluginFunctions.exports` and registers the function with
  `backend.function`. The calls are the consent; there is no permission.
- **Check `supports()` first.** `await backend.host.supports('lithe/host/functions/invoke')`
  is true only in a version 4 session of a plugin that declares a call.
  Under protocol 3 or earlier, `functions.providers`, `functions.invoke`, and
  the same methods through `host.request` fail at once with `HostError`
  `plugin.host.unavailable` (-32601) and send nothing.

```js
backend.function('report.build', async (params, context) => {
  const workspace = context.workspaceContext;
  if (!workspace || !(await context.host.supports('lithe/host/functions/invoke'))) {
    throw new PluginError('report.noRelay', 'This Lithe cannot call other plugins.');
  }
  const { providers } = await context.host.functions.providers('project.describe');
  if (providers.length === 0) throw new PluginError('report.noProvider', 'Install a plugin that describes projects.');
  const { value } = await context.host.functions.invoke(workspace, {
    pluginID: providers[0].pluginID,
    functionID: 'project.describe',
    params: { project: params.project },
    timeoutMilliseconds: 10000,
  });
  return summarize(value);
});
```

- **Results are raw.** `providers` returns `{ providers: [{ pluginID, version }] }`
  and `invoke` returns `{ value }`, where `null` is a result. A call is
  `{ pluginID, functionID, params?, timeoutMilliseconds?, parentOperationID? }`;
  `params` is a JSON object, not sent when omitted (Core reads `{}`).
  `timeoutMilliseconds` is a positive integer. Core defaults it to 60000
  and caps it at 600000 (`LIMITS.relayDefaultTimeoutMilliseconds`,
  `LIMITS.relayMaxTimeoutMilliseconds`); the SDK passes it on as given.
- **Mistakes fail locally, with no frame sent.** A `call` that is not an
  object, or a bad `pluginID`, `functionID`, `params`, `timeoutMilliseconds`
  or `parentOperationID`, rejects with `PluginSdkError`
  `sdk.functions.invalidCall`; a bad workspace context rejects with
  `sdk.host.invalidContext`, as for every other host method. The SDK sends only
  the fields above and drops any other, such as `caller`, which Core writes
  itself and refuses in a request.
- **`parentOperationID` is filled in inside a function.** In a function
  handler, `context.host` sets it to the `operationID` of the call being
  served (when that is a string of 1–128 bytes), so Core knows which call the
  new one belongs to and can cancel it with its parent, detect cycles, and
  bound the chain at 4 calls. An explicit `parentOperationID` wins. Everything
  else has no parent: `backend.host`, host notification callbacks
  (`onCommand`, `onContextsChanged`, `onHostNotification`), build, VCS, and
  shortcut import handlers, plain `onRequest` handlers, and async work left
  running after a call ended. Without a parent Core treats the call as a
  child of every relayed call the backend is serving at that moment. Never
  pass the `operationID` of a call that has finished: Core refuses it with
  `plugin.host.cancelled`.
- **A relayed call fills a host request slot.** It counts among the 16
  unanswered host requests until Core answers, which can take the whole
  timeout, and a caller has at most 8 relayed calls running. Start calls
  together only when they are independent.
- **Read the caller from `context.caller`.** In a provider's function,
  `context.caller.pluginID` is the plugin that called, written by Core and
  impossible to forge; `null` means the host started the call. Never decide by
  a plugin ID in `params`, which any caller can write. Do not accept an
  executable path, command line, or script from a caller and run it: take a
  request you interpret, and choose the executable yourself.

A failed relayed call rejects with a `HostError` whose `code` is one of these
(the `HOST_ERRORS` keys):

| Code | Number | Meaning |
| --- | --- | --- |
| `plugin.function.notDeclared` | -32027 | The manifest does not declare this call. |
| `plugin.function.notExported` | -32028 | The provider does not export the function, or its backend has not registered it. |
| `plugin.function.providerDisabled` | -32029 | The provider is not installed or enabled. |
| `plugin.function.versionMismatch` | -32030 | The declared and exported contract versions differ. |
| `plugin.function.callCycle` | -32065 | The provider is already on the call chain. |
| `plugin.function.depthExceeded` | -32066 | The call chain is longer than 4 calls. |
| `plugin.function.tooManyCalls` | -32067 | The caller already has 8 relayed calls running. |
| `plugin.function.payloadTooLarge` | -32068 | The params, or a result, exceeds 256 KiB. |
| `plugin.function.providerUnavailable` | -32069 | The provider's backend exited, withdrew the function, or did not start in time. |
| `plugin.function.failed` | -32070 | The provider's function answered with an error. |
| `plugin.function.invalidResult` | -32071 | The provider answered without a result, or with one over 1 MiB. |

The host codes `plugin.host.timeout`, `plugin.host.cancelled`,
`plugin.host.contextExpired`, `plugin.host.authorityUnsupported`,
`plugin.host.permissionDenied` and `plugin.host.invalidParams` also occur. On
`plugin.function.failed`, `error.message` is the provider's message and
`error.details.pluginErrorCode` is the code the provider's function threw;
only branch on a code its provider documents.

## Results and errors

The following mapping describes private functions. Typed build operations use
their domain result/error contract; see the native workflow guide above.

| Handler outcome | What Lithe sees |
| --- | --- |
| Returns a value (including `null`) | `result.value` |
| Throws `PluginError(code, message)` | `plugin.function.failed` with your `message` and `pluginErrorCode`. The code is for your plugin's own UI; Lithe never branches on it. |
| Throws anything else | `plugin.function.failed` with a generic message, so paths and stack traces never reach the UI. Automatic diagnostics record only a fixed failure category. |
| Ends because of `signal` | `requestCancelled` (or Lithe's timeout/revocation outcome) |

A result must serialize to at most 1 MiB of JSON. A value JSON cannot encode
(a `BigInt`, a cycle) or a larger one fails like any other unexpected error and
emits one typed failure fact; the session keeps running. A function ID is dot-separated
lowerCamelCase segments (`snapshot.refresh`), at most 128 bytes, and a session
holds at most 256 functions. A `PluginError` code matches
`[A-Za-z][A-Za-z0-9._-]*`, at most 64 bytes.

## Rules that keep Lithe in charge

The SDK leaves your plugin free to define its own functions, params, results,
UI, and state, but these rules keep Lithe's workflows, permissions, and
lifecycle unified. The SDK enforces the ones it can.

1. **Domain capabilities stay typed.** Build models and dependency trees go
   through `backend.build.*`, and version control through `backend.vcs`,
   never through a function, so every host and IDE workflow reads one
   contract. Functions are for actions only your plugin understands.
2. **No calls back into Lithe's dispatch.** A backend never sends
   `lithe/function/invoke` or `lithe/build/*`; Lithe would route it back to the
   same process, which would then wait for itself. Lithe refuses these with
   `plugin.function.recursiveCall`, and `sendRequest` refuses every `lithe/*`
   method. Host services go through `backend.host`, which sends only
   `lithe/host/*`. The only `lithe/vcs/*` message a provider sends is the
   execution report, which `context.run` writes for you.
3. **stdout belongs to the protocol.** When serving stdio, the SDK moves
   `console.log`, `console.info`, and `console.debug` to stderr. Never write to
   `process.stdout` yourself.
4. **The package is read-only.** Keep caches and snapshots under the
   directories Lithe passes through launch templates (`${stateDir}`,
   `${workspaceStateDir}`). `openStateStore` refuses any directory inside the
   plugin package. Diagnostics use the SDK and host; native log files require
   an explicitly supplied `${logDir}` on a compatible local host. Writing to
   the package breaks code signing and update deltas.
5. **Work ends with its call.** Start processes with `context.run`, and check
   `signal` in long loops. On cancellation the SDK stops the process tree
   politely, then forcibly. When the handler returns or throws, tools of that
   call still running (an unawaited `context.run`, the siblings of a failed
   `Promise.all`) are stopped the same way. A tool that exits on its own takes
   the workers it left in its process group with it. On `shutdown`, `exit`, or
   the end of input the SDK ends every process it started. If the backend
   itself is killed, no JavaScript runs; ending what it left is the host's
   job, not the SDK's. Lithe finds those processes by the markers it puts in
   the backend's environment, so `runProcess` carries them over even when
   `env` replaces the environment.
6. **Errors stay presentable.** Throw `PluginError` for failures the user
   should see; every other exception is reported generically.

## Testing

```bash
npm test --prefix sdk/node

# Development-only TypeScript compiler; no production dependencies are added.
npm ci --prefix sdk/node --ignore-scripts
npm run --prefix sdk/node test:types
```

The tests use a bounded in-memory host (`test/harness.mjs`), which plugin
authors can copy to test their own backends without starting Lithe.


### Read-only build model inputs

Build protocol 6 accepts the host's optional `params.modelInputs` snapshot.
A consumer declares exact `modelInputs` producer slots and requires
`buildProviders: 6` plus `buildModelInputs: 1`. A producer opts into
`modelSharing` and selects any ordinary build option values it wants to share.
The host supplies normalized projects, targets and content roots; tasks, run
plans, settings and unselected option values are excluded.

Treat `revision` as an opaque workspace snapshot identity. Recompute cached
work when it changes. A slot can be unavailable, and an instance can be loading,
stale or failed; only ready instances carry facts. Do not invoke another plugin
or install/activate a missing producer. Input sharing grants no process or
private-function authority. Version 1 producers cannot consume and re-export
inputs. Core validates the shape and the host revokes in-flight work when its
producer changes. Protocol 1–5 handlers reject requests containing model inputs.

The two native workflow samples demonstrate independent producer and consumer
backends in the Core end-to-end tests. Their engine is fictional; this does not
migrate Maven, JDT, LLDB or existing plugin business logic.

## Native terminal API

From version 0.3.0, the SDK includes wrappers for plugin-owned native terminal create/show/close,
reopen with bounded display history, disposal and exit subscriptions. These
methods are discovered within protocol 3; they require a newer host and
`terminal.create`, not a new protocol version. See
[the native terminal guide](../../guides/native-terminals.md).
