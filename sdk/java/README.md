# Lithe plugin backend SDK for Java

`lithe-plugin-sdk-java` is the official way to write a plugin backend in Java:
the process a plugin starts through a `runtime` launch
(`{ "kind": "runtime", "tool": "java", "minimumVersion": "8", "entry": "backend/app.jar" }`).
It is the Java twin of the [Node.js SDK](../node/README.md). Both implement the
the following protocols from
[plugin contract v1](../../contract/v1/README.md) with the
same rules and error mapping:

- **`litheBuild` 1**: typed `lithe/build/model` and `lithe/build/dependencies`
  requests, for a build system whose `model` or `dependencies` source is
  `provider`.
- **`litheFunctions` 1–3**: plugin functions, which are plugin-private actions
  that Lithe invokes for the same plugin. They support progress,
  cancellation, and registration at runtime. From version 2 a backend may
  call [host services](#host-services); version 3 adds workspace, document,
  editor, and window services, command events, and workspace contexts.
- **`litheKeymapImport` 1** (unreleased): independently negotiated shortcut
  import providers reading only explicitly selected source snapshots.

The SDK requires Java 8 or newer and has no runtime dependencies. JSON values
are plain `Map<String, Object>`, `List<Object>`, `String`, `Long`, `Double`,
`Boolean`, and `null` (see `Json`).

## Shortcut import providers (unreleased)

Declare `backend.keymapImportProvider(id, handler)` before listening. Its
handler receives a `KeymapSnapshot` with `providerId()`, `snapshotId()`,
`listInputs()` and `readChunk(fileId, offset, maximumBytes)`, plus the normal
call context. Paths returned by `listInputs` are relative labels; reads use
opaque file IDs. Return a normalized `{ bindings: [...] }` map and observe
the call's cancellation. The host revokes the snapshot when the call ends.
Do not turn conditional or multi-stroke rules into global bindings. Product
discovery and the import workflow are not available yet.

## A minimal backend

```java
Backend backend = Backend.create("example-backend", "1.0.0");

backend.onModel((params, context) -> {
    Map<String, Object> project = new LinkedHashMap<>();
    project.put("name", "app");
    project.put("path", Json.string(params, "root"));
    Map<String, Object> model = new LinkedHashMap<>();
    model.put("projects", Collections.singletonList(project));
    return model;
});

backend.function("snapshot.refresh", (params, context) -> {
    context.progress("Refreshing", 10.0);
    Processes.Result result = context.run(Arrays.asList("example-tool", "refresh"));
    if (result.exitCode() != 0) {
        throw new PluginError("example.refreshFailed", "The refresh failed.");
    }
    return Collections.singletonMap("refreshedAt", System.currentTimeMillis());
});

backend.listen();
```

[`examples/sample-build`](examples/sample-build) is a complete plugin. It has
a build provider, functions, persistent state, a function registered at
runtime, and cancellation. `node scripts/sdk-e2e.mjs` drives it through a real
Lithe Core session with the scenario in [`sdk/e2e`](../e2e/plugin_sdk/mod.rs),
the same one the Node.js sample answers.

## Getting the SDK

Releases are published on [this repository's Releases page](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases) and
tagged `sdk-java-v<version>`. Version 0.2.0 adds plugin-function protocols 2
and 3. Version 0.1.0 was released from the Lithe
repository before the SDK moved here; its
[release](https://github.com/1lck/Lithe-IDEA/releases/tag/sdk-java-v0.1.0)
stays available. A release ships
`lithe-plugin-sdk-java-<version>.jar` and a `.sha256` file holding the JAR's
SHA-256. The SDK is not published to a Maven repository. Use it like this:

1. Download the JAR during your plugin's build, never at runtime.
2. Check the JAR against the SHA-256 you pinned in your repository.
3. Shade it into your backend JAR, as the sample's `pom.xml` does.

A plugin package is installed read-only and must run offline, so the backend
JAR it starts must hold everything it loads.

## API

| API | Purpose |
| --- | --- |
| `Backend.create(name, version[, capabilities])` | Creates the backend. `capabilities` are standard LSP server capabilities; the SDK negotiates its experimental protocols. |
| `onModel(handler)`, `onDependencies(handler)` | Implement the typed build requests. The SDK stamps `format` and `version`. Core validates the document. `litheBuild` is answered only when a handler exists. |
| `onExecute(handler)` | Protocol 2+: a bounded provider-owned task, returning `{ exitCode, output }`. |
| `onTest(handler)`, `onDiscoverTests(handler)` | Protocol 3+: exact execution and runnable selectors; discovery format/version are stamped. |
| `onPrepareRun(handler)` | Protocol 4: prepare a portable plan; the host owns the application. |
| `negotiated().buildVersion()` | Highest common build version (1–6), or zero; newer methods are refused before their handler runs. |
| `function(id, handler)` | Declares a function offered in `initialize`. |
| `registerFunction(id, handler)`, `unregisterFunction(id)` | Add or withdraw a function. Called before the `initialize` answer (for example from `onInitialize`), they change the initial list. Later calls send `lithe/function/register` or `lithe/function/unregister`, in order. Both return a `CompletableFuture`. Withdrawing a function revokes its running calls. |
| `negotiated().functionsVersion()` | Highest common plugin-function version (1–3), or zero. |
| `host()` | [Host services](#host-services) for work outside a call, such as a command handler. |
| `onCommand(commandId, handler)` | Protocol 3: runs `handler.handle(commandId, workspaceContext)` when the user runs one of the plugin's commands. |
| `onContextsChanged(handler)` | Protocol 3: `lithe/host/workspace/didChangeContexts` with its `{ revision }`. |
| `onHostNotification(method, handler)` | A `lithe/host/*` notification a later host adds. The three this SDK routes itself are reserved. |
| `onInitialize(hook)`, `onShutdown(hook)` | Lifecycle hooks. `onInitialize` receives the `initialize` params and what was negotiated. |
| `onRequest(method, handler)`, `onNotification(method, handler)` | Ordinary LSP methods. Lifecycle and `lithe/*` methods are reserved. |
| `sendRequest`, `sendNotification` | Ordinary LSP messages to the host. `lithe/*` methods are reserved. |
| `log(message[, error])` | Writes to stderr, which the host keeps in its log. |
| `listen()` | Serves stdio until `exit` or the end of input, then exits the JVM with the protocol's exit code. `listen(in, out, err)` serves other streams and returns the code instead. |
| `PluginError(code, message)` | A failure meant for your own UI. |
| `Processes.run(command[, options])` | Runs a process without a shell. Its stdin is closed. `Options` set the directory, environment, output limits, a cancellation, and output callbacks. |
| `StateStore.open(directory)` | A small atomic JSON store in a host-provided directory. |

Handlers run on their own threads. Every handler receives a `CallContext`:

| Member | Meaning |
| --- | --- |
| `cancellation()`, `isCancelled()` | Cancelled when any of these happens: Lithe cancels the call, its deadline passes, the function is withdrawn, the session ends, or the handler finishes. Finishing the handler stops the work it left running. |
| `throwIfCancelled()` | Throws `CancelledException` once the call is cancelled. |
| `run(command[, options])` | `Processes.run` bound to the call: cancelling or ending the call stops the process. |
| `log(message)` | Same as `Backend.log`. |
| `functionId()` | Functions only. |
| `host()` | [Host services](#host-services) bound to the call: ending the call cancels the requests it left unanswered. |
| `workspaceContext()` | Protocol 3 functions: the `{ contextID, generation }` of the window that invoked the call, or null. |
| `operationId()` | Opaque wire identity for functions and bounded build/test/run-preparation calls; null for other requests. |
| `progress(message, percentage[, output])` | Functions use 1024-byte messages. Build/test/run preparation use 4096-byte messages and optional output up to 64 KiB. Percentages are clamped to 0–100. Cancelled or settled calls cannot report. Model/dependency/discovery reports are ignored. |

The typed handlers of build protocols 2–6 ship from SDK 0.2.0. See [native build workflows](../../guides/native-build-workflows.md)
and the [no-page Java sample](examples/sample-native-build/src/main/java/org/example/samplenativebuild/SampleNativeBuild.java).
Manifest `buildProviders` requirements and the negotiated protocol both apply;
registering a handler never bypasses either boundary. Build tool failure is an
exit-code result; throwing an error aborts preparation.

Private function results and errors map exactly as in the Node.js SDK; typed
build operations follow their domain contract:

- A returned value becomes `result.value`.
- `PluginError` becomes `plugin.function.failed` with your message and
  `pluginErrorCode`.
- Any other exception is reported with a generic message, and its stack trace
  goes to stderr.
- `CancelledException` ends the call as cancelled.
- A result larger than 1 MiB of JSON fails like an unexpected error.

## Host services

From plugin-function protocol 2 a backend may ask the host for services;
each is granted by a manifest permission and refused otherwise. The SDK
negotiates the highest version both sides offer, so a backend written for
protocol 1 keeps working unchanged. Every method returns a
`CompletableFuture`; wrong arguments fail it rather than throwing.

```java
backend.function("query.run", (params, context) -> {
    // Protocol 2, permission credentials.store: the platform's secret store, per plugin.
    String password = context.host().credentials().get("connection.prod:password").get();
    if (password == null) {
        throw new PluginError("query.noPassword", "Sign in first.");
    }
    // Protocol 3: the window the page runs in.
    Map<String, Object> workspace = context.workspaceContext();
    if (workspace != null && context.host().supports("lithe/host/workspace/getFolders").get()) {
        Object folders = context.host().workspace().getFolders(workspace).get().get("folders");
        // ...
    }
    return runQuery(params, password);
});

backend.onCommand("db.refresh", (commandId, workspaceContext) -> {
    if (workspaceContext != null) {
        backend.host().window().showInformation(workspaceContext, Collections.singletonMap("text", "Refreshed."));
    }
});
```

| `Host` member | Version | Purpose |
| --- | --- | --- |
| `credentials().get(key)`, `.set(key, secret)`, `.delete(key)` | 2 | Secrets in the platform's store. `get` completes with null when nothing is stored. A key is 1–128 of `[A-Za-z0-9._:-]`, a secret at most 16 KiB; others fail locally. |
| `capabilities()`, `supports(method)` | 3 | What this session may call, from `lithe/host/getCapabilities` (asked once). Under protocol 2, which has no discovery, `supports` reports the credential methods. |
| `workspace().listContexts()`, `.getContext(context)`, `.getFolders(context)` | 3 | Workspace contexts; only `getContext` returns, and creates, the context's `stateDirectory`. |
| `workspace().readDirectory`, `.stat`, `.readFile`, `.findFiles`, `.findText`, `.watch` | 3 | Files of a local context, addressed as `{ rootID, path }` from `getFolders`. |
| `documents().list`, `.getSnapshot`; `editor().getActive`, `.getSelections`, `.applyEdits`, `.replaceSelections`, `.insertText`, `.open`, `.reveal`, `.registerActions` | 3 | Live documents and editors. Edits name the `expectedVersion` they were computed against. |
| `preview().openHtml`, `browser().openExternal`, `window().showInformation`, `.showWarning`, `.showError` | 3 | Preview, browser, and messages. |
| `request(method, params)`, `subscribe(method, params, listener)` | 2 / 3 | Any `lithe/host/*` method, including ones a later host adds. |
| `subscriptions().dispose(id)` | 3 | Ends a subscription; `Subscription.dispose()` does the same. |
| `withCancellation(cancellation)` | 2 | The same services, cancelled together with `cancellation`. |

Methods taking a workspace context accept `{ contextID, generation }` or a
whole context from `listContexts`; their `params` are the contract's, without
`context`. The full params and results are in
[the contract](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3).

What the SDK does for you, and what it leaves to you:

- **Refusals are `HostError`.** Branch on `code()`, such as
  `plugin.host.contextExpired`; a code this SDK does not know is still a
  `HostError`, and `details()` holds the host's object. On `versionConflict`
  reread and recompute; on `contextExpired` resolve the original target
  again, never the focused window; retry `busy` and `rateLimited` (after
  `details().get("retryAfterMilliseconds")`) with a bound; `permissionDenied`
  and `readOnly` need the user. From protocol 3 an unknown param is
  `plugin.host.unsupportedOption` naming it in `details().get("option")`, so
  you can retry without it.
- **Cancellation has no rollback.** Cancelling the token given to
  `withCancellation`, or ending the call that sent a request, fails it at
  once with `CancelledException`; from protocol 3 the SDK also sends
  `$/cancelRequest` so the host stops. An edit or an external open may
  already have taken effect, and the SDK never sends a request again.
  Cancelling the returned future itself does not reach the host.
- **At most 16 host requests are unanswered.** Further ones wait in the SDK
  for a free slot instead of being refused.
- **Subscriptions route by ID.** `workspace().watch` and `subscribe` call
  the `Listener` with each `Host.Event` in order, one at a time; `gap()` is
  true when events were missed, so rescan. Nothing arrives after `dispose()`
  or the end of the session.
- **Commands are events.** `onCommand` runs when the user runs one of the
  plugin's commands; the host waits for no result. A stopped backend is
  started for it only when the manifest declares `onCommand:<commandID>`.

## Rules that keep Lithe in charge

These are the [Node.js SDK's rules](../node/README.md#rules-that-keep-lithe-in-charge).
In Java:

1. **Domain capabilities stay typed.** Build models and dependency trees go
   through `onModel` and `onDependencies`, never through a function.
2. **No calls back into Lithe's dispatch.** `sendRequest` and
   `sendNotification` refuse any `lithe/*` method. Host services go through
   `host()`, which sends only `lithe/host/*`.
3. **stdout belongs to the protocol.** `listen()` keeps the real stdout for
   protocol frames and points `System.out` at stderr. Never write to
   `FileDescriptor.out`.
4. **The package is read-only.** Keep state under the directories Lithe
   passes through launch templates (`${stateDir}`, `${workspaceStateDir}`).
   `StateStore.open` refuses a directory inside the plugin package. It finds
   the package by walking up from the SDK's own JAR to `plugin.json`.
5. **Work ends with its call.** Start processes with `context.run`, and check
   `isCancelled()` in long loops. On cancellation the SDK first asks the
   process to stop, then stops it forcibly after a grace period.
   - **macOS and Linux, every Java version:** each tool leads its own process
     group, as in the Node SDK. The SDK starts it through the system `perl`,
     which calls `setpgrp` and then becomes the tool. A stop reaches the whole
     group, and when the tool exits on its own, whatever it left running in
     its group is killed.
   - **Windows, or no `perl`:** on Java 9 or newer the SDK stops the
     descendants still attached to the tool; Java 8 stops only the direct
     child.

   A descendant that leaves the group (`setsid`) or that the fallback cannot
   reach is ended by Lithe's process scope together with the backend.
   A cancellation listener that throws is logged and does not keep the other
   listeners or the reply from running.

   On `shutdown`, `exit`, or the end of input the SDK ends every process it
   started. `Processes.run` keeps Lithe's process-scope markers even when
   `Options.environment` replaces the environment.
6. **Errors stay presentable.** Throw `PluginError` for failures the user
   should see; every other exception is reported generically.

## Testing

```bash
mvn -B -f sdk/java/pom.xml verify
```

This builds the SDK and the sample for Java 8 with any newer JDK, then runs
the JUnit tests. Add `-Djvm=/path/to/java8/bin/java` to run the tests on a
Java 8 runtime, as CI does.


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

## Native terminal API (unreleased)

The SDK includes wrappers for plugin-owned native terminal create/show/close,
reopen with bounded display history, disposal and exit subscriptions. These
methods are discovered within protocol 3; they require a newer host and
`terminal.create`, not a new protocol version. See
[the native terminal guide](../../guides/native-terminals.md).
