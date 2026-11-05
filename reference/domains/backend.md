# 插件后端与插件函数 / Plugin backend and plugin functions

[中文](#中文) · [English](#english)

## 中文

插件后端是插件自己的长驻进程，用来运行网页做不到的事：数据库驱动、网络客户端、文件格式解析、调用宿主服务。它只服务插件自己，不属于任何构建系统、语言或工作区，也不回答 `lithe/build/*`、VCS 等领域请求。后端与宿主之间说插件函数协议（`litheFunctions`），从版本 2 起可以按权限请求宿主服务；版本 3 起可以启动由宿主拥有的 Run 会话和调试会话；版本 4 起可以经 Core 调用其他插件导出的函数。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `backend`（与 `serverProcess` 的启动字段相同）；跨插件调用另有 `pluginFunctions` 段（`exports`、`calls`） |
| 能力版本 | `backend` 1–2、`serverProcess` 1；`ui` 4 的后端命令需要 `backend` 2；跨插件调用需要 `pluginFunctions` 1（同时要求 `backend` 2、`serverProcess` 1，`pluginAPIVersion` ≥ 3）；见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheFunctions` 1–4、`litheLogging` 1，见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | API-01、04、05、06、07、08、20、21、22、23、24、35、37、40、41，见 [API 总表](../api-catalog.md#现在能用的-api) |
| 权限 | `backend.launch`；宿主服务按方法另加权限（`credentials.store`、`workspace.metadata`、`workspace.files.read`、`workspace.files.watch`、`workspace.search`、`terminal.create`、`run.start` 等），见[已公开的宿主方法](../api-catalog.md#已公开的宿主方法)。跨插件调用没有清单权限：调用方的 `calls` 和被调方的 `exports` 就是双方的同意 |
| 激活事件 | `backend` 1：插件网页第一次调用 `functions.*` 时启动。`backend` 2 另可用 `onCommand:<id>` 或可选的 `onWorkspaceOpen`。只导出函数的插件不需要网页、命令或 `onWorkspaceOpen`，被调用时启动后端；只调用的插件仍要有其中一种 |
| Windows | 没有插件后端宿主，不声明 `backend` |
| 官方 SDK | [Node](../../sdk/node/README.md)、[Java](../../sdk/java/README.md)、[Rust](../../sdk/rust/README.md)；各 SDK 支持的协议版本见[宿主与官方 SDK 对照](../capabilities-and-versions.md#宿主与官方-sdk-对照)。三个 SDK 都协商函数协议 1–4，已封装 `functions/providers`、`functions/invoke`，被调函数能读到 `caller`，用法见各 SDK 的 README；`parentOperationID` 不传时，SDK 取正在处理的 `lithe/function/invoke` 的 `operationID`，协商到 3 及以下时这两个方法在 SDK 本地以 `plugin.host.unavailable` 失败。协议 3 的 `run/*`、`debug/start`、`languageServer/*`、`documents/registerContentProvider` 仍没有封装，可以先用 SDK 的通用宿主请求调用 |

### 版本对照

| 版本 | 新增 | 契约章节 |
| --- | --- | --- |
| `backend` 1 | 每个插件一个长驻进程，只有插件自己的网页能调用；没有网页工具窗口的 `backend` 1 会被拒绝 | [Plugin backend](../../contract/v1/README.md#plugin-backend) |
| `backend` 2 | 可以不依赖网页启动（网页、`ui.commands` 或 `onWorkspaceOpen` 至少一种）；所有工作区共享一个进程，每次调用显式带工作区上下文 | [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) |
| `litheFunctions` 1 | 注册、撤回、调用、结果、进度、取消；不提供宿主服务 | [Plugin functions](../../contract/v1/README.md#plugin-functions) |
| `litheFunctions` 2 | 宿主服务：凭据 `credentials/get`、`set`、`delete` | [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) |
| `litheFunctions` 3 | 工作区上下文与能力发现、宿主通知、公共文件/文档/窗口方法、独立编辑器动作、插件自有原生终端、宿主拥有的 Run 与调试会话（`run/start`、`run/stop`、`debug/start`、通知 `run/didExit`） | [Protocol 3](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3)、[Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) |
| `litheFunctions` 4 | 保留版本 3 的全部内容，新增跨插件调用：`functions/providers`、`functions/invoke`，被调方收到顶层 `caller` | [Cross-plugin calls (protocol version 4)](../../contract/v1/README.md#cross-plugin-calls-protocol-version-4) |
| `pluginFunctions` 1 | 清单 `pluginFunctions` 段：导出（`exports`）和调用声明（`calls`）；不授予权限 | [Declaring exports and calls](../../contract/v1/README.md#declaring-exports-and-calls) |
| `litheLogging` 1 | 结构化诊断通知 `lithe/log`；当前官方 SDK 要求宿主提供 | [Structured plugin diagnostics](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) |

### 跨插件调用（协议 4）

插件函数默认只属于插件自己。插件导出函数后，另一个已启用、且在清单里声明了这次调用的插件，可以经 Rust Core 调用它：调用方的后端碰不到被调方的进程，被调方的后端也只知道自己正在服务的那次调用；宿主只在被调方的后端没有运行时负责启动它。中转是 Core 的逻辑，在所有运行插件后端的平台上一样；目前只有 macOS 运行插件后端，Windows 不声明 `pluginFunctions`。

#### 清单声明

| 项目 | 规则 |
| --- | --- |
| 前提 | `pluginAPIVersion` ≥ 3，有 `backend`，`requires` 写 `pluginFunctions: 1`、`backend: 2`、`serverProcess: 1`；`requires` 列出 `pluginFunctions` 当且仅当清单有 `pluginFunctions` 段 |
| 段内容 | `exports` 与 `calls` 至少有一个，各最多 32 项、按 `id` 排序、`id` 不重复；多余的字段是 `plugin.manifest.invalid` |
| 导出 | `{id, version, context}`。`id` 是函数 ID；`version` 是导出的契约版本（1–1000 的整数），参数或结果有不兼容变更就换新版本；`context` 目前只有 `"workspace"`，表示只在本地工作区上下文里运行，缺失或未知则整个清单被拒绝 |
| 调用 | `{id, version}`。只写函数 ID 和调用方对照的版本，不写被调方的插件 ID，运行时再发现提供者；两个版本必须相等，否则 `plugin.function.versionMismatch` |
| 注册 | 声明导出不等于注册：被调方的后端仍要 `lithe/function/register`，未注册或已撤回的函数是 `plugin.function.notExported`。清单没有导出的函数保持私有，引用它仍是 `plugin.function.wrongPlugin` |
| 宿主支持 | 宿主向声明了 `calls` 的插件开放 `functions` 宿主服务；不支持 `pluginFunctions` 的宿主拒绝这类清单（`plugin.api.unsupported`） |

#### 宿主方法

只对协商到协议 4、且声明了至少一个 `calls` 的后端开放。协议 3 及以下返回 `-32601`、`plugin.host.unavailable`；没有声明任何 call 返回 `-32010`、`plugin.host.permissionDenied`。`getCapabilities` 在 `functions` 服务下列出这两个方法。

| 方法（前缀 `lithe/host/`） | 参数 | 结果 |
| --- | --- | --- |
| `functions/providers` | `{functionID}` | `{providers: [{pluginID, version}]}` |
| `functions/invoke` | `{pluginID, functionID, context: {contextID, generation}, params?, timeoutMilliseconds?, parentOperationID?}` | `{value}` |

- `providers` 列出导出该函数、已启用、且不是调用方自己的插件，按插件 ID 排序，每项带导出的 `version`。调用方必须声明过该函数，否则 `plugin.function.notDeclared`。被调方的后端不必正在运行。
- `invoke` 调用一个提供者。`context` 必填，是调用方从工作区服务得到的本地工作区上下文；`params` 是 JSON 对象，默认 `{}`；`timeoutMilliseconds` 是正整数，默认 60000，上限 600000；未定义的字段返回 `-32018`、`plugin.host.unsupportedOption`。`value` 是被调方的结果，`null` 也是结果。
- 调用方用 `$/cancelRequest` 取消，得到 `plugin.host.cancelled`。
- Core 按固定顺序检查（调用方声明、被调方启用、被调方导出、版本相等、上下文、本地工作区、参数大小、所属调用、调用链、深度、并发数），在向被调方写任何内容之前拒绝，第一个失败的检查决定错误码。

#### 被调方收到的请求

Core 把中转的调用以 `lithe/function/invoke` 转给被调方。和任何函数调用一样，请求顶层有 `functionID`、`operationID`、`context`、`params`；中转的调用另有顶层字段 `caller: {pluginID}`。

- `caller.pluginID` 由 Core 按调用会话的身份写入，在 `params` 之外，调用方无法提供：`functions/invoke` 请求带顶层 `caller` 会被 `-32018` 拒绝。要按调用方区分行为或保存状态的函数，读这个字段，不要读 `params` 里的插件 ID。调用方 `params` 里叫 `caller` 的键只是普通数据，原样传递。
- 宿主自己发起的调用没有 `caller`，缺失表示宿主。协商到协议 3 及以下的被调方收不到这个字段；依赖它的被调方必须协商协议 4。
- `operationID` 标识正在服务的这次调用。被调方在服务一次中转的调用时再发起 `functions/invoke`（嵌套调用），要把这个 `operationID` 作为 `parentOperationID` 传入，Core 才知道新调用属于哪一次。`parentOperationID` 是 1–128 字节的字符串，否则 `-32602`、`plugin.host.invalidParams`；不传时，Core 把新调用当作后端正在服务的每一次中转调用的子调用，所以后端应当总是传。
- 调用链：新调用的链是父调用的链加上调用方。调用一个已在链上的插件（包括调用自己）是 `plugin.function.callCycle`，链最长 4 次调用。取消、超时或结束父调用，会取消它的子调用，各被调方收到 `$/cancelRequest`。`parentOperationID` 指向宿主直接发起的调用时，新调用开始一条新链；指向后端没有在服务的调用（已结束或不存在）时被拒绝，`plugin.host.cancelled`。
- 被调方的后端没有运行时，Core 等待，并在调用方会话上发出运行时事件 `functionProviderStartRequired`，宿主按普通的插件启动流程启动它；30 秒内没有协商并注册好函数，调用以 `plugin.function.providerUnavailable` 结束。

#### 信任不传递

- 被调方用自己的权限、状态目录和凭据运行；调用方不获得被调方的任何权限，被调方也不获得调用方的。调用方的插件 ID 只出现在 `caller` 字段里，不写进被调方的会话。
- 导出的函数不得接受调用方给的可执行路径、命令行或脚本去启动进程，也不得读写调用方指定的路径而不核对自己的访问范围。调用方需要被调方运行什么，函数应当接收一个由被调方自己解释的请求（任务名、文档），由被调方决定可执行文件。
- 这不是安装依赖：安装调用方不会连带安装被调方，停用被调方只会让调用找不到提供者；不参与安装闭包，不做版本求解。远程工作区的上下文被拒绝，`plugin.host.authorityUnsupported`；导出的 `context: "workspace"` 只指本地工作区，后端仍在运行 Lithe 的机器上。
- 停用、启用、替换和卸载插件对中转立即生效，不需要重启。macOS 只在启动时注册后端插件，所以新安装或更新的后端插件要重启 Lithe 后才会出现在 `providers` 里。

#### 错误码

| 错误码 | 何时 |
| --- | --- |
| `plugin.function.notDeclared` | 调用方没有声明该函数；调用进行中调用方更新后不再声明，也是它 |
| `plugin.function.providerDisabled` | 被调方没有安装或未启用；调用进行中被调方被停用、卸载，或更新后不再导出该函数或改了版本，也是它 |
| `plugin.function.notExported` | 被调方的后端没有注册该函数，或已撤回 |
| `plugin.function.versionMismatch` | 调用方声明的版本与被调方导出的版本不相等 |
| `plugin.function.callCycle` | 被调方已在调用链上，包括调用自己 |
| `plugin.function.depthExceeded` | 调用链超过 4 次调用 |
| `plugin.function.tooManyCalls` | 调用方已有 8 个中转调用在运行 |
| `plugin.function.payloadTooLarge` | 参数超过 256 KiB，或被调方的结果超过 256 KiB（超过 1 MiB 是 `invalidResult`） |
| `plugin.function.providerUnavailable` | 被调方的后端 30 秒内没有就绪；或调用期间后端退出、撤回了该函数 |

被调方自己的失败保留原意：函数回答了错误时是 `plugin.function.failed`（`-32070`），`details.pluginErrorCode` 是被调方的 `error.data.code`；没有 `result`，或结果序列化后超过 1 MiB，是 `plugin.function.invalidResult`（`-32071`）。工作区上下文被关闭或换代是 `plugin.host.contextExpired`，截止时间到了是 `plugin.host.timeout`。Core 不重启后端，也不重试调用。

### Run 与调试会话（协议 3）

后端可以启动宿主当作自己的会话呈现的运行或调试：后端决定启动什么，宿主拥有进程、控制台、停止按钮和会话的结束。Run 会话只增加宿主呈现（Run 窗口里的条目、控制台、退出状态），不授予其他能力。`run/start` 与 `debug/start` 带 `context: {contextID, generation}` 和对应权限；`run/stop` 不带 `context`，所以启动它的上下文关闭后仍能停止。

| 方法（前缀 `lithe/host/`） | 权限 | 参数 | 结果 |
| --- | --- | --- | --- |
| `run/start` | `run.start` | `context, label, process, temporaryFiles?, debug?, rerun?` | `{runID}` |
| `run/stop` | `run.start` | `{runID}` 或 `{sessionID}` | `null` |
| `debug/start` | `debug.start` | `context, label, adapter, request, configuration` | `{sessionID}`，见[调试适配器](debug-adapters.md#由插件后端发起调试会话) |

通知（后端不回复）：`lithe/host/run/didExit`，参数 `{runID, exitCode?, signal?}`，授予条件是 `run.start`。`run.start` 需要 `backend` 2，授予 `run/start`、`run/stop` 和 `run/didExit`。

`run/start` 的参数：

- `label`：1–256 个字符，不含控制字符，作为 Run 窗口条目的标题。
- `process`：`{executable, arguments, workingDirectory, environment}`，四个字段都必填，未知字段被拒绝。`executable` 是 `{kind: "tool", tool}`（`tool` 是清单 `requiresTools` 列出的工具定义 ID，宿主按[工具链](toolchains.md)的分层解析）或 `{kind: "packaged", path}`（插件包内文件的规范化相对路径）。参数、工作目录（相对工作区根、且是其中已存在的目录）和环境的上限同[运行准备](../../contract/v1/README.md#provider-run-preparation-buildproviders-4)。
- `temporaryFiles`：最多 4 个 `{name, contents}`，每个文本最多 4 MiB。宿主在自己的临时目录写入，运行结束时删除，并把 `process.arguments` 和 `process.environment` 里的 `${temporaryFile:<name>}` 替换成文件路径，插件不需要知道临时目录在哪。`name` 唯一，1–64 个字母、数字、`.`、`_`、`-`，以字母或数字开头。
- `debug`：`{configurationType, target}`，让这次运行成为调试会话，见[调试适配器](debug-adapters.md#由插件后端发起调试会话)。
- `rerun`：`{function, data}`，插件已注册的函数和它的参数（最多 64 KiB 的 JSON 对象）。用户选“再次运行”时，宿主以 `data` 为参数、当前 `context` 调用该函数，后端再回答一次新的 `run/start`，所以重启由后端准备；没有 `rerun` 的条目不能再次运行。

其他规则：

- `run/stop` 按 `runID` 停止运行，或按 `sessionID` 停止调试会话，已经结束时同样返回 `null`；取消宿主请求等同停止。后端只能停止自己启动的运行和会话。
- 进程结束时宿主发 `run/didExit`，每次运行只发一次，只发给持有 `run.start` 的会话；`exitCode` 是退出状态，`signal`（1–127）是结束它的信号，宿主丢失进程时两个字段都没有。
- 参数格式错误是 `plugin.host.invalidParams`，Core 的拒绝（`plugin.run.invalidStart`、`plugin.debug.invalidStart`）在 `details.code` 和 `details.field`；上下文已关闭是 `plugin.host.contextExpired`；`tool` 解析不到可执行文件是 `plugin.host.toolUnresolved`。
- 插件被禁用或后端退出时，宿主保留运行中的进程，只是不再通知后端；重新启动的后端不会收到它没见过开始的运行的 `run/didExit`。
- 想让 Run 控制台里的输出出现链接，用 `outputLinkMatchers` 1 声明匹配器：Core 负责匹配，`resolve` 函数在用户点击时才调用，见 [Output link matchers](../../contract/v1/README.md#output-link-matchers-outputlinkmatchers-1)。
- Windows 没有插件后端宿主，没有这些方法。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Plugin backend](../../contract/v1/README.md#plugin-backend) | 后端声明、状态目录、启动与停止时机、按平台发布 |
| [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) | 启动方式、`onWorkspaceOpen` 不阻塞工作区、`workspace.metadata` |
| [Plugin functions](../../contract/v1/README.md#plugin-functions) | 协议总览 |
| [Capability ownership](../../contract/v1/README.md#capability-ownership) | 插件函数不能代替领域契约 |
| [Declarative or backend implementation](../../contract/v1/README.md#declarative-or-backend-implementation) | 什么时候用声明式数据、什么时候写后端 |
| [Negotiation](../../contract/v1/README.md#negotiation) | `initialize` 里的版本协商 |
| [References and IDs](../../contract/v1/README.md#references-and-ids) | 函数引用和 ID 规则 |
| [Registration and withdrawal](../../contract/v1/README.md#registration-and-withdrawal) | 注册和撤回函数 |
| [Invocation](../../contract/v1/README.md#invocation)、[Progress](../../contract/v1/README.md#progress) | 调用、结果、进度、取消 |
| [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) | 凭据服务 |
| [Workspace contexts and capability discovery (protocol version 3)](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) | `getCapabilities`、上下文 `contextID`/`generation`、宿主失败码 |
| [Host notifications (protocol version 3)](../../contract/v1/README.md#host-notifications-protocol-version-3) | `workspace/didChangeContexts`、`subscriptions/notify`、`commands/didRun`、`run/didExit` |
| [Extending version 3](../../contract/v1/README.md#extending-version-3) | 协议 3 如何不升版本地增加方法 |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | 文件读取、监听、搜索与窗口消息（文档和编辑器方法见[编辑器](editor.md)） |
| [Plugin-owned native terminals (function protocol 3)](../../contract/v1/README.md#plugin-owned-native-terminals-function-protocol-3)、[Close, retained context and reopen](../../contract/v1/README.md#close-retained-context-and-reopen) | 插件自有原生终端 |
| [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) | `run/start`、`run/stop`、`debug/start`、`run/didExit`：参数、限制、失败码 |
| [Cross-plugin calls (protocol version 4)](../../contract/v1/README.md#cross-plugin-calls-protocol-version-4) | `pluginFunctions` 清单声明、`functions/providers`、`functions/invoke`、检查顺序、调用链、`caller` 字段、未运行的提供者、热生效 |
| [Output link matchers (`outputLinkMatchers` 1)](../../contract/v1/README.md#output-link-matchers-outputlinkmatchers-1) | Run 与测试控制台里的输出链接，`resolve` 函数在点击时调用 |
| [Structured plugin diagnostics (logging protocol 1)](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) | `lithe/log` 的字段和限制 |
| [Compatibility](../../contract/v1/README.md#compatibility)、[Official backend SDK](../../contract/v1/README.md#official-backend-sdk) | 版本兼容规则与官方 SDK |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | 结果与错误码 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`backend/backend.plugin.json`](../../contract/v1/fixtures/backend/backend.plugin.json) | `backend` 1：网页工具窗口调用后端，使用凭据服务 |
| [`backend/backend-workspace.plugin.json`](../../contract/v1/fixtures/backend/backend-workspace.plugin.json) | `backend` 2：`onWorkspaceOpen` 启动，`workspace.metadata` |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `backend` 2 加 `ui` 4：不打开工具窗口的命令，读文档并编辑 |
| [`backend/terminal-backend.plugin.json`](../../contract/v1/fixtures/backend/terminal-backend.plugin.json) | `backend` 2：插件自有原生终端的最小清单 |
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | 插件函数协议的标准报文：协商（含协议 4）、注册、调用、结果、进度、93 个宿主请求、23 个宿主失败、29 个宿主通知，以及被中转调用的 `caller`（`invocationCallers`）和中转失败码（`relayErrors`） |
| [`backend/plugin-functions-exporter.plugin.json`](../../contract/v1/fixtures/backend/plugin-functions-exporter.plugin.json) | `pluginFunctions` 1：只导出两个函数（`document.format` 版本 1、`symbols.list` 版本 2，`context` 为 `workspace`），没有网页、命令或 `onWorkspaceOpen` |
| [`backend/plugin-functions-caller.plugin.json`](../../contract/v1/fixtures/backend/plugin-functions-caller.plugin.json) | `pluginFunctions` 1：声明调用 `document.format` 版本 1，`onWorkspaceOpen` 启动 |
| [`backend/plugin-logging.json`](../../contract/v1/fixtures/backend/plugin-logging.json) | `litheLogging` 1：提供与回答、通知、拒绝用例、级别 |
| [`run/run-start.json`](../../contract/v1/fixtures/run/run-start.json) | `run/start` 的参数和 `run.decodeStart` 对每个用例的返回，共 17 个用例（工具与包内可执行文件、`debug` 与 `rerun`、各种拒绝）；临时文件依赖宿主的临时目录，由 Core 自己的测试覆盖 |
| [`run/tool-reference-launch.plugin.json`](../../contract/v1/fixtures/run/tool-reference-launch.plugin.json) | `run-start.json` 使用的清单：`requiresTools` 列出工具 `interpreter`，声明 `run.start`、`debug.start`，并带 `outputLinkMatchers` 1 |
| [`run/output-link-matchers.json`](../../contract/v1/fixtures/run/output-link-matchers.json) | `outputLinks.match` 的请求与返回的链接，以及匹配器 `resolve` 函数的回复经 `outputLinks.decodeResolution` 得到的结果 |

后端相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。

### 相关指南

- [插件后端：工作区上下文与宿主服务](../../guides/plugin-backend-workspace.md)
- [从插件打开原生终端](../../guides/native-terminals.md)
- [插件接入统一日志](../../guides/plugin-logging.md)

---

## English

A plugin backend is the plugin's own long-lived process. It runs what a web page cannot: database drivers, network clients, file-format parsers, calls to host services. It serves only its plugin, belongs to no build system, language, or workspace, and never answers domain requests such as `lithe/build/*` or VCS. Backend and host speak the plugin-function protocol (`litheFunctions`); from version 2 on, the backend may request host services its permissions grant; from version 3 on, it may start Run and debug sessions that the host owns; and from version 4 on, it may call functions that other plugins export, through Core.

| Item | Value |
| --- | --- |
| Manifest field | `backend` (the same launch fields as `serverProcess`); cross-plugin calls add a `pluginFunctions` section (`exports`, `calls`) |
| Capability versions | `backend` 1–2, `serverProcess` 1; `ui` 4 backend commands need `backend` 2; cross-plugin calls need `pluginFunctions` 1 (which also requires `backend` 2 and `serverProcess` 1, and `pluginAPIVersion` ≥ 3); see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheFunctions` 1–4, `litheLogging` 1; see [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | API-01, 04, 05, 06, 07, 08, 20, 21, 22, 23, 24, 35, 37, 40, 41; see the [API catalog](../api-catalog.md#apis-usable-now) |
| Permissions | `backend.launch`; host services need their own permission per method (`credentials.store`, `workspace.metadata`, `workspace.files.read`, `workspace.files.watch`, `workspace.search`, `terminal.create`, `run.start`, …); see [public host methods](../api-catalog.md#public-host-methods). Cross-plugin calls have no manifest permission: the caller's `calls` and the provider's `exports` are the consent of the two sides |
| Activation | `backend` 1 starts on the first `functions.*` call from the plugin's page. `backend` 2 may also start on `onCommand:<id>` or the optional `onWorkspaceOpen`. A plugin that only exports functions needs no page, command, or `onWorkspaceOpen`: a call starts its backend; a plugin that only calls still needs one of them |
| Windows | No plugin backend host; `backend` is not advertised |
| Official SDKs | [Node](../../sdk/node/README.md), [Java](../../sdk/java/README.md), [Rust](../../sdk/rust/README.md); protocol versions per SDK are in [Host and official SDKs](../capabilities-and-versions.md#host-and-official-sdks). The three SDKs all negotiate function protocol 1–4: they wrap `functions/providers` and `functions/invoke`, and a called function can read `caller`; see each SDK's README for usage. When `parentOperationID` is omitted, the SDK uses the `operationID` of the `lithe/function/invoke` being handled, and when version 3 or lower is negotiated both methods fail locally in the SDK with `plugin.host.unavailable`. The protocol 3 methods `run/*`, `debug/start`, `languageServer/*`, and `documents/registerContentProvider` still have no wrappers and can be called through the SDK's generic host request |

### Versions compared

| Version | Adds | Contract section |
| --- | --- | --- |
| `backend` 1 | One long-lived process per plugin, called only by the plugin's own web pages; a `backend` 1 without a web tool window is refused | [Plugin backend](../../contract/v1/README.md#plugin-backend) |
| `backend` 2 | Starts without a page (a web window, `ui.commands`, or `onWorkspaceOpen`, at least one); one process shared by every workspace, each call names its workspace context | [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) |
| `litheFunctions` 1 | Registration, withdrawal, invocation, results, progress, cancellation; no host services | [Plugin functions](../../contract/v1/README.md#plugin-functions) |
| `litheFunctions` 2 | Host services: credentials `credentials/get`, `set`, `delete` | [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) |
| `litheFunctions` 3 | Workspace contexts and capability discovery, host notifications, public file/document/window methods, independent editor actions, plugin-owned native terminals, host-owned Run and debug sessions (`run/start`, `run/stop`, `debug/start`, and the `run/didExit` notification) | [Protocol 3](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3), [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) |
| `litheFunctions` 4 | Everything in version 3, plus cross-plugin calls: `functions/providers` and `functions/invoke`; a provider receives a top-level `caller` | [Cross-plugin calls (protocol version 4)](../../contract/v1/README.md#cross-plugin-calls-protocol-version-4) |
| `pluginFunctions` 1 | The manifest `pluginFunctions` section: exports (`exports`) and call declarations (`calls`); grants no permission | [Declaring exports and calls](../../contract/v1/README.md#declaring-exports-and-calls) |
| `litheLogging` 1 | Structured diagnostics notification `lithe/log`; current official SDKs require the host to offer it | [Structured plugin diagnostics](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) |

### Cross-plugin calls (protocol 4)

A plugin function is private to its plugin unless the plugin exports it. An exported function can be called, through Rust Core, by another enabled plugin whose manifest declares that call: the calling backend never reaches the provider's process, the provider's backend never learns more than the call it serves, and the host takes part only by starting a provider's backend that is not running. The relay is Core logic, the same on every platform that runs plugin backends; today only macOS does, and Windows does not advertise `pluginFunctions`.

#### Manifest declarations

| Item | Rule |
| --- | --- |
| Prerequisites | `pluginAPIVersion` ≥ 3, a `backend`, and `requires` with `pluginFunctions: 1`, `backend: 2`, `serverProcess: 1`; `requires` lists `pluginFunctions` exactly when the manifest has a `pluginFunctions` section |
| Section | At least one of `exports` and `calls`, each at most 32 entries, sorted by `id`, no `id` twice; any other field is `plugin.manifest.invalid` |
| Export | `{id, version, context}`. `id` is a function ID; `version` is the exported contract version (an integer from 1 to 1000), and an incompatible change to the params or result takes a new version; `context` has one value for now, `"workspace"`, meaning the function runs only for a local workspace context; a missing or unknown `context` refuses the whole manifest |
| Call | `{id, version}`. It names a function ID and the version the caller was written against, never the providing plugin's ID: the caller discovers providers at run time; the two versions must be equal, otherwise `plugin.function.versionMismatch` |
| Registration | Declaring an export does not register it: the provider's backend still calls `lithe/function/register`, and a function it has not registered, or has withdrawn, is `plugin.function.notExported`. A function the manifest does not export stays private, and a reference to it is still `plugin.function.wrongPlugin` |
| Host support | The host grants a plugin that declares `calls` the `functions` host service; a host without `pluginFunctions` refuses such a manifest (`plugin.api.unsupported`) |

#### Host methods

Only a backend that negotiated protocol version 4 and whose plugin declares at least one call may use them. A backend on version 3 or earlier gets `-32601`, `plugin.host.unavailable`; a plugin that declares no call gets `-32010`, `plugin.host.permissionDenied`. `getCapabilities` lists both methods under the `functions` service.

| Method (prefix `lithe/host/`) | Params | Result |
| --- | --- | --- |
| `functions/providers` | `{functionID}` | `{providers: [{pluginID, version}]}` |
| `functions/invoke` | `{pluginID, functionID, context: {contextID, generation}, params?, timeoutMilliseconds?, parentOperationID?}` | `{value}` |

- `providers` lists the enabled plugins other than the caller that export the function, sorted by plugin ID, each with its exported `version`. The caller must have declared the function, otherwise `plugin.function.notDeclared`. A provider's backend need not be running.
- `invoke` calls one provider. `context` is required: a local workspace context the caller learned from the workspace service. `params` is a JSON object and defaults to `{}`. `timeoutMilliseconds` is a positive integer, defaults to 60000, and is capped at 600000. A field the method does not define is `-32018`, `plugin.host.unsupportedOption`. `value` is the provider's result, and `null` is a result.
- The caller cancels a call with `$/cancelRequest`; it fails with `plugin.host.cancelled`.
- Core checks in a fixed order (the caller declared the function, the provider is enabled, the provider exports it, the versions are equal, the context, a local workspace, the params size, the call it belongs to, the call chain, the depth, the concurrency) and refuses before anything is written to the provider; the first failing check decides the code.

#### What the provider receives

Core forwards a relayed call to the provider as `lithe/function/invoke`. As for any function call, the request has top-level `functionID`, `operationID`, `context`, and `params`; a relayed call adds one more top-level field, `caller: {pluginID}`.

- Core writes `caller.pluginID` from the identity of the calling session. It sits outside `params` and the caller cannot supply it: a `functions/invoke` request carrying a top-level `caller` is refused with `-32018`. A function that decides by who asks, or keeps per-caller state, reads this field, never a plugin ID in `params`. A key named `caller` inside the caller's own `params` is ordinary data and is passed unchanged.
- A call the host itself started has no `caller`; absent means the host. A provider that negotiated version 3 or earlier never receives the field; a provider that relies on it must negotiate version 4.
- `operationID` identifies the call being served. A provider that makes a nested `functions/invoke` while serving a relayed call sets `parentOperationID` to this `operationID`, so Core knows which call the new one belongs to. `parentOperationID` is a string of 1 to 128 bytes, otherwise `-32602`, `plugin.host.invalidParams`. Without it, Core treats the new call as a child of every relayed call the backend is serving, so a backend should always set it.
- The call chain: the chain of a new call is the chain of its parent plus the caller. A call to a plugin already on the chain, including a plugin calling itself, is `plugin.function.callCycle`, and a chain has at most 4 calls. Cancelling, timing out, or ending the parent call cancels its children, and each provider receives `$/cancelRequest`. If `parentOperationID` names a call the host started directly, the new call starts a new chain; if it names a call this backend is not serving (finished, or never existed), the call is refused with `plugin.host.cancelled`.
- If the provider's backend is not running, Core waits for it and emits the runtime event `functionProviderStartRequired` on the calling session; the host starts the backend as an ordinary plugin start. If the provider has not negotiated the protocol and registered the function within 30 seconds, the call ends with `plugin.function.providerUnavailable`.

#### Trust is not transitive

- The provider runs with its own permissions, state directory, and credentials. The caller gains none of the provider's, and the provider gains none of the caller's. The caller's plugin ID appears only in the `caller` field, never in the provider's session.
- An exported function must not accept an executable path, command line, or script from its caller and start a process with it, nor read or write a path the caller chose without checking it against what the provider itself may access. If a caller needs the provider to run something, the function takes a request the provider interprets (a task name, a document) and the provider decides the executable.
- This is not an install dependency: installing the caller does not install a provider, and disabling a provider only leaves calls without one; it takes no part in the install closure and does no version solving. A call whose context is a remote workspace is refused with `plugin.host.authorityUnsupported`; the exported `context: "workspace"` means a local workspace only, and backends still run on the machine that runs Lithe.
- Disabling, enabling, replacing, and uninstalling a plugin take effect for the relay at once, with no restart. macOS registers backend plugins only when it launches, so a newly installed or updated backend plugin appears in `providers` only after Lithe restarts.

#### Error codes

| Code | When |
| --- | --- |
| `plugin.function.notDeclared` | The caller did not declare the function; also when a caller update stops declaring it while the call runs |
| `plugin.function.providerDisabled` | The provider is not installed or not enabled; also when, while the call runs, the provider is disabled or uninstalled, or its update stops exporting the function or changes its version |
| `plugin.function.notExported` | The provider's backend has not registered the function, or has withdrawn it |
| `plugin.function.versionMismatch` | The version the caller declared differs from the version the provider exports |
| `plugin.function.callCycle` | The provider is already on the call chain, including the caller itself |
| `plugin.function.depthExceeded` | The chain would have more than 4 calls |
| `plugin.function.tooManyCalls` | The caller already has 8 relayed calls running |
| `plugin.function.payloadTooLarge` | The params are over 256 KiB, or the provider's result is over 256 KiB (over 1 MiB is `invalidResult`) |
| `plugin.function.providerUnavailable` | The provider's backend was not ready within 30 seconds, or it exited or withdrew the function while the call ran |

A provider's own failure keeps its meaning: when its function answered with an error, the code is `plugin.function.failed` (`-32070`) and `details.pluginErrorCode` is the provider's own `error.data.code`; a reply with no `result`, or a result over 1 MiB serialized, is `plugin.function.invalidResult` (`-32071`). A workspace context that is closed or replaced by a newer generation is `plugin.host.contextExpired`, and the deadline passing is `plugin.host.timeout`. Core never restarts a backend or retries a call.

### Run and debug sessions (protocol 3)

A backend can start a run or a debug session that the host presents as its own: the backend decides what to start, and the host owns the process, the console, the Stop button, and the end of the session. A Run session adds host presentation (a Run window entry, the console, the exit status) and grants no other capability. `run/start` and `debug/start` take `context: {contextID, generation}` and the permission shown; `run/stop` takes no `context`, so a run can still be stopped after the context it started in has closed.

| Method (prefix `lithe/host/`) | Permission | Params | Result |
| --- | --- | --- | --- |
| `run/start` | `run.start` | `context, label, process, temporaryFiles?, debug?, rerun?` | `{runID}` |
| `run/stop` | `run.start` | `{runID}` or `{sessionID}` | `null` |
| `debug/start` | `debug.start` | `context, label, adapter, request, configuration` | `{sessionID}`; see [Debug adapters](debug-adapters.md#starting-a-debug-session-from-a-plugin-backend) |

Notification (the backend does not reply): `lithe/host/run/didExit` with `{runID, exitCode?, signal?}`, granted by `run.start`. `run.start` requires `backend` 2 and grants `run/start`, `run/stop`, and `run/didExit`.

`run/start` parameters:

- `label`: 1 to 256 characters without control characters; it titles the Run window entry.
- `process`: `{executable, arguments, workingDirectory, environment}`, all four required; an unknown field is refused. `executable` is `{kind: "tool", tool}` (`tool` is a tool-definition ID that the manifest lists in `requiresTools`, which the host resolves through the layers described in [Toolchains](toolchains.md)) or `{kind: "packaged", path}` (a normalized relative path of a file inside the plugin package). Arguments, the working directory (relative to the workspace root and an existing directory inside it), and the environment have the bounds of [run preparation](../../contract/v1/README.md#provider-run-preparation-buildproviders-4).
- `temporaryFiles`: at most four `{name, contents}` files of up to 4 MiB of text each. The host writes them into its own temporary directory, removes them when the run ends, and replaces `${temporaryFile:<name>}` in `process.arguments` and `process.environment` with the file's path, so the plugin never needs to know where the host keeps temporary files. A `name` is unique, 1 to 64 letters, digits, `.`, `_`, or `-`, and starts with a letter or digit.
- `debug`: `{configurationType, target}`, which makes the run a debug session; see [Debug adapters](debug-adapters.md#starting-a-debug-session-from-a-plugin-backend).
- `rerun`: `{function, data}`, one of the plugin's registered functions and its params (a JSON object of up to 64 KiB). When the user chooses Run again, the host invokes the function with `data` as the params and the current `context`, and the backend answers with a new `run/start`, so the backend prepares the restart; without `rerun` the entry cannot be rerun.

Other rules:

- `run/stop` stops a run by `runID` or a debug session by `sessionID` and answers `null`, also when it already ended; cancelling the host request is the same as stopping. A backend can stop only runs and sessions it started.
- The host sends `run/didExit` when the process ends, once per run and only to a session that holds the `run.start` grant. `exitCode` is the exit status and `signal` (1 to 127) the signal that ended it; both are absent when the host lost the process.
- Malformed params are `plugin.host.invalidParams`, and Core's rejection (`plugin.run.invalidStart` or `plugin.debug.invalidStart`) is in `details.code` and `details.field`; a closed context is `plugin.host.contextExpired`; a `tool` that resolves to no executable is `plugin.host.toolUnresolved`.
- The host keeps running processes when the plugin is disabled or its backend exits; it only stops telling the backend. A backend that starts again does not receive `run/didExit` for a run it did not see begin.
- To make output in the Run console clickable, declare matchers with `outputLinkMatchers` 1: Core does the matching and the `resolve` function is called only when the user clicks; see [Output link matchers](../../contract/v1/README.md#output-link-matchers-outputlinkmatchers-1).
- Windows has no plugin backend host, so it has none of these methods.

### Contract sections

| Section | Content |
| --- | --- |
| [Plugin backend](../../contract/v1/README.md#plugin-backend) | Declaration, state directory, when it starts and stops, per-platform archives |
| [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) | Start routes, `onWorkspaceOpen` never blocks the workspace, `workspace.metadata` |
| [Plugin functions](../../contract/v1/README.md#plugin-functions) | Protocol overview |
| [Capability ownership](../../contract/v1/README.md#capability-ownership) | Plugin functions never replace a domain contract |
| [Declarative or backend implementation](../../contract/v1/README.md#declarative-or-backend-implementation) | When to use declarative data and when to write a backend |
| [Negotiation](../../contract/v1/README.md#negotiation) | Version negotiation in `initialize` |
| [References and IDs](../../contract/v1/README.md#references-and-ids) | Function references and ID rules |
| [Registration and withdrawal](../../contract/v1/README.md#registration-and-withdrawal) | Registering and withdrawing functions |
| [Invocation](../../contract/v1/README.md#invocation), [Progress](../../contract/v1/README.md#progress) | Calls, results, progress, cancellation |
| [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) | Credentials service |
| [Workspace contexts and capability discovery (protocol version 3)](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) | `getCapabilities`, `contextID`/`generation` contexts, host failure codes |
| [Host notifications (protocol version 3)](../../contract/v1/README.md#host-notifications-protocol-version-3) | `workspace/didChangeContexts`, `subscriptions/notify`, `commands/didRun`, `run/didExit` |
| [Extending version 3](../../contract/v1/README.md#extending-version-3) | How protocol 3 gains methods without a new version |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | File reads, watches, search, and window messages (document and editor methods are on [Editor](editor.md)) |
| [Plugin-owned native terminals (function protocol 3)](../../contract/v1/README.md#plugin-owned-native-terminals-function-protocol-3), [Close, retained context and reopen](../../contract/v1/README.md#close-retained-context-and-reopen) | Plugin-owned native terminals |
| [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) | `run/start`, `run/stop`, `debug/start`, `run/didExit`: params, limits, failure codes |
| [Cross-plugin calls (protocol version 4)](../../contract/v1/README.md#cross-plugin-calls-protocol-version-4) | The `pluginFunctions` manifest declarations, `functions/providers`, `functions/invoke`, the check order, the call chain, the `caller` field, providers that are not running, hot activation |
| [Output link matchers (`outputLinkMatchers` 1)](../../contract/v1/README.md#output-link-matchers-outputlinkmatchers-1) | Links in Run and Test console output; the `resolve` function is called on click |
| [Structured plugin diagnostics (logging protocol 1)](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) | `lithe/log` fields and limits |
| [Compatibility](../../contract/v1/README.md#compatibility), [Official backend SDK](../../contract/v1/README.md#official-backend-sdk) | Compatibility rules and the official SDKs |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | Outcomes and error codes |

### Examples and cases

| File | Content |
| --- | --- |
| [`backend/backend.plugin.json`](../../contract/v1/fixtures/backend/backend.plugin.json) | `backend` 1: a web tool window calls the backend, which uses the credentials service |
| [`backend/backend-workspace.plugin.json`](../../contract/v1/fixtures/backend/backend-workspace.plugin.json) | `backend` 2: starts on `onWorkspaceOpen`, uses `workspace.metadata` |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `backend` 2 with `ui` 4: a command without a tool window that reads and edits documents |
| [`backend/terminal-backend.plugin.json`](../../contract/v1/fixtures/backend/terminal-backend.plugin.json) | `backend` 2: the smallest manifest for plugin-owned native terminals |
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | Canonical plugin-function messages: negotiation (including protocol 4), registration, invocation, results, progress, 93 host requests, 23 host failures, 29 host notifications, plus the `caller` of a relayed call (`invocationCallers`) and the relay failure codes (`relayErrors`) |
| [`backend/plugin-functions-exporter.plugin.json`](../../contract/v1/fixtures/backend/plugin-functions-exporter.plugin.json) | `pluginFunctions` 1: only exports two functions (`document.format` version 1, `symbols.list` version 2, `context` `workspace`), with no page, command, or `onWorkspaceOpen` |
| [`backend/plugin-functions-caller.plugin.json`](../../contract/v1/fixtures/backend/plugin-functions-caller.plugin.json) | `pluginFunctions` 1: declares a call to `document.format` version 1, starts on `onWorkspaceOpen` |
| [`backend/plugin-logging.json`](../../contract/v1/fixtures/backend/plugin-logging.json) | `litheLogging` 1: offer and answer, notifications, rejections, levels |
| [`run/run-start.json`](../../contract/v1/fixtures/run/run-start.json) | The params of `run/start` and what `run.decodeStart` returns for each of its 17 cases (tool and packaged executables, `debug` and `rerun`, the refusals); temporary files depend on the host's temporary directory and are covered by Core's own tests |
| [`run/tool-reference-launch.plugin.json`](../../contract/v1/fixtures/run/tool-reference-launch.plugin.json) | The manifest `run-start.json` uses: `requiresTools` lists the tool `interpreter`, and it declares `run.start`, `debug.start`, and `outputLinkMatchers` 1 |
| [`run/output-link-matchers.json`](../../contract/v1/fixtures/run/output-link-matchers.json) | The requests of `outputLinks.match` and the links it returns, plus what `outputLinks.decodeResolution` makes of a matcher's `resolve` reply |

Backend rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json).

### Related guides

- [Plugin backend: workspace contexts and host services](../../guides/plugin-backend-workspace.md)
- [Open a native Lithe terminal from a plugin](../../guides/native-terminals.md)
- [Plugin logging](../../guides/plugin-logging.md)
