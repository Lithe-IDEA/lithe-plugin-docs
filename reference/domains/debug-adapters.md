# 调试适配器 / Debug adapters

[中文](#中文) · [English](#english)

## 中文

插件在 `debugAdapters[]` 里声明自己实现的调试适配器。每个适配器是一个 DAP 进程，宿主启动它，并通过 stdio 或本机回环 TCP 端口和它通信；也可以是插件语言服务器里内置的调试服务（例如运行在 JDT LS 里的 Java Debug Server）。声明只是数据：没有解析器、工作流、宿主集成或 `initializationOptions`，出现这些字段会被拒绝。调试界面、断点、变量、控制台都归宿主。插件后端还可以用宿主方法 `debug/start` 在自己声明的适配器上发起调试会话。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `debugAdapters[]` |
| 能力版本 | `debugAdapters` 1–3，需要级别 4；`launch.process` 用到的 `serverProcess` 1，`runSelectedRuntime` 需要 `serverProcess` 2。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | DAP（Debug Adapter Protocol），没有 Lithe 专用协议；后端发起会话用 `litheFunctions` 3 的宿主方法 `debug/start`，见[由插件后端发起调试会话](#由插件后端发起调试会话) |
| API 编号 | [API-30](../api-catalog.md#现在能用的-api)：调试适配器，包括 `debug/start` |
| 权限 | `debug.launch`；任何适配器支持 `attach` 时加 `debug.attach`（远程 attach 不另设权限）；后端调用 `debug/start` 时加 `debug.start`（需要 `backend` 2 和已声明的 `debugAdapters` 条目，只授予 `debug/start`）；`tcpLanguageServer` 依赖同一插件的语言服务器，所以还要有 `languageServers` 和 `languageServer.launch` |
| 激活事件 | 每个适配器一个 `onDebug:<id>` |
| Windows | 不支持；Windows 宿主不声明 `debugAdapters` |

### 版本对照

| 版本 | 最低级别 | 新增 | macOS | Windows |
| --- | --- | --- | --- | --- |
| `debugAdapters` 1 | 4 | `debugAdapters[]` 的全部字段：`launch`/`attach` 请求、自动匹配 `selectors`、`configurationTypes`、默认参数与 `targetBindings`（`program`、`processId`、`cwd`、`connection`）、`executable` 候选、`capabilities.disabled`、`console`、四种传输 | 支持 | 不支持 |
| `debugAdapters` 2 | 4 | `targetBindings` 新增有界的 `args` ← `target.arguments`、`env` ← `target.environment` | 支持 | 不支持 |
| `debugAdapters` 3 | 4 | 适配器可声明 `features`（`hotCodeReplace`、`stepFilters`）和 `defaults.stepFilters`；任何 `features` 或 `defaults` 都要求 `requires.debugAdapters` 3，见[适配器行为](#适配器行为debugadapters-3) | 支持 | 不支持 |
| `serverProcess` 2 | 4 | `launch.process` 可以写成 `{"launch": {"kind": "runSelectedRuntime", "tool": "python" \| "java" \| "node"}}`，由宿主用用户选中的运行时启动适配器；这是 `serverProcess` 2 唯一的用途。从 `serverProcess` 4 起 `tool` 是 `requiresTools` 列出的[工具定义 ID](../../contract/v1/README.md#tool-references-serverprocess-4) | 只有 Core 支持，macOS 宿主声明 `serverProcess` 1 和 3 | 不支持 |
| `buildProviders` 5 | 4 | 构建提供者为原生调试准备目标，运行声明 `debug: {"adapter": "native"}`；同时需要 `debugAdapters` 2，见[构建提供者](build-providers.md) | 支持 | 不支持 |
| `buildProviders` 11 | 3 | 服务运行声明 `debug: {"configurationType": "…"}`，宿主选一个已启用、支持 `attach` 和该类型的适配器连接 | 支持 | 不支持 |

低于级别 4 的清单不能声明 `debugAdapters`；级别 3 的清单不能要求 `debugAdapters` 或 `serverProcess` 2。

### 传输方式

| `launch.transport` | 端点 | 需要 `launch.process` |
| --- | --- | --- |
| `stdio` | 适配器的标准输入输出 | 是 |
| `tcpAnnounced` | 适配器在输出流里宣布端口：`endpoint` 带 `stream`、单行 `prefix`、`value`（`port` 或 `hostPort`） | 是 |
| `tcpPort` | 固定端口 `endpoint.port` | 是 |
| `tcpLanguageServer` | 同一插件的语言服务器：`endpoint` 带 `languageID` 和 `command`；宿主每次会话发 `workspace/executeCommand`，服务器回答回环端口 | 否，且不能有 |

所有 TCP 端点只能是本机回环地址。`tcpLanguageServer` 的命令是安装和更新审查的一部分，命令改变时会重新询问用户。

### 适配器行为（`debugAdapters` 3）

宿主从不按适配器 ID 识别适配器。适配器特有的行为由声明给出，共享的调试引擎按声明执行：

| 声明 | 含义 |
| --- | --- |
| `features.hotCodeReplace.request` | 宿主向运行中的被调试程序应用变更代码时发送的 DAP 命令（标准的 `redefineClasses` 检查）。写了表示适配器支持热代码替换，没写表示不支持 |
| `features.hotCodeReplace.event` | 一个 DAP 事件，调试控制台把它的 `message` 显示为进度 |
| `features.hotCodeReplace.changedField`、`errorField` | 响应体里的字段：已替换单元的列表，以及适配器在成功响应里报告的失败消息。没有 `changedField` 的响应不会显示任何已替换内容 |
| `features.stepFilters.argument` | 适配器接收步进过滤器的启动参数，形状为 `{skipClasses, skipSynthetics, skipStaticInitializers, skipConstructors}`。用户配置里已设置的参数保持不变 |
| `defaults.stepFilters` | `classNameFilters`（最多 256 个单行模式，每个最多 256 个字符）、`skipSynthetics`、`skipStaticInitializers`、`skipConstructors`、`hideFilteredStackFrames`。启动和用户设置都没有给出过滤器时使用 |

名称（`request`、`event`、各字段、`argument`）是一个字母加最多 63 个字母、数字、`.`、`_` 或 `-`。名称格式错误，或写了这一版没有定义的字段，会被拒绝为 `plugin.manifest.invalid`，`field` 指明位置；任何 `features` 或 `defaults` 都要求 `requires.debugAdapters` 3。启动计划的 `debugAdapter` 描述符同时携带两者，宿主据此传给 `debug.createSession`。两者都不声明的插件，行为和 `debugAdapters` 1、2 相同。

### 远程 attach

连接另一台机器上的被调试程序（带 `-agentlib:jdwp` 的 JVM、Node inspector、gdbserver）时，适配器仍在本机运行，只有被调试程序在远端。`attach` 配置的 `targetBindings.connection` 把用户确认的主机、端口、路径映射写进适配器的 `arguments`：

| 字段 | 说明 |
| --- | --- |
| `host` + `port` | 一起声明，值是 `arguments` 里的路径（1–4 段，如 `connect.host`） |
| `address` | 单个 `host:port` 字符串，IPv6 用方括号；可以代替 `host` + `port`，也可以同时写 |
| `pathMappings.shape: "objects"` | 对象列表，`localKey`、`remoteKey` 命名每个对象的键 |
| `pathMappings.shape: "first"` | 只有一对根目录的适配器；目标多于一对时拒绝，不截断 |
| `pathMappings.shape: "dictionary"` | 一个对象，`keyedBy` 决定哪一侧做键 |
| `pathMappings.shape: "pairs"` | 二元数组列表，`order` 为 `remoteLocal` 或 `localRemote` |

Core 只把 `localhost`、`127.0.0.0/8`、`::1` 判为本机；其他主机都是 `remote: true`，宿主每次都请用户确认，默认不记住。`connection` 不声明外部 DAP 端点或远程执行适配器（例如 `ssh host gdb -i=dap`），也不提供代理、隧道、端口转发。

### 由插件后端发起调试会话

后端可以让宿主启动一个调试会话：后端决定启动什么，调试界面、进程、控制台、停止按钮和会话结束都归宿主。有两条路径，都按 `litheFunctions` 3 的宿主方法调用，都带 `context: {contextID, generation}`：

| 方法（前缀 `lithe/host/`） | 权限 | 参数 | 结果 |
| --- | --- | --- | --- |
| `debug/start` | `debug.start` | `context, label, adapter, request, configuration` | `{sessionID}` |
| `run/start`，带 `debug` | `run.start` | `context, label, process, debug: {configurationType, target}`，另有 `temporaryFiles?`、`rerun?` | `{runID}` |
| `run/stop` | `run.start` | `{runID}` 或 `{sessionID}` | `null` |

`debug/start`：

- `label`：1 到 256 个字符，不含控制字符。
- `adapter`：本插件自己的适配器，写本地 ID 或限定 ID `<plugin-id>/<local-id>`。别的插件的适配器，以及本插件没有声明的适配器，都返回 `plugin.host.notFound`。
- `request`：`launch` 或 `attach`，适配器必须支持它。
- `configuration`：最多 256 KiB 的 JSON 对象，宿主在适配器声明的默认值之上，把它作为 DAP 的 launch 或 attach 参数传给适配器。`noDebug` 和所有以 `__` 开头的键归宿主，会被拒绝。

`run/start` 的 `debug` 字段让一次宿主拥有的运行成为调试会话：

- `configurationType` 是公开的调试适配器别名，和[按配置类型选择 attach 适配器](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11)相同。
- `target` 为 `{kind: "connection", port}` 时，进程在 `port`（1 到 65535，由后端写进参数）上监听调试器，宿主等它就绪后连接适配器；为 `{kind: "launch"}` 时，适配器自己启动进程，通过 `targetBindings` 从 `process` 取程序、参数、工作目录和环境。

`run/start`、`run/stop` 和 `run/didExit` 的其他规则见[插件后端](backend.md#run-与调试会话协议-3)。参数格式错误返回 `plugin.host.invalidParams`，Core 的拒绝（`plugin.debug.invalidStart` 或 `plugin.run.invalidStart`，带 `field`）在 `details.code` 和 `details.field` 里；已关闭的上下文是 `plugin.host.contextExpired`。macOS 支持；Windows 没有插件后端宿主，也不声明 `debugAdapters`，没有这些方法。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [API level 4: debug adapters](../../contract/v1/README.md#api-level-4-debug-adapters) | 全部字段、绑定、`executable` 候选、控制台、传输 |
| [Adapter features and defaults (`debugAdapters` 3)](../../contract/v1/README.md#adapter-features-and-defaults-debugadapters-3) | `features.hotCodeReplace`、`features.stepFilters` 和 `defaults.stepFilters` |
| [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) | `debug/start`、`run/start` 的 `debug` 字段、`run/stop`：参数、限制、失败码 |
| [Remote attach](../../contract/v1/README.md#remote-attach) | `connection` 绑定、四种路径映射、目标格式、本机判定、拒绝层次 |
| [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | 构建提供者准备原生调试目标 |
| [Registry connection debugging (buildProviders 11)](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | 服务运行按配置类型选择 attach 适配器 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`debug-adapters/debug-adapter.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) | 合法示例：两个适配器（`example`、`native`），`settings` 2 的 `file` 设置选择要调试的程序 |
| [`debug-adapters/debug-adapter-language-server.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-language-server.plugin.json) | `tcpLanguageServer`：调试服务在插件的语言服务器里 |
| [`debug-adapters/debug-adapter-remote-attach.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-remote-attach.plugin.json) | 六个适配器，分别演示 `connection` 的不同写法：`flat`（平铺的 `host`、`port`）、`nested`（嵌套路径加 `objects`）、`scalar`（`first`）、`joined`（`address`）、`keyed`（`dictionary`）、`paired`（`pairs`） |
| [`debug-adapters/debug-adapter-features.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-features.plugin.json) | `debugAdapters` 3 加 `backend` 2、`serverProcess` 1，权限 `debug.launch` 和 `debug.start`：适配器 `script` 声明 `features`（`hotCodeReplace`、`stepFilters`）和 `defaults.stepFilters` |
| [`debug-adapters/debug-start.json`](../../contract/v1/fixtures/debug-adapters/debug-start.json) | `debug/start` 的参数，以及 `debug.decodeBackendConfiguration` 对 10 个用例返回的规范化启动或拒绝：本插件适配器按本地 ID 或限定 ID、别的插件的适配器、未声明的适配器、`noDebug` 和 `__` 键等；256 KiB 上限由 Core 自己的测试覆盖 |
| [`run/run-start.json`](../../contract/v1/fixtures/run/run-start.json) | `run/start` 的参数，以及 `run.decodeStart` 的结果；其中包含 `debug` 的 `connection` 和 `launch` 目标、端口越界等用例 |
| [`debug-adapters/debug-adapter-rejections.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json) | 76 个拒绝用例，每个写明稳定错误码、字段和报告层次（`schema`、`coreManifest`、`corePlan`、`coreInstall`） |
| [`build-providers/build-provider-debug-preparation.plugin.json`](../../contract/v1/fixtures/build-providers/build-provider-debug-preparation.plugin.json) | `buildProviders` 5 加 `debugAdapters` 2 |
| [`build-providers/build-debug-preparation.json`](../../contract/v1/fixtures/build-providers/build-debug-preparation.json) | `lithe/build/prepareDebug` 的结果和拒绝用例 |
| [`build-providers/build-connection-debug-options.json`](../../contract/v1/fixtures/build-providers/build-connection-debug-options.json) | `buildProviders` 11 的服务运行调试选项 |
| [Lithe-IDEA/lithe-plugin-lldb](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | 官方 lldb-dap 调试插件（预发布），使用 `executable` 候选和 `console.commandPrefix` |

### 相关指南

- [编写调试适配器插件（API 4）](../../guides/debug-adapter-plugin.md)
- [原生构建、测试与运行准备](../../guides/native-build-workflows.md)

---

## English

A plugin declares the debug adapters it implements in `debugAdapters[]`. Each adapter is a DAP process the host starts and talks to over stdio or a loopback TCP port, or a debug server built into the plugin's language server (such as the Java Debug Server inside JDT LS). The declaration is data only: there is no resolver, workflow, host integration, or `initializationOptions`, and any such field is refused. The debug UI, breakpoints, variables, and console belong to the host. A plugin backend can also start a debug session on one of its own declared adapters with the host method `debug/start`.

| Item | Value |
| --- | --- |
| Manifest field | `debugAdapters[]` |
| Capability versions | `debugAdapters` 1–3, at level 4; `serverProcess` 1 for `launch.process`, and `serverProcess` 2 for `runSelectedRuntime`. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | DAP (Debug Adapter Protocol); no Lithe-specific protocol. A backend starts a session with the `litheFunctions` 3 host method `debug/start`; see [Starting a debug session from a plugin backend](#starting-a-debug-session-from-a-plugin-backend) |
| API IDs | [API-30](../api-catalog.md#apis-usable-now): debug adapters, including `debug/start` |
| Permissions | `debug.launch`; `debug.attach` when any adapter serves `attach` (remote attach has no separate permission); `debug.start` when a backend calls `debug/start` (needs `backend` 2 and a declared `debugAdapters` entry, and grants only `debug/start`); `tcpLanguageServer` relies on a language server of the same plugin, so it also needs `languageServers` and `languageServer.launch` |
| Activation events | One `onDebug:<id>` per adapter |
| Windows | Not supported; the Windows host does not advertise `debugAdapters` |

### Versions compared

| Version | Lowest level | Adds | macOS | Windows |
| --- | --- | --- | --- | --- |
| `debugAdapters` 1 | 4 | Every `debugAdapters[]` field: `launch`/`attach` requests, automatic `selectors`, `configurationTypes`, default arguments and `targetBindings` (`program`, `processId`, `cwd`, `connection`), `executable` candidates, `capabilities.disabled`, `console`, the four transports | Yes | No |
| `debugAdapters` 2 | 4 | Bounded `args` ← `target.arguments` and `env` ← `target.environment` in `targetBindings` | Yes | No |
| `debugAdapters` 3 | 4 | An adapter may declare `features` (`hotCodeReplace`, `stepFilters`) and `defaults.stepFilters`; any `features` or `defaults` requires `requires.debugAdapters` 3; see [Adapter features](#adapter-features-debugadapters-3) | Yes | No |
| `serverProcess` 2 | 4 | `launch.process` may be `{"launch": {"kind": "runSelectedRuntime", "tool": "python" \| "java" \| "node"}}`, so the host starts the adapter with the runtime the user selected; this is the only use of `serverProcess` 2. From `serverProcess` 4, `tool` is a [tool-definition ID](../../contract/v1/README.md#tool-references-serverprocess-4) that `requiresTools` lists | Core only; the macOS host advertises `serverProcess` 1 and 3 | No |
| `buildProviders` 5 | 4 | A build provider prepares a native debug target, and a run declares `debug: {"adapter": "native"}`; also needs `debugAdapters` 2; see [build providers](build-providers.md) | Yes | No |
| `buildProviders` 11 | 3 | A service run declares `debug: {"configurationType": "…"}` and the host connects an enabled adapter that serves `attach` and that type | Yes | No |

A manifest below level 4 may not declare `debugAdapters`, and a level 3 manifest may not require `debugAdapters` or `serverProcess` 2.

### Transports

| `launch.transport` | Endpoint | Needs `launch.process` |
| --- | --- | --- |
| `stdio` | The adapter's standard input and output | Yes |
| `tcpAnnounced` | The adapter announces its port on an output stream: `endpoint` has `stream`, a one-line `prefix`, and `value` (`port` or `hostPort`) | Yes |
| `tcpPort` | A fixed `endpoint.port` | Yes |
| `tcpLanguageServer` | A language server of the same plugin: `endpoint` has `languageID` and `command`; for each session the host sends `workspace/executeCommand` and the server answers a loopback port | No, and it must not have one |

Every TCP endpoint is loopback only. The `tcpLanguageServer` command is part of the install and update review, and changing it asks the user again.

### Adapter features (`debugAdapters` 3)

A host never recognizes an adapter by its ID. What is specific to an adapter is declared, and the shared debug engine follows the declaration:

| Declaration | Meaning |
| --- | --- |
| `features.hotCodeReplace.request` | The DAP command the host sends to apply changed code to the running debuggee (the standard `redefineClasses` inspection). Present means the adapter supports hot code replace; absent means it does not |
| `features.hotCodeReplace.event` | A DAP event whose `message` the Debug Console shows as progress |
| `features.hotCodeReplace.changedField`, `errorField` | Fields of the response body: the list of replaced units, and a failure message the adapter reports in a successful response. A response without `changedField` replaces nothing visible |
| `features.stepFilters.argument` | The launch argument through which the adapter receives stepping filters, shaped `{skipClasses, skipSynthetics, skipStaticInitializers, skipConstructors}`. An argument the user's configuration already sets is kept |
| `defaults.stepFilters` | `classNameFilters` (up to 256 single-line patterns of at most 256 characters), `skipSynthetics`, `skipStaticInitializers`, `skipConstructors`, `hideFilteredStackFrames`. Used when neither the launch nor the user's settings name filters |

Names (`request`, `event`, fields, `argument`) are a letter followed by up to 63 letters, digits, `.`, `_`, or `-`. A declaration with a malformed name, or a field this version does not define, is `plugin.manifest.invalid` with the `field` that names it, and any `features` or `defaults` requires `requires.debugAdapters` 3. The launch plan's `debugAdapter` descriptor carries both so the host can pass them to `debug.createSession`. A plugin that declares neither keeps the behavior of `debugAdapters` 1 and 2.

### Remote attach

When the debuggee runs on another machine (a JVM with `-agentlib:jdwp`, a Node inspector, gdbserver), the adapter still runs locally and only the debuggee is remote. The `targetBindings.connection` of an `attach` configuration writes the host, port, and path mappings the user confirmed into the adapter's `arguments`:

| Field | Notes |
| --- | --- |
| `host` + `port` | Declared together; values are paths into `arguments` (1–4 segments, such as `connect.host`) |
| `address` | One `host:port` string, IPv6 in brackets; replaces `host` + `port` or sits beside them |
| `pathMappings.shape: "objects"` | A list of objects; `localKey` and `remoteKey` name each object's keys |
| `pathMappings.shape: "first"` | For an adapter with a single pair of roots; a target with more is refused, not truncated |
| `pathMappings.shape: "dictionary"` | One object; `keyedBy` chooses which side is the key |
| `pathMappings.shape: "pairs"` | A list of two-element arrays; `order` is `remoteLocal` or `localRemote` |

Core treats only `localhost`, `127.0.0.0/8`, and `::1` as local; every other host is `remote: true`, and the host asks the user to confirm it each time without remembering the answer by default. `connection` declares no external DAP endpoint or remote adapter execution (such as `ssh host gdb -i=dap`), and no proxy, tunnel, or port forwarding.

### Starting a debug session from a plugin backend

A backend can ask the host to start a debug session: the backend decides what to start, and the debug UI, the process, the console, the Stop button, and the end of the session belong to the host. There are two paths. Both are `litheFunctions` 3 host methods and both take `context: {contextID, generation}`:

| Method (prefix `lithe/host/`) | Permission | Params | Result |
| --- | --- | --- | --- |
| `debug/start` | `debug.start` | `context, label, adapter, request, configuration` | `{sessionID}` |
| `run/start` with `debug` | `run.start` | `context, label, process, debug: {configurationType, target}`, plus `temporaryFiles?` and `rerun?` | `{runID}` |
| `run/stop` | `run.start` | `{runID}` or `{sessionID}` | `null` |

`debug/start`:

- `label`: 1 to 256 characters without control characters.
- `adapter`: one of the plugin's own adapters, by its local ID or its qualified ID `<plugin-id>/<local-id>`. An adapter of another plugin, and one the plugin does not declare, both return `plugin.host.notFound`.
- `request`: `launch` or `attach`, and the adapter must serve it.
- `configuration`: a JSON object of at most 256 KiB that the host passes to the adapter as the DAP launch or attach arguments, on top of the adapter's declared defaults. `noDebug` and every key that starts with `__` belong to the host and are refused.

The `debug` field of `run/start` makes a host-owned run a debug session:

- `configurationType` is a public debug adapter alias, as in [registry connection debugging](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11).
- With `target` `{kind: "connection", port}`, the process listens for a debugger on `port` (1 to 65535; the backend puts it into the arguments), and the host waits for it and attaches the adapter. With `{kind: "launch"}`, the adapter starts the process itself, taking program, arguments, directory, and environment from `process` through its `targetBindings`.

The other rules of `run/start`, `run/stop`, and `run/didExit` are in [Plugin backend](backend.md#run-and-debug-sessions-protocol-3). Malformed params are `plugin.host.invalidParams`, and Core's rejection (`plugin.debug.invalidStart` or `plugin.run.invalidStart`, with the `field`) is in `details.code` and `details.field`; a closed context is `plugin.host.contextExpired`. macOS supports them; Windows has no plugin backend host and does not advertise `debugAdapters`, so it has none of these methods.

### Contract sections

| Section | Content |
| --- | --- |
| [API level 4: debug adapters](../../contract/v1/README.md#api-level-4-debug-adapters) | Every field, bindings, `executable` candidates, console, transports |
| [Adapter features and defaults (`debugAdapters` 3)](../../contract/v1/README.md#adapter-features-and-defaults-debugadapters-3) | `features.hotCodeReplace`, `features.stepFilters`, and `defaults.stepFilters` |
| [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) | `debug/start`, the `debug` field of `run/start`, and `run/stop`: params, limits, failure codes |
| [Remote attach](../../contract/v1/README.md#remote-attach) | The `connection` binding, the four mapping shapes, the target format, local detection, refusal layers |
| [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | Build providers preparing native debug targets |
| [Registry connection debugging (buildProviders 11)](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | Service runs choosing an attach adapter by configuration type |

### Examples and cases

| File | Content |
| --- | --- |
| [`debug-adapters/debug-adapter.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) | Valid example: two adapters (`example`, `native`), and a `settings` 2 `file` setting picks the program to debug |
| [`debug-adapters/debug-adapter-language-server.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-language-server.plugin.json) | `tcpLanguageServer`: the debug server lives in the plugin's language server |
| [`debug-adapters/debug-adapter-remote-attach.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-remote-attach.plugin.json) | Six adapters, one per way of taking a `connection`: `flat` (top-level `host` and `port`), `nested` (nested paths with `objects`), `scalar` (`first`), `joined` (`address`), `keyed` (`dictionary`), `paired` (`pairs`) |
| [`debug-adapters/debug-adapter-features.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-features.plugin.json) | `debugAdapters` 3 with `backend` 2 and `serverProcess` 1, and the permissions `debug.launch` and `debug.start`: the adapter `script` declares `features` (`hotCodeReplace`, `stepFilters`) and `defaults.stepFilters` |
| [`debug-adapters/debug-start.json`](../../contract/v1/fixtures/debug-adapters/debug-start.json) | The params of `debug/start` and the normalized start or refusal `debug.decodeBackendConfiguration` returns for each of its 10 cases: the plugin's own adapter by local or qualified ID, an adapter of another plugin, an undeclared adapter, `noDebug`, `__` keys, and more; the 256 KiB limit is covered by Core's own tests |
| [`run/run-start.json`](../../contract/v1/fixtures/run/run-start.json) | The params of `run/start` and the `run.decodeStart` results; its cases include a `debug` `connection` target, a `launch` target, and a port out of range |
| [`debug-adapters/debug-adapter-rejections.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json) | 76 refusal cases, each with its stable code, field, and reporting layer (`schema`, `coreManifest`, `corePlan`, `coreInstall`) |
| [`build-providers/build-provider-debug-preparation.plugin.json`](../../contract/v1/fixtures/build-providers/build-provider-debug-preparation.plugin.json) | `buildProviders` 5 with `debugAdapters` 2 |
| [`build-providers/build-debug-preparation.json`](../../contract/v1/fixtures/build-providers/build-debug-preparation.json) | `lithe/build/prepareDebug` results and refusal cases |
| [`build-providers/build-connection-debug-options.json`](../../contract/v1/fixtures/build-providers/build-connection-debug-options.json) | `buildProviders` 11 service-run debug options |
| [Lithe-IDEA/lithe-plugin-lldb](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | The official lldb-dap debug plugin (prerelease), using `executable` candidates and `console.commandPrefix` |

### Related guides

- [Writing a debug adapter plugin (API 4)](../../guides/debug-adapter-plugin.md)
- [Native build, test, and run preparation](../../guides/native-build-workflows.md)
