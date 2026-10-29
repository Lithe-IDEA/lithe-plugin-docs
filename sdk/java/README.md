# Lithe plugin backend SDK for Java

`lithe-plugin-sdk-java` is the official way to write a plugin backend in Java:
the process a plugin starts through a `runtime` launch
(`{ "kind": "runtime", "tool": "java", "minimumVersion": "8", "entry": "backend/app.jar" }`).
It is the Java twin of the [Node.js SDK](../node/README.md). Both implement the
same two protocols from
[plugin contract v1](../../contract/v1/README.md) with the
same rules and error mapping:

- **`litheBuild` 1**: typed `lithe/build/model` and `lithe/build/dependencies`
  requests, for a build system whose `model` or `dependencies` source is
  `provider`.
- **`litheFunctions` 1**: plugin functions, which are plugin-private actions
  that Lithe invokes for the same plugin. They support progress,
  cancellation, and registration at runtime.

The SDK requires Java 8 or newer and has no runtime dependencies. JSON values
are plain `Map<String, Object>`, `List<Object>`, `String`, `Long`, `Double`,
`Boolean`, and `null` (see `Json`).

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
tagged `sdk-java-v<version>`. Version 0.1.0 was released from the Lithe
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
| `Backend.create(name, version[, capabilities])` | Creates the backend. `capabilities` are standard LSP server capabilities. `experimental.litheBuild` and `experimental.litheFunctions` are reserved for the SDK. |
| `onModel(handler)`, `onDependencies(handler)` | Implement the typed build requests. The SDK stamps `format` and `version`. Core validates the document. `litheBuild` is answered only when a handler exists. |
| `onExecute(handler)` | Protocol 2+: a bounded provider-owned task, returning `{ exitCode, output }`. |
| `onTest(handler)`, `onDiscoverTests(handler)` | Protocol 3+: exact execution and runnable selectors; discovery format/version are stamped. |
| `onConfiguration(handler)` | Protocol 7+: read-only non-sensitive raw/effective native configuration; stamps `litheBuildConfiguration`, version 1. The host owns explicit native-file saves. |
| `onPrepareRun(handler)` | Protocol 4: prepare a portable plan; the host owns the application. |
| `negotiated().buildVersion()` | Highest common build version (1–7), or zero; newer methods are refused before their handler runs. |
| `function(id, handler)` | Declares a function offered in `initialize`. |
| `registerFunction(id, handler)`, `unregisterFunction(id)` | Add or withdraw a function. Called before the `initialize` answer (for example from `onInitialize`), they change the initial list. Later calls send `lithe/function/register` or `lithe/function/unregister`, in order. Both return a `CompletableFuture`. Withdrawing a function revokes its running calls. |
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
| `operationId()` | Opaque wire identity for functions and bounded build/test/run-preparation calls; null for other requests. |
| `progress(message, percentage[, output])` | Functions use 1024-byte messages. Build/test/run preparation use 4096-byte messages and optional output up to 64 KiB. Percentages are clamped to 0–100. Cancelled or settled calls cannot report. Model/dependency/discovery reports are ignored. |

The new build handlers are under development and are not in a published SDK
release. See [native build workflows](../../guides/native-build-workflows.md)
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

## Rules that keep Lithe in charge

These are the [Node.js SDK's rules](../node/README.md#rules-that-keep-lithe-in-charge).
In Java:

1. **Domain capabilities stay typed.** Build models and dependency trees go
   through `onModel` and `onDependencies`, never through a function.
2. **No calls back into Lithe's dispatch.** The SDK refuses to send any
   `lithe/*` method.
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
migrate Maven, JDT, LLDB or existing plugin business logic. The contract mirror
must be synced from the merged Lithe revision before these changes are released.

## Inline test reports (development protocol 8)

A test declared with `results: { "decoder": "provider" }` requires buildProviders 8.
Core sends `testResults: true`; answer with `testResults: { format: "litheTestResults",
version: 1, tests: [...] }` beside `exitCode` and `output`. Cases carry suite/case/status
and optional project/source paths, message, durationMillis and invocations. A failed
test still returns its nonzero exit code and current reports. Report discovery and
freshness belong to the provider's upstream engine; never return old files or silently
empty results after a read failure. Core validates 10,000 cases/8 MiB and keeps equal
suite/case names in distinct projects separate. Earlier negotiation refuses the new
request field. Ordinary execute replies may include reports for build phases that run
tests. These SDK sources are development inputs pending a coordinated host/SDK release.

## Opaque test selectors (development protocol 9)

`onTest` can receive `selectorKind` and `selectors` for a namespace declared by
`test.selectorKinds`. The SDK admits them only after protocol 9 negotiation and
rejects conflicts with suite/case, more than 64 values, empty/control-containing
values and values exceeding 4096 UTF-8 bytes. An empty list selects the normal
workspace/project scope. Preserve language-engine identities until mapping them
to the upstream runner, and reject selections that cannot be represented exactly.
