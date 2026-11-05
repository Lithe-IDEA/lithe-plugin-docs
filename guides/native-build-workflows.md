# 原生构建、测试与运行准备 / Native build, test, and run preparation

[中文](#中文) · [English](#english)

## 中文

插件可以只提供工作区后端，不创建网页。用户从原生工具窗口执行构建、发现与运行测试、准备应用启动；宿主继续管理工作区、取消、输出和应用进程。

这些入口对应 `buildProviders` 2–6，macOS 宿主已经实现；Windows 宿主不支持构建提供者。Node SDK 和 Java SDK 从 0.2.0 起包含这些入口，更早的 SDK 不包含它们。不要手改契约副本或向旧版本宿主隐瞒 `requires.buildProviders`。

### 选择入口

| 工作 | Node SDK | Java SDK | 最低 `buildProviders` / 进程协议 |
| --- | --- | --- | --- |
| 执行有界任务 | `backend.build.execute` | `onExecute` | 2 |
| 运行精确测试 | `backend.build.test` | `onTest` | 3 |
| 发现可运行的用例 | `backend.build.discoverTests` | `onDiscoverTests` | 3 |
| 准备应用启动 | `backend.build.prepareRun` | `onPrepareRun` | 4 |
| 准备原生程序调试 | `backend.build.prepareDebug` | `onPrepareDebug` | 5 |

安装时检查清单的能力版本，启动时 SDK 从宿主提供的版本中选最高的共同版本。没有对应协议时，新方法在进入处理器之前被拒绝；不会改发私有函数或展开成另一条宿主命令。Node 用 `backend.buildVersion`、Java 用 `backend.negotiated().buildVersion()` 查看结果。

处理器必须在 `listen` 前注册。模型、依赖和测试发现回复由 SDK 补齐对应 `format` 和 `version: 1`；执行回复是 `{ exitCode, output }`，启动准备回复是完整的可移植计划。Core 继续检查选择器、工作区路径和大小限制。

### 所有权与清理

插件从其选定的上游引擎读取当前工程、测试和运行事实。发现结果中的每个用例都必须能通过同一套件、用例、工程与目标选择器精确执行；处理器重新核对选择，不把失效用例当成“运行全部”。不要在宿主或 SDK 里再扫描 Java 注解、拼 classpath 或解释 Maven 模型。

构建与测试处理器使用调用上下文的 `run` 启动工具，取消或调用结束会结束它拥有的工具树。工具正常返回非零码仍是执行结果；异常属于协议/准备失败。不要启动脱离调用的进程，或用一条私有函数代替这些领域入口。

运行准备先等待上游同步和必要构建，再返回 `executable`、原始 `arguments` 数组、工作区相对 `workingDirectory` 和 `environment`。`executable.kind` 只能是 `command`（宿主解析的裸命令名）或 `toolchain`（宿主工具链标识）。处理器不启动长期运行的应用；宿主校验后启动、停止并显示应用输出。构建失败、取消或旧结果失效时不能启动应用。

执行/测试请求的后端拒绝由 Core 报告为 `plugin.build.failed`。运行准备的 RPC 错误保留通用 LSP `serverError` 分类，不是带退出码的执行结果，也不使用私有函数的 `pluginErrorCode`。宿主在两种失败下都不能启动应用或改走旧命令。

执行、测试、运行与调试准备上下文携带 Core 分配的 `operationID`，进度自动带上它。Node 使用 `context.progress({ message, percentage, output })`；Java 使用 `context.progress(message, percentage, output)`。消息上限 4096 UTF-8 字节，单次输出上限 64 KiB，百分比限制在 0–100；取消或结束后的报告被丢弃。模型、依赖和发现不提供执行进度。

状态写入宿主提供的可写目录；安装后的插件包只读。`prepareRun` 的结果不能变成跨重启缓存的 classpath 或命令：宿主保存入口选择器，重启重新准备。

### 可运行示例

两套示例使用相同的虚构构建引擎，工程事实来自工作区的 `lithe-sample.json`。这是协议教学用的引擎，不是 Maven/Java 迁移成果。

- [Node 示例](../sdk/node/examples/sample-native-build/backend/main.mjs)及其[清单](../sdk/node/examples/sample-native-build/plugin.json)。开发时从源码引入 SDK；分发时将 `src/` 放入 `backend/vendor/`，把导入改为 `./vendor/index.mjs`。端到端测试按这种结构打包验证。
- [Java 示例](../sdk/java/examples/sample-native-build/src/main/java/org/example/samplenativebuild/SampleNativeBuild.java)及其[清单](../sdk/java/examples/sample-native-build/plugin.json)。运行 `mvn -B -ntp -f sdk/java/pom.xml package`，将 `sample-native-build/target/sample-native-build.jar` 放到包内 `backend/`。SDK 已 shade 进同一 JAR。

示例工程包含构建命令、精确用例及应用参数，例如：

```json
{
  "projects": [{
    "name": "app", "path": "app",
    "build": ["node", "-e", "console.log('built')"],
    "tests": [{ "suite": "Suite", "case": "exact case",
      "command": ["node", "-e", "console.log('passed')", "--"] }],
    "run": { "command": "node",
      "arguments": ["-e", "console.log(process.argv.slice(1))", "--"] }
  }]
}
```

模型的路径相对构建实例，后续请求的工程/目标选择器相对整个工作区；示例显式转换这两种表示，不把嵌套实例当成根工程。

`app` 目录需存在，测试工具需可执行。示例重新读取配置、核对工程/目标和测试，并解析真实路径以拒绝工作区外目录；实际插件应调用其上游工程引擎，而非复制这份虚构引擎。

### 验证

```sh
node --test sdk/node/test/*.test.mjs
npm ci --prefix sdk/node --ignore-scripts
npm run --prefix sdk/node test:types
mvn -B -ntp -f sdk/java/pom.xml verify
node --test scripts/*.test.mjs

# 锁定契约对应的兼容场景。
node scripts/sdk-e2e.mjs

# 宿主分支已提交，但尚未合并时。
node scripts/sdk-e2e.mjs --source /path/to/Lithe-IDEA --ref branch --build-workflows

# 显式使用尚未提交的宿主源码；不可和 --ref 混用。
node scripts/sdk-e2e.mjs --source /path/to/Lithe-IDEA --working-tree --build-workflows
```

`--working-tree` 只把 `rust/`、`shared/` 的暂存、未暂存和未跟踪源码复制到一次性 Core 检出，不修改源仓库，也不复制忽略的构建缓存。该模式报告基线 commit 加工作区改动，不能把输出里的基线 SHA 当成完整已提交源码。

契约镜像已包含 `buildProviders` 6，所以默认验证就会运行原生构建工作流场景；`--build-workflows` 用来在较旧的镜像或宿主分支上显式加入它们，每周的 preview 检查也显式加了这个参数。新的测试通过真实 Core 规划与解码，覆盖构建非零退出码、精确筛选、运行前构建、准备计划、进度、取消后的工具清理、并行工作区隔离，以及包含 SDK/JAR 的完整包文件不变。它不代替原生界面、真实 Maven/JDT 工程或 Windows 实机验证。

若复用预构建 Java 示例，分别设置 `LITHE_PLUGIN_SDK_JAVA_SAMPLE` 与 `LITHE_PLUGIN_SDK_JAVA_NATIVE_SAMPLE`。编译使用 JDK 21，产物面向 Java 8；可用 `LITHE_PLUGIN_SDK_JAVA_RUNTIME` 指定实际运行时。

调试适配器、长期服务和跨插件协作有各自的宿主契约；这些处理器不开放跨包私有函数或反向 `lithe/*` 调用。

### 原生程序调试准备

运行声明加 `debug: {"adapter":"native"}`，同一插件提供 `native` 适配器；manifest 要求 `buildProviders: 5` 与 `debugAdapters: 2`。适配器声明 launch/executable，使用 program、cwd、args、env 的目标绑定。不要用跨包 ID，也不要在后端返回 DAP 进程、端口或任意调试参数。

`prepareDebug` 先完成构建，再返回 `{program, arguments, workingDirectory, environment}`。程序是当前实例内的已有文件，路径从工作区起算；宿主校验路径和可执行权限，再复用现有 DebugAdapter Registry。取消、错误和过期结果都不能启动调试。Node 和 Java 的 sample-native-build 从虚构引擎的 `run.debugProgram` 读取程序，参数使用当前请求的 userArguments；示例的 `example-dap` 是占位适配器命令，真实插件应打包符合其许可的成熟适配器。

端到端示例验证两个 SDK 的真实进程、运行前构建、原生调试准备前构建、Core 路径解码、进度及失败不返回目标。它不启动真实调试器，也不代表 Java/JDT、跨插件协作或 Windows 已完成。

---

## English

A plugin can provide only a workspace backend and create no web page. Users build, discover and run tests, and prepare application launches from native tool windows; the host keeps managing the workspace, cancellation, output, and application processes.

These entry points are `buildProviders` 2–6, which the macOS host implements; the Windows host does not support build providers. The Node SDK and Java SDK include these entry points from 0.2.0; earlier SDKs do not. Do not edit the contract copy by hand, and do not hide `requires.buildProviders` from older hosts.

### Choosing an entry point

| Work | Node SDK | Java SDK | Minimum `buildProviders` / process protocol |
| --- | --- | --- | --- |
| Execute a bounded task | `backend.build.execute` | `onExecute` | 2 |
| Run exact tests | `backend.build.test` | `onTest` | 3 |
| Discover runnable cases | `backend.build.discoverTests` | `onDiscoverTests` | 3 |
| Prepare an application launch | `backend.build.prepareRun` | `onPrepareRun` | 4 |
| Prepare native program debugging | `backend.build.prepareDebug` | `onPrepareDebug` | 5 |

The manifest's capability versions are checked at install time, and at startup the SDK picks the highest version shared with what the host offers. Without the matching protocol, a new method is rejected before it reaches the handler; it is never rerouted to a private function or expanded into a different host command. Read the result with `backend.buildVersion` in Node or `backend.negotiated().buildVersion()` in Java.

Register handlers before `listen`. For model, dependency, and test discovery replies, the SDK fills in the matching `format` and `version: 1`; an execution reply is `{ exitCode, output }`, and a launch preparation reply is a complete portable plan. Core still checks selectors, workspace paths, and size limits.

### Ownership and cleanup

The plugin reads the current project, test, and run facts from the upstream engine it chose. Every case in a discovery result must be exactly runnable through the same suite, case, project, and target selectors; the handler checks the selection again and never treats a stale case as "run all". Do not scan Java annotations, assemble a classpath, or interpret a Maven model in the host or SDK.

Build and test handlers start tools through the call context's `run`; cancellation or the end of the call ends the tool tree it owns. A tool that returns a non-zero code normally is still an execution result; an exception is a protocol or preparation failure. Do not start processes that outlive the call, and do not replace these domain entry points with a private function.

Run preparation first waits for upstream sync and any required build, then returns `executable`, the raw `arguments` array, a workspace-relative `workingDirectory`, and `environment`. `executable.kind` is only `command` (a bare command name the host resolves) or `toolchain` (a host toolchain identifier). The handler does not start a long-running application; the host validates the plan, then starts and stops the application and shows its output. A build failure, cancellation, or stale result must not start the application.

Core reports a backend rejection of an execute or test request as `plugin.build.failed`. An RPC error from run preparation keeps the generic LSP `serverError` classification; it is not an execution result with an exit code and does not use the private-function `pluginErrorCode`. In both failures the host must not start the application or fall back to an old command.

Execute, test, run preparation, and debug preparation contexts carry an `operationID` assigned by Core, and progress carries it automatically. Node uses `context.progress({ message, percentage, output })`; Java uses `context.progress(message, percentage, output)`. Messages are limited to 4096 UTF-8 bytes, each output chunk to 64 KiB, and percentages are clamped to 0–100; reports after cancellation or completion are dropped. Model, dependency, and discovery requests have no execution progress.

State goes into the writable directories the host provides; an installed plugin package is read-only. A `prepareRun` result must not become a classpath or command cached across restarts: the host stores the entry selector and prepares again after a restart.

### Runnable examples

Both examples use the same fictional build engine, whose project facts come from `lithe-sample.json` in the workspace. It is an engine for teaching the protocol, not a result of migrating Maven or Java.

- The [Node example](../sdk/node/examples/sample-native-build/backend/main.mjs) and its [manifest](../sdk/node/examples/sample-native-build/plugin.json). During development it imports the SDK from source; for distribution, put `src/` into `backend/vendor/` and change the import to `./vendor/index.mjs`. The end-to-end tests package and verify that layout.
- The [Java example](../sdk/java/examples/sample-native-build/src/main/java/org/example/samplenativebuild/SampleNativeBuild.java) and its [manifest](../sdk/java/examples/sample-native-build/plugin.json). Run `mvn -B -ntp -f sdk/java/pom.xml package` and put `sample-native-build/target/sample-native-build.jar` into the package's `backend/`. The SDK is already shaded into the same JAR.

The example project contains a build command, exact cases, and application arguments, for example:

```json
{
  "projects": [{
    "name": "app", "path": "app",
    "build": ["node", "-e", "console.log('built')"],
    "tests": [{ "suite": "Suite", "case": "exact case",
      "command": ["node", "-e", "console.log('passed')", "--"] }],
    "run": { "command": "node",
      "arguments": ["-e", "console.log(process.argv.slice(1))", "--"] }
  }]
}
```

Model paths are relative to the build instance, while project and target selectors in later requests are relative to the whole workspace; the example converts between the two explicitly and never treats a nested instance as the root project.

The `app` directory must exist and the test tool must be executable. The example rereads its configuration, checks the project, target, and test, and resolves real paths to reject directories outside the workspace. A real plugin should call its upstream project engine instead of copying this fictional one.

### Verification

```sh
node --test sdk/node/test/*.test.mjs
npm ci --prefix sdk/node --ignore-scripts
npm run --prefix sdk/node test:types
mvn -B -ntp -f sdk/java/pom.xml verify
node --test scripts/*.test.mjs

# Compatibility scenarios for the pinned contract.
node scripts/sdk-e2e.mjs

# When the host branch is committed but not yet merged.
node scripts/sdk-e2e.mjs --source /path/to/Lithe-IDEA --ref branch --build-workflows

# Explicitly use uncommitted host sources; cannot be combined with --ref.
node scripts/sdk-e2e.mjs --source /path/to/Lithe-IDEA --working-tree --build-workflows
```

`--working-tree` copies only the staged, unstaged, and untracked sources under `rust/` and `shared/` into a throwaway Core checkout. It does not modify the source repository or copy ignored build caches. This mode reports the base commit plus working-tree changes, so the base SHA in its output does not stand for complete committed sources.

The contract mirror already contains `buildProviders` 6, so the default run includes the native build workflow scenarios; `--build-workflows` adds them explicitly against an older mirror or a host branch, and the weekly preview check passes it explicitly too. The new tests plan and decode through a real Core and cover a non-zero build exit code, exact filtering, build before run, preparation plans, progress, tool cleanup after cancellation, isolation between parallel workspaces, and an unchanged complete package including the SDK and JAR. They do not replace verification of the native UI, real Maven/JDT projects, or real Windows machines.

To reuse prebuilt Java examples, set `LITHE_PLUGIN_SDK_JAVA_SAMPLE` and `LITHE_PLUGIN_SDK_JAVA_NATIVE_SAMPLE`. They compile with JDK 21 and target Java 8; `LITHE_PLUGIN_SDK_JAVA_RUNTIME` selects the runtime that actually runs them.

Debug adapters, long-running services, and cross-plugin collaboration have host contracts of their own; these handlers open no cross-package private functions or reverse `lithe/*` calls.

### Native program debug preparation

Add `debug: {"adapter":"native"}` to the run declaration and provide a `native` adapter from the same plugin; the manifest requires `buildProviders: 5` and `debugAdapters: 2`. The adapter declares launch/executable and uses the target bindings for program, cwd, args, and env. Do not use a cross-package ID, and do not return a DAP process, port, or arbitrary debug arguments from the backend.

`prepareDebug` finishes the build first and then returns `{program, arguments, workingDirectory, environment}`. The program is an existing file inside the current instance, with a path relative to the workspace; the host checks the path and execute permission and then reuses the existing DebugAdapter Registry. Cancellation, errors, and stale results must not start debugging. The Node and Java sample-native-build examples read the program from the fictional engine's `run.debugProgram` and use the current request's userArguments as arguments. The examples' `example-dap` is a placeholder adapter command; a real plugin should package a mature adapter whose license allows it.

The end-to-end example verifies real processes for both SDKs, build before run, build before native debug preparation, Core path decoding, progress, and that failures return no target. It does not start a real debugger and does not mean Java/JDT, cross-plugin collaboration, or Windows is done.
