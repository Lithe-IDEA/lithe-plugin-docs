# 语言、语言服务器与运行/测试 / Languages, language servers, run and test

[中文](#中文) · [English](#english)

## 中文

让 Lithe 认识一种语言：识别文件，用包内 Monarch 语法高亮，启动 LSP 语言服务器，提供“运行当前文件”和测试。全部是数据声明，宿主负责进程生命周期、LSP 会话和停止进程，插件不需要写宿主代码。命令行要到启动时才能确定、或后端需要驱动自己的服务器时，可以让插件后端回答启动命令，并在自己声明的范围内向服务器发请求（见下文“后端决定启动方式”和“插件自有会话”）。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `languages`、`languageServers`、`executionProviders`、`outputLinkMatchers` |
| 能力版本 | `languages` 1–2、`languageServers` 1–3、`serverProcess` 1–4、`executionProviders` 1、`outputLinkMatchers` 1，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | LSP；所有插件进程都可以协商 [`litheLogging` 1](../capabilities-and-versions.md#进程协议版本)。后端驱动服务器用的 `languageServer/*` 宿主方法在函数协议 3 里，见[插件后端](backend.md) |
| API 编号 | [API-25](../api-catalog.md#现在能用的-api)：语言、语法、LSP。[API-12](../api-catalog.md#现在能用的-api)：按 URI scheme 提供的只读文档（`documents/registerContentProvider`）。[API-20](../api-catalog.md#现在能用的-api)：语言服务器按工作区的状态目录及其回收。[API-22](../api-catalog.md#现在能用的-api)：语言服务器的准备状态（`languageServer/setStatus`）和 Run、测试控制台输出里的链接（`outputLinkMatchers`）。插件函数提供的编辑器语言功能见[编辑器](editor.md)的 `editor/registerProviders` |
| 权限 | `languageServer.launch`；声明 `run`、`test` 时加 `process.run`、`process.test`；`languageServer.request`（`languageServers` 3 的服务器列出 `requestMethods` 或 `forwardNotifications` 时才可声明，需要 `backend` 2）；`documents.contentProvider`（后端注册只读文档内容提供者，需要 `backend` 2） |
| 激活事件 | `onLanguage:<id>`、`onRun:<id>`、`onTest:<id>` |
| macOS | `languageServers` 3 的会话、`serverProcess` 3 的后端决定启动方式和 `outputLinkMatchers` 只在 macOS 实现；后端决定启动方式的服务器不在远程工作区启动（远程后端启动未支持）。宿主目前只声明 `serverProcess` 1 和 3：版本 4 已由 Core 校验，宿主尚未声明，需要它的插件版本暂时装不上 |
| Windows | 级别 3 清单按宿主能力表准入：`languages` 1、`languageServers` 1–2、`serverProcess` 1。不支持 `languages` 2 的语法，也不声明 `executionProviders`、`backend`、`outputLinkMatchers`，因此没有 `languageServers` 3 的会话、`serverProcess` 3–4 和输出链接 |

### 各级别的变化

| 版本 | 新增 |
| --- | --- |
| 级别 1 | 语言、`packaged` 和 `systemCommand` 语言服务器、`node` 和 `go` 运行时依赖、运行和测试提供者 |
| 级别 2 | `runtime` 启动（包内运行时或宿主发现的运行时）、`java` 运行时、启动模板、`initializationOptions`、宿主集成（所有插件都可以声明） |
| `languages` 2 | 包内 Monarch 语法 `grammar` 与编辑器配置 `configuration`；只有语法的插件不需要任何进程 |
| `languageServers` 2 | `workspaceConfiguration`：Core 按设置展开后回答 `workspace/configuration`，`immediate` 设置变化时不重启即可推送 |
| `languageServers` 3 | 插件自有会话字段 `requestMethods`、`commands`、`serializedCommands`、`forwardNotifications`、`state`、`remote`：后端只能通过服务器声明的内容驱动它；会话策略由 Core 执行 |
| `serverProcess` 2 | `runSelectedRuntime`，只给调试适配器用，见[调试适配器](debug-adapters.md) |
| `serverProcess` 3 | 语言服务器的 `launch.kind: "backend"`：命令行由插件后端在每次启动时回答；需要 `backend` 2 和不低于 4 的 `pluginAPIVersion` |
| `serverProcess` 4 | 运行时按工具定义 ID 引用（顶层 `requiresTools`），不再用宿主的封闭运行时列表，见[工具链](toolchains.md) |
| `outputLinkMatchers` 1 | 清单顶层 `outputLinkMatchers[]`：插件声明匹配规则，让 Run 和测试控制台的输出出现链接，见下文“输出链接” |

### 宿主集成（API 2）

语言服务器的 `hostIntegration` 对所有插件开放，不限命名空间。集成是宿主代码，但它只读取声明插件自己包里的资源，交给同一个插件启动的语言服务器或辅助进程，除了运行插件自己的服务器进程（安装确认时用户已同意）之外不授予任何权限。契约目前只列出 `jdt`；宿主没有实现的 ID 返回 `plugin.integration.unsupported`。构建系统的 `hostIntegration` 是另一个声明，仍只限官方插件，见[构建系统](build-systems.md)。

### 后端决定启动方式（`serverProcess` 3）

服务器的命令行取决于启动时才知道的东西（解析出的版本、引导出来的工具、从工程推导的选项）时，不写死 `launch`，让插件后端每次回答：

```json
"launch": {"kind": "backend", "function": "launch.resolve", "timeoutMilliseconds": 30000}
```

| 项目 | 规则 |
| --- | --- |
| 适用 | 只有语言服务器能用。`function` 是插件函数 ID；`timeoutMilliseconds` 为 1000–120000，默认 30000 |
| 其他字段 | 服务器不能再声明 `arguments`、`validationArguments`、`runtimeDependencies`、`initializationOptions` 和 `hostIntegration`：前四项由后端的应答携带，宿主集成是宿主代码，正是这种启动方式要避免的。`workspaceConfiguration` 仍可声明式给出，除非应答自带 |
| 启动计划 | `plugins.declarative.languageServerLaunchPlan` 返回第一步而不是命令行：`launch: {kind: "backend", function, timeoutMilliseconds}`、`resolveParams`、`requiredDirectories`（固定为 `state`、`workspaceState`、`log`、`temporary`）、`settings`、`sessionPolicy`。请求要带绝对路径的 `temporaryDirectory`，否则是 `plugin.launch.invalidRequest` |
| 调用 | 宿主建好四个目录，必要时启动插件后端，用 `lithe/function/invoke` 调用 `function`，参数是 `resolveParams`（`serverID`、`platform`、四个目录、`settings`）和工作区 `context`。调用失败、超时，或工作区代次已变，这次启动就不产生进程 |
| 应答 | `executable`（`{kind: "packaged", path}` 是包内只读文件；`{kind: "tool", tool}` 是插件自己定义的工具，由宿主按工具解析）、`arguments`、`environment`、`initializationOptions`、`workspaceConfiguration`、`gates`（最多 8 个）、`writablePaths`（最多 32 个，都在四个目录之内）；不认识的字段会被拒绝 |
| 解码 | 宿主把 `{manifest, languageID, request, result}` 交给 `plugins.languageServers.decodeLaunch`，得到普通的启动计划（`launch` 为 `packaged` 或 `tool`），`sessionPolicy` 另带应答的 `gates` |
| 拒绝码 | `plugin.launch.invalidResult`、`plugin.launch.outsidePackage`、`plugin.launch.undeclaredTool`、`plugin.launch.unwritablePath`、`plugin.launch.invalidRequest`、`plugin.launch.unavailable`、`plugin.launch.notBackend` |
| macOS 现状 | `{kind: "tool"}` 应答按用户的工具选择和发现解析，位于项目内的程序会被拒绝（打开项目不能运行它）；不在远程工作区启动；临时目录在服务器进程结束时删除 |

### 插件自有会话（`languageServers` 3）

后端只能通过服务器声明的内容驱动它自己的服务器。`languageServers` 3 的服务器可以声明：

| 字段 | 含义 |
| --- | --- |
| `requestMethods` | 后端可以用 `languageServer/request`、`languageServer/notify` 发送的 LSP 方法，最多 64 个 |
| `commands` | 后端可以运行的 `workspace/executeCommand` 命令，最多 128 个；恰好在 `requestMethods` 列出 `workspace/executeCommand` 时声明 |
| `serializedCommands` | `commands` 的子集，一次只运行一个：同一命令已在运行时，新请求并入它，得到同一个答案 |
| `forwardNotifications` | 宿主转发给后端的服务器通知，最多 32 个 |
| `state` | `{scope: "workspace", retention: {maxEntries, maxAgeDays}}`，`maxEntries` 为 1–64，`maxAgeDays` 为 1–365 |
| `remote` | `supported`（默认）或 `unsupported`；`unsupported` 的服务器不为远程工作区启动 |

- 名称是不含空白和控制字符、最多 256 字节的非空文本，在各自列表里唯一。宿主自己发送的方法（`initialize`、`initialized`、`shutdown`、`exit`、`textDocument/didOpen` 等文档同步方法、`workspace/didChangeWorkspaceFolders` 和所有 `$/` 方法）不能出现在 `requestMethods` 或 `forwardNotifications` 里。
- 低于 `languageServers` 3 声明这些字段是 `plugin.manifest.invalid`。服务器列出 `requestMethods` 或 `forwardNotifications` 时必须声明 `languageServer.request`；没有这样的服务器却声明该权限，也是 `plugin.manifest.invalid`。
- 启动计划的 `sessionPolicy` 就是这些字段（排序后的列表）加 `serverID`（即 `languageID`）和 `gates`。宿主原样交给 `lsp.startServer.sessionPolicy`，由 Core 的 LSP 引擎执行：宿主不能放宽，后端不能越界。
- 请求必须在 `requestMethods` 里，`workspace/executeCommand` 的 `params.command` 必须在 `commands` 里，否则是 `plugin.host.methodNotDeclared`。通知可以是 `workspace/didChangeConfiguration`，或 `requestMethods` 里除 `workspace/executeCommand` 以外的方法；宿主自己的方法仍然被拒绝。
- 会话必须属于调用方插件，并且正在运行；请求等待最多 `timeoutMilliseconds`（默认 60000，最大 300000），取消时向服务器发 `$/cancelRequest`。服务器的 JSON-RPC 错误不转发 `data`。
- **gate** 是 `{id, until: "notification", method, match}`，由启动应答声明：服务器发出该 `method` 的通知、且参数包含 `match` 的所有成员时才打开。所有 gate 打开之前，`request` 按顺序等待，最多 128 个，第 129 个是 `plugin.host.busy`；通知从不被挡住。
- 服务器停止、崩溃、重启或插件被停用时，所有未完成的请求以 `plugin.host.languageServerUnavailable` 结束；后端在 `didChangeState` 报告新的 `generation` 的 `running` 之后再请求。

**状态目录与回收（API-20）。** 宿主在插件的存储目录下（不在插件包或应用包里）为每个服务器、每个工作区保留一个状态目录，并把它作为 `workspaceStateDirectory` 交给服务器；停用和更新插件都不会删除。目录内的子目录名由插件决定，每个工作区目录本身的名字由宿主决定，宿主从不查看里面的内容。宿主每次启动服务器时刷新该目录的修改时间，并在启动后于后台回收：先用 Core 的 `plugins.languageServers.stateRetention` 决定要删除的目录，即超过 `maxAgeDays` 的，以及其余目录里扣除打开的工作区占用的名额后只保留最新 `maxEntries` 个以外的。打开的工作区从不被回收，没有声明 `state.retention` 的插件保留全部。这一回收仅 macOS。

### 宿主方法与通知

后端用这些方法和通知与自己的语言服务器交谈；方法名省略前缀 `lithe/host/`。`languageServer/*` 的每个方法都带 `context: {contextID, generation}` 和 `serverID`（本插件某个服务器的 `languageID`），别的插件的服务器返回 `plugin.host.foreignContribution`。

| API | 方法 | 用途 | 权限 |
| --- | --- | --- | --- |
| API-25 | `languageServer/request` | 向本插件自己的语言服务器发 JSON-RPC 请求，参数加 `method`、`params?`、`timeoutMilliseconds?`（1–300000，默认 60000），返回 `{value}`（服务器的 `result`，可以是 `null`）；受会话策略检查 | `languageServer.request` |
| API-25 | `languageServer/notify` | 向本插件自己的语言服务器发通知，参数加 `method`、`params?`，返回 `{delivered}`；只有服务器初始化前发出的 `workspace/didChangeConfiguration` 是 `false`，这时值会随 `initialized` 送出 | `languageServer.request` |
| API-22、API-25 | `languageServer/setStatus` | 在宿主状态界面显示插件自己的准备状态（导入工程、建索引）：`state` 为 `preparing`、`ready`、`degraded` 或 `failed`，可带 `message?`（原样显示，截到 1024 字节）和 `percentage?`（0–100 的整数），返回 `{}`；服务器停止时宿主移除它。它不同于只属于正在运行的调用的 `lithe/function/progress` | `languageServer.request` |
| API-12 | `documents/registerContentProvider` | 把一个 URI scheme 绑定到后端函数：打开该 scheme 的文档时，宿主调用函数取只读内容（不超过 1 MiB，宿主等 30 秒）；每个工作区窗口每个 scheme 只能有一个提供者，返回 `{subscriptionID}`。不带 `serverID`，用 `subscriptions/dispose` 撤销 | `documents.contentProvider` |

| API | 通知 | 含义 | 权限 |
| --- | --- | --- | --- |
| API-25 | `languageServer/didChangeState` | `{context, serverID, state, generation}`；`state` 为 `starting`、`running`、`stopped` 或 `crashed` | `languageServer.request` |
| API-25 | `languageServer/didReceiveNotification` | `{context, serverID, generation, method, params?, truncated?}`：服务器发出且 `forwardNotifications` 列出的通知；超过 `limits.notificationBytes` 时只送一次 `truncated: true`，不带 `params` | `languageServer.request` |

`generation` 是宿主每次为这个上下文启动服务器时递增的正整数，后端据此分辨状态或通知属于哪个进程；新进程启动后，宿主不再送旧进程的通知。

这些方法的错误码：

| 错误码 | JSON-RPC | 含义 |
| --- | --- | --- |
| `plugin.host.methodNotDeclared` | `-32024` | 方法或 `workspace/executeCommand` 命令不在服务器声明的会话里 |
| `plugin.host.languageServerUnavailable` | `-32022` | 服务器没有为这个上下文运行，或请求进行中服务器已停止 |
| `plugin.host.languageServerError` | `-32023` | 服务器以 JSON-RPC 错误应答；`details` 是 `{code, message}` |
| `plugin.host.foreignContribution` | `-32026` | 指名的服务器属于别的插件 |
| `plugin.host.timeout` | `-32015` | 超过宿主的截止时间 |
| `plugin.host.cancelled` | `-32800` | 请求被取消 |
| `plugin.host.busy` | `-32020` | 第 129 个等待中的请求（见 gate） |
| `plugin.host.limitExceeded` | `-32014` | 结果超过宿主的上限 |
| `plugin.host.preconditionFailed` | `-32019` | `documents/registerContentProvider`：这个 scheme 已有提供者 |

### 输出链接（`outputLinkMatchers` 1）

`outputLinkMatchers[]` 让插件把 Run 和测试控制台里的文字变成可点击的链接，例如回溯里的一帧或 panic 的位置。宿主不认识任何工具的输出格式：插件声明匹配规则，Core 找出匹配。需要 `requires.outputLinkMatchers` 1。只声明匹配器本身就是完整的插件；不带 `resolve` 的匹配器不需要后端。

```json
"outputLinkMatchers": [
  { "id": "traceback", "pattern": "File \"(?P<path>[^\"]+)\", line (?P<line>\\d+)", "scope": "run" }
]
```

| 字段 | 含义 |
| --- | --- |
| `id` | 插件内的 ID：小写字母开头，后面最多 63 个小写字母、数字、`.`、`_` 或 `-`，在插件内唯一。每个插件最多 16 个匹配器 |
| `pattern` | 对控制台的一行做匹配的正则，1–512 字节，使用 Rust `regex` 语法，所以没有反向引用和环视，编译后的大小也有上限。最多命名 8 个分组，每个是 camelCase 名称，且不能匹配空文本 |
| `scope` | `run` 或 `test`：匹配器作用的控制台。缺省表示两者 |
| `resolve.function` | 可选，插件 `backend` 2 进程里的函数 ID；需要 `backend` 2 |

- 不带 `resolve` 时，规则必须捕获 `path` 分组，可以捕获 `line` 和 `column`（从 1 开始），不接受其他分组名。相对 `path` 对每个工作区根生成一个候选，绝对路径只在某个工作区根之内有效，离开所有根的路径不生成链接：控制台里的一行不可能打开工作区之外的文件。
- 带 `resolve` 时，规则可以命名任意分组。用户点击时宿主调用该函数，参数是 `{groups}`（按名称的捕获文本）和窗口的 `context`，后端返回 `{uri, range?}` 或 `null`。
- 匹配由 Core 决定，所有宿主链接同样的文字：`outputLinks.match` 接收 `{scope, platform, caseSensitive?, workspaceRoots, sources: [{pluginID, matchers}], lines}`，返回 `{links}`。每个链接是 `{line, start, length, pluginID, matcherID}` 加 `location`（`{paths, line?, column?}`，宿主打开第一个存在的路径）或 `resolve`（`{function, groups}`）。`start` 和 `length` 按该行的 UTF-16 代码单元计数。按来源顺序、再按声明顺序找匹配，与前面链接重叠的匹配被跳过。一次请求最多 256 行、64 个来源、16 个根；每行最多扫描 4096 字节，最多 8 个链接。
- `outputLinks.decodeResolution` 接收 `{reply}`（函数的应答），返回要打开的位置 `{path, line?, column?}`（从 1 开始）或 `null`。应答只会打开 `file:` URI，只接受 `uri` 和 `range` 两个成员；从 0 开始的 `range.start` 会被转换。
- 宿主只对显示出来的行做匹配，不对没有启用该插件的输出做匹配，并忽略文件不存在的链接。只在 macOS 实现（API-22）。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Languages](../../contract/v1/README.md#languages) | 语言 ID、扩展名、文件名、项目标记、依赖路径 |
| [Grammars and editor configuration (`languages` 2)](../../contract/v1/README.md#grammars-and-editor-configuration-languages-2) | Monarch 语法的上限和拒绝规则、注释与括号配置、只在可终止的 worker 中运行 |
| [Language servers](../../contract/v1/README.md#language-servers) | 启动方式（含 `backend`）、参数、探测参数、运行时依赖、会话字段 |
| [Runtime launch (API 2)](../../contract/v1/README.md#runtime-launch-api-2) | 用 `java` 或 `node` 启动包内入口文件，可以自带运行时 |
| [Tool references (`serverProcess` 4)](../../contract/v1/README.md#tool-references-serverprocess-4) | 按工具定义 ID 引用运行时、启动计划里的 `tool` 启动方式 |
| [Host integrations (API 2)](../../contract/v1/README.md#host-integrations-api-2) | 数据表达不了的行为由宿主代码实现，所有插件都可以按名称声明；宿主没有实现的 ID 返回 `plugin.integration.unsupported`。构建系统的宿主集成仍只限官方插件 |
| [Launch templates (API 2)](../../contract/v1/README.md#launch-templates-api-2) | `${pluginDir}`、`${stateDir}`、`${workspaceStateDir}` 等模板变量 |
| [Workspace configuration (`languageServers` 2)](../../contract/v1/README.md#workspace-configuration-languageservers-2) | `workspaceConfiguration` 的展开与推送 |
| [Backend-resolved launch (`serverProcess` 3)](../../contract/v1/README.md#backend-resolved-launch-serverprocess-3) | 后端回答命令行：`resolveParams`、应答字段、拒绝码、macOS 现状 |
| [Plugin-owned sessions (`languageServers` 3)](../../contract/v1/README.md#plugin-owned-sessions-languageservers-3) | 会话字段、会话策略、gate、状态目录与回收、准备状态 |
| [Language server host methods](../../contract/v1/README.md#language-server-host-methods) | `languageServer/request`、`notify`、`setStatus` 和两个通知 |
| [Content providers](../../contract/v1/README.md#content-providers) | `documents/registerContentProvider`：按 URI scheme 提供只读文档 |
| [Output link matchers (`outputLinkMatchers` 1)](../../contract/v1/README.md#output-link-matchers-outputlinkmatchers-1) | Run 与测试控制台输出里的链接、`outputLinks.match`、`outputLinks.decodeResolution` |
| [Execution providers](../../contract/v1/README.md#execution-providers) | `run`、`test` 的可执行文件、参数模板、测试发现 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`languages/language-packaged-server.plugin.json`](../../contract/v1/fixtures/languages/language-packaged-server.plugin.json) | 级别 1：包内语言服务器加运行/测试提供者（PHP） |
| [`languages/language-system-command.plugin.json`](../../contract/v1/fixtures/languages/language-system-command.plugin.json) | 级别 1：用系统命令启动的语言服务器加运行/测试提供者（Go） |
| [`languages/node-runtime.plugin.json`](../../contract/v1/fixtures/languages/node-runtime.plugin.json) | 级别 2：用 `node` 运行时启动，带插件设置 |
| [`languages/language-host-integration.plugin.json`](../../contract/v1/fixtures/languages/language-host-integration.plugin.json) | 级别 2：声明宿主集成的语言服务器（Java） |
| [`languages/language-grammar.plugin.json`](../../contract/v1/fixtures/languages/language-grammar.plugin.json) | 级别 3、`languages` 2：只有语法、不启动进程的插件 |
| [`languages/language-server-backend-launch.plugin.json`](../../contract/v1/fixtures/languages/language-server-backend-launch.plugin.json) | 级别 3、`serverProcess` 3 与 `languageServers` 3：命令行由后端回答、声明会话字段和 `languageServer.request` 的语言服务器 |
| [`languages/grammars/http.monarch.json`](../../contract/v1/fixtures/languages/grammars/http.monarch.json) | 完整的 Monarch 语法示例 |
| [`languages/command-plans.json`](../../contract/v1/fixtures/languages/command-plans.json) | 35 个标准答案：运行/测试模板和语言服务器启动计划的展开结果 |
| [`languages/runtime-versions.json`](../../contract/v1/fixtures/languages/runtime-versions.json) | 12 个标准答案：`plugins.declarative.checkRuntimeVersion` 的运行时版本判断 |
| [`languages/language-server-launch-resolution.json`](../../contract/v1/fixtures/languages/language-server-launch-resolution.json) | 17 个标准答案：后端解析启动的第一步（`languageServerLaunchPlan`）和解码后端应答（`plugins.languageServers.decodeLaunch`）得到的启动计划或拒绝码 |
| [`run/output-link-matchers.json`](../../contract/v1/fixtures/run/output-link-matchers.json) | `outputLinks.match` 的请求与返回的链接，以及匹配器 `resolve` 函数的回复经 `outputLinks.decodeResolution` 得到的结果 |
| [`run/tool-reference-launch.plugin.json`](../../contract/v1/fixtures/run/tool-reference-launch.plugin.json) | 语言服务器的 `runtime` 启动按工具 ID `interpreter` 引用运行时（`serverProcess` 4），并带一个 `outputLinkMatchers` 条目 |

语言相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。

### 相关指南

- [从零构建一个完整生命周期的 LSP 插件](../../guides/lsp-plugin-lifecycle.md)
- [为语言插件接入构建、运行与测试](../../guides/build-tool-integration.md)

---

## English

Teach Lithe a language: recognize its files, highlight them with a packaged Monarch grammar, start an LSP language server, and offer "run current file" and tests. Everything is declared as data. The host owns the process lifecycle, the LSP session, and stopping the process; the plugin writes no host code. When the command line is only known at launch time, or the backend needs to drive its own server, the plugin backend can answer the launch command and send requests to the server within what the plugin declares (see "Backend-resolved launch" and "Plugin-owned sessions" below).

| Item | Value |
| --- | --- |
| Manifest fields | `languages`, `languageServers`, `executionProviders`, `outputLinkMatchers` |
| Capability versions | `languages` 1–2, `languageServers` 1–3, `serverProcess` 1–4, `executionProviders` 1, `outputLinkMatchers` 1; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | LSP; every plugin process may negotiate [`litheLogging` 1](../capabilities-and-versions.md#process-protocol-versions). The `languageServer/*` host methods a backend uses to drive a server are in function protocol 3; see [Plugin backend](backend.md) |
| API IDs | [API-25](../api-catalog.md#apis-usable-now): languages, grammars, LSP. [API-12](../api-catalog.md#apis-usable-now): read-only documents served by URI scheme (`documents/registerContentProvider`). [API-20](../api-catalog.md#apis-usable-now): per-workspace language-server state directories and their pruning. [API-22](../api-catalog.md#apis-usable-now): language-server preparation status (`languageServer/setStatus`) and links in Run and Test console output (`outputLinkMatchers`). Editor language features provided by plugin functions are covered by `editor/registerProviders` in [Editor](editor.md) |
| Permissions | `languageServer.launch`; `process.run` and `process.test` when `run` or `test` is declared; `languageServer.request` (declarable only when a `languageServers` 3 server lists `requestMethods` or `forwardNotifications`; needs `backend` 2); `documents.contentProvider` (the backend registers a read-only document content provider; needs `backend` 2) |
| Activation events | `onLanguage:<id>`, `onRun:<id>`, `onTest:<id>` |
| macOS | The `languageServers` 3 sessions, the `serverProcess` 3 backend-resolved launch, and `outputLinkMatchers` are implemented on macOS only; a server whose launch the backend resolves does not start in a remote workspace (remote backend launch is unsupported). The host currently advertises only `serverProcess` 1 and 3: version 4 is validated by Core but the host does not advertise it yet, so a plugin version that needs it cannot be installed for now |
| Windows | Level 3 manifests are admitted by the host table: `languages` 1, `languageServers` 1–2, `serverProcess` 1. No `languages` 2 grammars, and none of `executionProviders`, `backend`, or `outputLinkMatchers` is advertised, so there are no `languageServers` 3 sessions, no `serverProcess` 3–4, and no output links |

### What each version adds

| Version | Adds |
| --- | --- |
| Level 1 | Languages, `packaged` and `systemCommand` language servers, `node` and `go` runtime dependencies, run and test providers |
| Level 2 | `runtime` launches (packaged or host-discovered runtime), the `java` runtime, launch templates, `initializationOptions`, host integrations (any plugin may declare one) |
| `languages` 2 | Packaged Monarch `grammar` and editor `configuration`; a grammar-only plugin needs no process |
| `languageServers` 2 | `workspaceConfiguration`: Core expands it from settings, answers `workspace/configuration`, and pushes `immediate` setting changes without a restart |
| `languageServers` 3 | Plugin-owned session fields `requestMethods`, `commands`, `serializedCommands`, `forwardNotifications`, `state`, `remote`: the backend drives its server only through what the server declares; Core enforces the session policy |
| `serverProcess` 2 | `runSelectedRuntime`, used only by debug adapters; see [Debug adapters](debug-adapters.md) |
| `serverProcess` 3 | A language server's `launch.kind: "backend"`: the plugin backend answers the command line at each start; needs `backend` 2 and a `pluginAPIVersion` of at least 4 |
| `serverProcess` 4 | Runtimes are referenced by tool-definition ID (top-level `requiresTools`) instead of the host's closed runtime lists; see [Toolchains](toolchains.md) |
| `outputLinkMatchers` 1 | Top-level `outputLinkMatchers[]`: a plugin declares patterns that turn Run and Test console output into links; see "Output links" below |

### Host integrations (API 2)

A language server's `hostIntegration` is open to every plugin, whatever its namespace. An integration is host code, but it reads only resources inside the declaring plugin's own package and hands them to the language server or helper process that same plugin starts; it grants no permission beyond running the plugin's own server process, which the user already agreed to at the install review. The contract lists only `jdt` today; an ID the host does not implement returns `plugin.integration.unsupported`. A build system's `hostIntegration` is a different declaration and stays limited to official plugins; see [Build systems](build-systems.md).

### Backend-resolved launch (`serverProcess` 3)

When a server's command line depends on something only known at launch time (a version the plugin resolves, a tool it bootstraps, options derived from the project), do not hard-code `launch`. Let the plugin backend answer each time:

```json
"launch": {"kind": "backend", "function": "launch.resolve", "timeoutMilliseconds": 30000}
```

| Item | Rule |
| --- | --- |
| Applies to | Language servers only. `function` is a plugin function ID; `timeoutMilliseconds` is 1000–120000, default 30000 |
| Other fields | The server cannot also declare `arguments`, `validationArguments`, `runtimeDependencies`, `initializationOptions`, or `hostIntegration`: the answer carries the first four, and a host integration is host code, which this launch exists to avoid. `workspaceConfiguration` can stay declarative unless the answer carries its own |
| Launch plan | `plugins.declarative.languageServerLaunchPlan` returns a first step, not a command line: `launch: {kind: "backend", function, timeoutMilliseconds}`, `resolveParams`, `requiredDirectories` (always `state`, `workspaceState`, `log`, and `temporary`), `settings`, and `sessionPolicy`. The request must carry an absolute `temporaryDirectory`, otherwise the result is `plugin.launch.invalidRequest` |
| Call | The host creates the four directories, starts the plugin backend if needed, and calls `function` through `lithe/function/invoke` with `resolveParams` (`serverID`, `platform`, the four directories, `settings`) and the workspace `context`. A failed or timed-out call, or a context whose generation has changed, ends this start without a process |
| Answer | `executable` (`{kind: "packaged", path}` is a read-only file in the package; `{kind: "tool", tool}` names a tool definition of the plugin itself, which the host resolves like any tool launch), `arguments`, `environment`, `initializationOptions`, `workspaceConfiguration`, `gates` (at most 8), `writablePaths` (at most 32, all inside the four directories); an unknown field is refused |
| Decoding | The host passes `{manifest, languageID, request, result}` to `plugins.languageServers.decodeLaunch`, which returns an ordinary launch plan (`launch` is `packaged` or `tool`); the plan's `sessionPolicy` also carries the answer's `gates` |
| Rejection codes | `plugin.launch.invalidResult`, `plugin.launch.outsidePackage`, `plugin.launch.undeclaredTool`, `plugin.launch.unwritablePath`, `plugin.launch.invalidRequest`, `plugin.launch.unavailable`, `plugin.launch.notBackend` |
| macOS today | A `{kind: "tool"}` answer is resolved with the user's tool choice and discovery, and a program inside the project is refused (opening a project must not run it); it does not start in a remote workspace; the temporary directory is removed when the server process ends |

### Plugin-owned sessions (`languageServers` 3)

A plugin backend drives its own server only through what the server declares. A `languageServers` 3 server can declare:

| Field | Meaning |
| --- | --- |
| `requestMethods` | LSP methods the backend may send with `languageServer/request` or `languageServer/notify`; at most 64 |
| `commands` | `workspace/executeCommand` commands the backend may run; at most 128; declared exactly when `requestMethods` lists `workspace/executeCommand` |
| `serializedCommands` | A subset of `commands` that runs one at a time: a request for a command that is already running joins it and receives the same answer |
| `forwardNotifications` | Server notifications the host forwards to the backend; at most 32 |
| `state` | `{scope: "workspace", retention: {maxEntries, maxAgeDays}}`; `maxEntries` is 1–64, `maxAgeDays` is 1–365 |
| `remote` | `supported` (default) or `unsupported`; an `unsupported` server is not started for a remote workspace |

- A name is non-empty text of at most 256 bytes without whitespace or control characters, unique within its list. Methods the host sends itself (`initialize`, `initialized`, `shutdown`, `exit`, the document-sync methods such as `textDocument/didOpen`, `workspace/didChangeWorkspaceFolders`, and every `$/` method) cannot appear in `requestMethods` or `forwardNotifications`.
- Declaring these fields below `languageServers` 3 is `plugin.manifest.invalid`. A server that lists `requestMethods` or `forwardNotifications` must declare `languageServer.request`; declaring the permission without such a server is also `plugin.manifest.invalid`.
- The launch plan's `sessionPolicy` is these fields (as sorted lists) plus `serverID` (the `languageID`) and `gates`. The host passes it unchanged as `lsp.startServer.sessionPolicy` and Core's LSP engine enforces it: the host cannot widen it and the backend cannot step outside it.
- A request must be in `requestMethods`, and a `workspace/executeCommand` must carry a `params.command` in `commands`; otherwise the result is `plugin.host.methodNotDeclared`. A notification may be `workspace/didChangeConfiguration` or a `requestMethods` entry other than `workspace/executeCommand`; the host's own methods are still refused.
- The session must belong to the calling plugin and be running. A request waits at most `timeoutMilliseconds` (default 60000, at most 300000); cancelling it sends `$/cancelRequest` to the server. A server's JSON-RPC error does not forward its `data`.
- A **gate** is `{id, until: "notification", method, match}`, declared by the launch answer. It opens when the server sends a notification with that `method` whose params contain every member of `match`. Until every gate has opened, `request` calls wait in order, at most 128; the 129th is `plugin.host.busy`. Notifications are never held.
- When the server stops, crashes, restarts, or the plugin is disabled, every pending request ends with `plugin.host.languageServerUnavailable`; the backend asks again after `didChangeState` reports `running` with a new `generation`.

**State directories and pruning (API-20).** For each server and each workspace the host keeps one state directory under the plugin's storage (never under the package or the application bundle) and gives the server its own as `workspaceStateDirectory`. Disabling and updating the plugin do not delete it. The plugin chooses the sub-directory names inside it; the host chooses the name of each per-workspace directory and never looks inside. The host refreshes the directory's modification time at every server start and prunes in the background after a start: it asks Core's `plugins.languageServers.stateRetention` which directories to delete, namely those older than `maxAgeDays` and, of the rest, all but the newest `maxEntries` that open workspaces leave free. A workspace that is open is never pruned, and a plugin that declares no `state.retention` keeps everything. Pruning runs on macOS only.

### Host methods and notifications

A backend talks to its own language server with these methods and notifications; the `lithe/host/` prefix is omitted. Every `languageServer/*` method takes `context: {contextID, generation}` and `serverID` (the `languageID` of one of this plugin's servers); a server of another plugin is `plugin.host.foreignContribution`.

| API | Method | Purpose | Permission |
| --- | --- | --- | --- |
| API-25 | `languageServer/request` | Send a JSON-RPC request to the plugin's own language server; extra params `method`, `params?`, `timeoutMilliseconds?` (1–300000, default 60000); returns `{value}` (the server's `result`, which may be `null`); checked against the session policy | `languageServer.request` |
| API-25 | `languageServer/notify` | Send a notification to the plugin's own language server; extra params `method`, `params?`; returns `{delivered}`, which is `false` only for a `workspace/didChangeConfiguration` sent before the server is initialized (the value is then sent with `initialized`) | `languageServer.request` |
| API-22, API-25 | `languageServer/setStatus` | Show the plugin's own preparation state (importing a project, indexing) in the host status UI: `state` is `preparing`, `ready`, `degraded`, or `failed`, with optional `message?` (shown as given, cut to 1024 bytes) and `percentage?` (a whole number 0–100); returns `{}`; the host removes it when the server stops. It is not `lithe/function/progress`, which belongs to a running call | `languageServer.request` |
| API-12 | `documents/registerContentProvider` | Bind a URI scheme to a backend function: when a document of that scheme is opened, the host calls the function for read-only content (at most 1 MiB, the host waits 30 seconds); one provider per scheme per workspace window; returns `{subscriptionID}`. Takes no `serverID`; revoke it with `subscriptions/dispose` | `documents.contentProvider` |

| API | Notification | Meaning | Permission |
| --- | --- | --- | --- |
| API-25 | `languageServer/didChangeState` | `{context, serverID, state, generation}`; `state` is `starting`, `running`, `stopped`, or `crashed` | `languageServer.request` |
| API-25 | `languageServer/didReceiveNotification` | `{context, serverID, generation, method, params?, truncated?}`: a notification the server sent and `forwardNotifications` lists; above `limits.notificationBytes` it is delivered once with `truncated: true` and without `params` | `languageServer.request` |

`generation` is a positive integer the host raises each time it starts the server for this context, so the backend can tell which process a state or notification belongs to; once a new process has started, the host no longer delivers notifications of the old one.

Errors of these methods:

| Error code | JSON-RPC | Meaning |
| --- | --- | --- |
| `plugin.host.methodNotDeclared` | `-32024` | The method or `workspace/executeCommand` command is not in the server's declared session |
| `plugin.host.languageServerUnavailable` | `-32022` | The server is not running for this context, or it stopped while the request was in flight |
| `plugin.host.languageServerError` | `-32023` | The server answered with a JSON-RPC error; `details` is `{code, message}` |
| `plugin.host.foreignContribution` | `-32026` | The named server belongs to another plugin |
| `plugin.host.timeout` | `-32015` | The host's deadline passed |
| `plugin.host.cancelled` | `-32800` | The request was cancelled |
| `plugin.host.busy` | `-32020` | The 129th waiting request (see gate) |
| `plugin.host.limitExceeded` | `-32014` | The result exceeds the host's bound |
| `plugin.host.preconditionFailed` | `-32019` | `documents/registerContentProvider`: the scheme already has a provider |

### Output links (`outputLinkMatchers` 1)

`outputLinkMatchers[]` lets a plugin make text in the Run and Test consoles clickable, such as a traceback frame or a panic location. The host knows no tool's output format: the plugin declares the pattern and Core finds the matches. Requires `requires.outputLinkMatchers` 1. Matchers alone are a complete plugin; a matcher without `resolve` needs no backend.

```json
"outputLinkMatchers": [
  { "id": "traceback", "pattern": "File \"(?P<path>[^\"]+)\", line (?P<line>\\d+)", "scope": "run" }
]
```

| Field | Meaning |
| --- | --- |
| `id` | Plugin-local ID: a lowercase letter followed by up to 63 lowercase letters, digits, `.`, `_`, or `-`, unique in the plugin. At most 16 matchers per plugin |
| `pattern` | A regular expression of 1–512 bytes matched against one console line. It uses the Rust `regex` syntax, so it has no backreferences and no look-around, and its compiled size is limited. It names at most 8 groups, each a camelCase name, and must not match empty text |
| `scope` | `run` or `test`: the console the matcher applies to. Absent means both |
| `resolve.function` | Optional ID of a function of the plugin's `backend` 2 process; needs `backend` 2 |

- Without `resolve`, the pattern must capture a `path` group and may capture `line` and `column` (1-based); no other group name is accepted. A relative `path` becomes one candidate per workspace root, an absolute one counts only inside a workspace root, and a path that leaves every root is not linked: a console line can never open a file outside the workspace.
- With `resolve`, the pattern may name any groups. When the user clicks, the host calls the function with `{groups}` (the captured text by name) and the window's `context`, and the backend answers `{uri, range?}` or `null`.
- Core owns the decision, so every host links the same text: `outputLinks.match` takes `{scope, platform, caseSensitive?, workspaceRoots, sources: [{pluginID, matchers}], lines}` and returns `{links}`. Each link is `{line, start, length, pluginID, matcherID}` plus either `location` (`{paths, line?, column?}`: the host opens the first existing path) or `resolve` (`{function, groups}`). `start` and `length` count UTF-16 code units of the line. Matches are found in source order and then declaration order, and a match overlapping an earlier link is skipped. A request carries at most 256 lines, 64 sources, and 16 roots; a line is scanned up to 4096 bytes and links at most 8 times.
- `outputLinks.decodeResolution` takes `{reply}` (what the function answered) and returns the location to open, `{path, line?, column?}` with 1-based positions, or `null`. A reply opens only a `file:` URI, only the members `uri` and `range` are accepted, and a 0-based `range.start` is converted.
- The host runs matching only for lines it displays, never for output of a plugin that is not enabled, and ignores a link whose file does not exist. Implemented on macOS only (API-22).

### Contract sections

| Section | Content |
| --- | --- |
| [Languages](../../contract/v1/README.md#languages) | Language IDs, extensions, file names, project markers, dependency paths |
| [Grammars and editor configuration (`languages` 2)](../../contract/v1/README.md#grammars-and-editor-configuration-languages-2) | Monarch limits and rejections, comment and bracket configuration, worker-only execution |
| [Language servers](../../contract/v1/README.md#language-servers) | Launch kinds (including `backend`), arguments, probe arguments, runtime dependencies, session fields |
| [Runtime launch (API 2)](../../contract/v1/README.md#runtime-launch-api-2) | Start a packaged entry with `java` or `node`, optionally with a packaged runtime |
| [Tool references (`serverProcess` 4)](../../contract/v1/README.md#tool-references-serverprocess-4) | Reference runtimes by tool-definition ID; the `tool` launch kind in a launch plan |
| [Host integrations (API 2)](../../contract/v1/README.md#host-integrations-api-2) | Behavior data cannot express lives in host code; any plugin can opt in by name; an ID the host does not implement returns `plugin.integration.unsupported`. A build system's host integration is still official plugins only |
| [Launch templates (API 2)](../../contract/v1/README.md#launch-templates-api-2) | Template variables such as `${pluginDir}`, `${stateDir}`, `${workspaceStateDir}` |
| [Workspace configuration (`languageServers` 2)](../../contract/v1/README.md#workspace-configuration-languageservers-2) | Expanding and pushing `workspaceConfiguration` |
| [Backend-resolved launch (`serverProcess` 3)](../../contract/v1/README.md#backend-resolved-launch-serverprocess-3) | The backend answers the command line: `resolveParams`, answer fields, rejection codes, macOS limits |
| [Plugin-owned sessions (`languageServers` 3)](../../contract/v1/README.md#plugin-owned-sessions-languageservers-3) | Session fields, session policy, gates, state directories and pruning, preparation status |
| [Language server host methods](../../contract/v1/README.md#language-server-host-methods) | `languageServer/request`, `notify`, `setStatus` and the two notifications |
| [Content providers](../../contract/v1/README.md#content-providers) | `documents/registerContentProvider`: read-only documents served by URI scheme |
| [Output link matchers (`outputLinkMatchers` 1)](../../contract/v1/README.md#output-link-matchers-outputlinkmatchers-1) | Links in Run and Test console output, `outputLinks.match`, `outputLinks.decodeResolution` |
| [Execution providers](../../contract/v1/README.md#execution-providers) | `run` and `test` executables, argument templates, test discovery |

### Examples and cases

| File | Content |
| --- | --- |
| [`languages/language-packaged-server.plugin.json`](../../contract/v1/fixtures/languages/language-packaged-server.plugin.json) | Level 1: packaged language server with run and test providers (PHP) |
| [`languages/language-system-command.plugin.json`](../../contract/v1/fixtures/languages/language-system-command.plugin.json) | Level 1: system-command language server with run and test providers (Go) |
| [`languages/node-runtime.plugin.json`](../../contract/v1/fixtures/languages/node-runtime.plugin.json) | Level 2: launched with the `node` runtime, with plugin settings |
| [`languages/language-host-integration.plugin.json`](../../contract/v1/fixtures/languages/language-host-integration.plugin.json) | Level 2: a language server that declares a host integration (Java) |
| [`languages/language-grammar.plugin.json`](../../contract/v1/fixtures/languages/language-grammar.plugin.json) | Level 3, `languages` 2: a grammar-only plugin that starts no process |
| [`languages/language-server-backend-launch.plugin.json`](../../contract/v1/fixtures/languages/language-server-backend-launch.plugin.json) | Level 3, `serverProcess` 3 and `languageServers` 3: a language server whose command line the backend answers, with session fields and `languageServer.request` |
| [`languages/grammars/http.monarch.json`](../../contract/v1/fixtures/languages/grammars/http.monarch.json) | A complete Monarch grammar |
| [`languages/command-plans.json`](../../contract/v1/fixtures/languages/command-plans.json) | 35 golden expansions of run/test templates and language-server launch plans |
| [`languages/runtime-versions.json`](../../contract/v1/fixtures/languages/runtime-versions.json) | 12 golden outcomes of `plugins.declarative.checkRuntimeVersion` |
| [`languages/language-server-launch-resolution.json`](../../contract/v1/fixtures/languages/language-server-launch-resolution.json) | 17 golden outcomes: the first step of a backend-resolved launch (`languageServerLaunchPlan`) and the launch plan or rejection code from decoding the backend's answer (`plugins.languageServers.decodeLaunch`) |
| [`run/output-link-matchers.json`](../../contract/v1/fixtures/run/output-link-matchers.json) | The requests of `outputLinks.match` and the links it returns, plus what `outputLinks.decodeResolution` makes of a matcher's `resolve` reply |
| [`run/tool-reference-launch.plugin.json`](../../contract/v1/fixtures/run/tool-reference-launch.plugin.json) | A language server's `runtime` launch references the runtime by tool ID `interpreter` (`serverProcess` 4), and the manifest also declares an `outputLinkMatchers` entry |

Language rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json).

### Related guides

- [Building an LSP plugin end to end](../../guides/lsp-plugin-lifecycle.md)
- [Build, run, and test for language plugins](../../guides/build-tool-integration.md)
