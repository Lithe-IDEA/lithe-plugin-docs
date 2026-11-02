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
- **`litheFunctions` 1–3**: plugin functions, which are plugin-private actions
  that Lithe invokes for the same plugin, with progress, cancellation, and
  registration at runtime (1); host services such as credentials (2); and
  workspace contexts, capability discovery, host notifications, and the
  public document, file, editor, and window methods (3). See
  [Host services](#host-services).
- **`litheVcs` 1**: typed `lithe/vcs/*` requests, for a plugin's
  `vcsProviders` entry, from the
  [VCS provider contract](https://github.com/1lck/Lithe-IDEA/blob/preview/shared/contracts/vcs-provider-v1.md).

The SDK owns the wire format, negotiation, cancellation, process cleanup, and
error mapping. Your code supplies only handlers. Node 20.3 or newer is
required; declare it as the runtime's `minimumVersion`.

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
tagged `sdk-node-v<version>`. A release ships
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

## API

| API | Purpose |
| --- | --- |
| `createBackend({ name?, version?, capabilities? })` | Creates the backend. `capabilities` are standard LSP server capabilities; `experimental.litheBuild`, `experimental.litheFunctions`, and `experimental.litheVcs` are reserved for the SDK. |
| `backend.build.model(handler)`, `backend.build.dependencies(handler)` | Implements the typed build requests. The SDK stamps `format` and `version`; a handler returning another format or a version other than 1 fails. Core validates the document. `litheBuild` is answered only when a handler exists. |
| `backend.build.execute(handler)` | Protocol 2+: provider-owned bounded task, returning `{ exitCode, output }`. |
| `backend.build.test(handler)`, `backend.build.discoverTests(handler)` | Protocol 3+: exact test execution and runnable selectors; discovery gets `format: "litheTestDiscovery"`, `version: 1`. |
| `backend.build.prepareRun(handler)` | Protocol 4: returns a portable launch plan after upstream synchronization/build; the host owns the application process. |
| `backend.buildVersion` | Highest common negotiated build version (1–6), or `null`; methods newer than that version are refused before their handler runs. |
| `backend.functionsVersion` | Highest common negotiated plugin-function version (1–3), or `null`. |
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
| `backend.log(...)` | Writes a line to stderr, which the host keeps in its log. |
| `backend.listen()` | Serves stdio until `exit` or the end of input. |
| `PluginError(code, message)` | A failure meant for your own UI, or with a `VCS_ERRORS` code, for the host's. |
| `HostError` | The host refused or failed a host request: `code` (`error.data.code`), `rpcCode`, and protocol 3 `details`. `HOST_ERRORS` maps each code to its number. |
| `runProcess(command, args, options)` | Runs a process without a shell, as one cancellable process tree. |
| `openStateStore(directory)` | A small atomic JSON store in a host-provided directory. |

Every handler receives a context:

| Member | Meaning |
| --- | --- |
| `signal` | Aborted when Lithe cancels the call, its deadline passes, the function is withdrawn, or the session ends; also once the handler has settled, so work it left running stops. |
| `throwIfCancelled()` | Throws the cancellation if `signal` is aborted. |
| `run(command, args, options)` | `runProcess` bound to the call: cancelling or ending the call ends the whole process tree. In VCS handlers it also reports the finished command unless `options.report` is `false`. |
| `log(...)` | Same as `backend.log`. |
| `host` | `backend.host` bound to the call: host requests it left unanswered are cancelled when the call ends. |
| `functionID` | Functions only. |
| `workspaceContext` | Functions only, protocol 3: `{ contextID, generation }` of the page's window, or `null`. |
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

## Results and errors

The following mapping describes private functions. Typed build operations use
their domain result/error contract; see the native workflow guide above.

| Handler outcome | What Lithe sees |
| --- | --- |
| Returns a value (including `null`) | `result.value` |
| Throws `PluginError(code, message)` | `plugin.function.failed` with your `message` and `pluginErrorCode`. The code is for your plugin's own UI; Lithe never branches on it. |
| Throws anything else | `plugin.function.failed` with a generic message, so paths and stack traces never reach the UI. The real error goes to stderr. |
| Ends because of `signal` | `requestCancelled` (or Lithe's timeout/revocation outcome) |

A result must serialize to at most 1 MiB of JSON. A value JSON cannot encode
(a `BigInt`, a cycle) or a larger one fails like any other unexpected error and
is logged to stderr; the session keeps running. A function ID is dot-separated
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
4. **The package is read-only.** Keep caches, snapshots, and logs under the
   directories Lithe passes through launch templates (`${stateDir}`,
   `${workspaceStateDir}`). `openStateStore` refuses any directory inside the
   plugin package. Writing to the package breaks code signing and update
   deltas.
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
