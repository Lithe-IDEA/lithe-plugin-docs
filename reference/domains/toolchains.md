# 工具链 / Toolchains

[中文](#中文) · [English](#english)

## 中文

插件可以告诉 Lithe 一个工具（JDK、Maven、Node、Go 等）长什么样、去哪里找、怎么判断版本，也可以用后端回答"这台机器装了哪些版本、这个项目要哪个版本"。宿主没有内置任何工具定义、别名表或按工具名的分支：工具定义只来自已启用的插件，Core 负责校验和选择，宿主只做文件系统观察、运行版本探针和启动后端这些本机 I/O。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `toolDefinitions[]`（工具定义）、`toolchainProviders[]`（工具链提供者）；`executables` 候选集里的 `toolchain` 候选 |
| 能力版本 | 工具定义：`toolDefinitions` 1 + `executableResolution` 2；提供者：`toolchainProviders` 1 + `backend` 2 + `serverProcess` 1；用工具定义 ID 启动进程：`serverProcess` 4；都是级别 3。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheToolchains` 1：Core 只向声明了提供者的插件会话提供 `experimental.litheToolchains: {versions: [1], providers}`，后端回答 `{version: 1, providers}`；请求方法是 `toolchains/query`。见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | [API-23](../api-catalog.md#现在能用的-api)：工具链与受管进程（部分完成） |
| 权限 | 工具定义不需要权限；提供者需要 `backend.launch`、`toolchains.provide` |
| 激活事件 | 不需要；提供者被宿主的第一次查询唤起，不需要页面或启动事件 |
| 后端启动 | 提供者的后端只能是包内可执行文件（`packaged`），或包内运行时（`runtime` + `packagedRuntime`），不能依赖需要发现的运行时 |
| Windows | 不支持；Windows 宿主没有工具链适配器，不声明 `toolDefinitions`、`toolchainProviders`、`executableResolution` |
| 官方 SDK | 三个官方 SDK 都还没有实现 `litheToolchains`，提供者后端要自己处理 `toolchains/query` |

### 版本对照

| 版本 | 新增 | Core | macOS | Windows | 官方 SDK |
| --- | --- | --- | --- | --- | --- |
| `executableResolution` 1 | [共享可执行文件候选](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12)：设置、环境变量、命令、宿主工具链等候选的排序和严格选择 | 支持 | 支持 | 不支持 | 不涉及 |
| `executableResolution` 2 | 工具定义里的候选，构建候选集里的 `toolchain` 候选可以指向插件定义的工具；不改变已有构建候选集 | 支持 | 支持（只读设置、构建与运行工具） | 不支持 | 不涉及 |
| `toolDefinitions` 1 | 工具身份、可执行文件布局、版本探针、声明式发现、主目录变量、导出的环境变量、`PATH` 前缀、启动模板 | 支持 | 支持（只读设置、构建与运行工具） | 不支持 | 不涉及 |
| `toolchainProviders` 1 | 提供者声明 `{id, displayName, tools, watchFiles?}`；按权限校验应答 | 支持 | 支持（只读设置、构建与运行工具） | 不支持 | 不涉及 |
| `serverProcess` 4 | 进程启动按工具定义 ID 引用工具：清单顶层 `requiresTools`，启动计划是 `launch.kind: "tool"`，`toolchains.instantiateLaunch` 按定义的 `launch` 模板展开；见下文“按工具定义 ID 启动进程” | 支持 | 宿主目前只声明 `serverProcess` 1 和 3：版本 4 已由 Core 校验，宿主尚未声明，需要它的插件版本暂时装不上 | 只声明 `serverProcess` 1 | 不涉及 |
| `litheToolchains` 1 | `toolchains/query` 请求和应答；后端从声明的提供者中选出非空子集 | 支持 | 支持 | 不支持 | 都未实现 |
| `.lithe/toolchains/local.json` `version` 1 | 机器本地文档：旧的 `toolchains` 条目和按目录的 `pins` | 支持读取和解析 | 只读，不保存 | 不支持 | 不涉及 |

### 两条安装来源

一个工具可以走其中一条、两条都走，或者都不走（只靠固定、项目文件、全局默认和登录 shell 里的命令）。

| 来源 | 由谁声明 | 适合 | 谁执行 |
| --- | --- | --- | --- |
| 声明式发现 | 工具定义里的 `discovery`、`homeLayouts`、`homeVariable`、`commands`、`version` | 目录形状固定的安装：用户主目录、环境变量或系统绝对目录下的单段 `*` 通配，可选的主目录后缀，加上对版本探针输出的正则 | 宿主展开 `discoveryPlan` 的扫描，不运行插件代码 |
| 提供者查询 | `toolchainProviders[]` 回答 `toolchains/query` | 声明式原语表达不了的位置：注册表、属性列表、版本管理器数据库、命令输出、版本管理器自己的版本文件 | 提供者的包内后端 |

声明式原语已经冻结：根目录只有 `home`、`environment`、`absolute` 三种，通配只有单段 `*`。新的安装形状由提供者回答，不再增加根目录类型或宿主解析器。没有 `watchFiles` 的提供者只报告安装，应答里不能有项目版本要求。

### 选择顺序

| 层 | 规则 |
| --- | --- |
| 固定（pin） | 最近的祖先目录的固定；只按完整路径段匹配，前缀相似的兄弟目录不继承。固定不可用时直接失败，不回退 |
| 项目 | 最近的声明目录；同一深度 wrapper > 环境变量 > 版本声明；同一个文件里原生读取器优先于兼容读取器。离得近的项目要求遮住更上层的要求 |
| 全局 | Lithe 的全局默认 > 没有歧义（或用户已选）的提供者全局默认 > 登录 shell |
| 自动 | 已安装、满足最低版本的最高版本；手动添加的安装只能被固定，不参与自动选择。不同主目录有相同最高版本时算冲突，要求用户选择 |

没有安装的项目要求会带着原因保留为"未满足"，即使更低的层选中了别的安装。冲突必须由用户明确选择，不按插件 ID 或路径先后决定。一次操作还可以带 `fixed: {home}` 或 `fixed: {executable}`，它排在目录固定之前，无效时直接失败。

### 工具 ID 与冲突

- 任何插件都可以定义短 ID（例如 `compiler`）或限定 ID `<自己的插件 ID>:<本地 ID>`。限定 ID 的前缀必须是定义它的插件自己的 ID，所以不同插件的限定 ID 永远不会冲突。宿主不提供来源证明，插件自称的来源也不会被读取。提供者可以回答短 ID。
- 同一个短 ID 被多个已启用插件定义时，它不归其中任何一方：注册表不登记这些定义，其余定义照常登记；`discoveryPlan` 不给它列扫描和提供者；解析它的消费者把它当作没有人声明。归属从不按到达顺序或插件 ID 顺序决定，否则恶意插件可以靠先加载或取更小的 ID 抢走工具。
- 注册表的输出带 `conflicts: [{tool, pluginIDs}]`：每个被争用的工具一条，按 `tool` 排序，`pluginIDs` 是所有定义它的插件 ID（也排序）。宿主应该告诉用户是哪些插件在争用；macOS 设置页对每个冲突显示一条诊断，建议只保留其中一个插件启用。
- 同一个插件 ID 在一批输入里重复出现，仍然拒绝整批，不会替换先前登记的归属。

### Core 命令

所有命令都只处理 UTF-8 JSON，输入上限 4 MiB，不做任何本机 I/O。成功返回 `{status: "ok", <键>: 值}`，拒绝返回 `{status: "rejected", error: {code, message, field?}}`。

| 命令 | 输入 | 成功时的键 | 用途 |
| --- | --- | --- | --- |
| `toolchains.registry` | `{plugins: [{manifest}]}` | `registry` | 重新校验完整清单，汇总定义和提供者，返回 `{definitions, providers, conflicts}`；宿主不提供来源证明；被多个插件争用的短 ID 不登记，列在 `conflicts: [{tool, pluginIDs}]`；最多 128 个插件、512 个定义、256 个提供者 |
| `toolchains.decodeReply` | 查询请求、提供者应答、宿主观察到的文件系统事实 | `snapshot` | 校验一次 `toolchains/query` 应答，得到 `{installations, statements, watch, diagnostics}` |
| `toolchains.failureReply` | `{params, code}` | `reply` | 查询没有应答时生成对应的 JSON-RPC 错误；`code` 只能是 `plugin.toolchain.timeout` 或 `plugin.toolchain.providerUnavailable` |
| `toolchains.resolvePins` | `{document, directory, tool}` | `resolution` | 在 `local.json` 里找最近的固定，返回 `{pin, sourceDirectory}` |
| `toolchains.resolve` | 某个目录的完整解析输入 | `resolution` | 走四层选择，结果是 `resolved`、`skip` 或 `terminalFailure`，附带 `trace`、`unmet`、`conflicts` |
| `toolchains.checkVersion` | `{registry, tool, output, minimumVersion?}` | `version` | 用定义的版本正则取版本并比较；最多 8 段数字、64 字节，不解释别名 |
| `toolchains.instantiateLaunch` | `{registry, tool, executable, home?, entry?, runtimeArguments, arguments}` | `launch` | 按工具定义的 `launch` 模板，为宿主选中的可执行文件展开启动参数，返回 `{executable, arguments, versionArguments}`；见“按工具定义 ID 启动进程” |
| `toolchains.discoveryPlan` | `{registry, platform}` | `plan` | 每个工具的扫描计划和声明它的提供者；最多 256 个根目录、4096 个条目、64 层祖先 |
| `toolchains.candidates` | 设置页的发现输入 | `candidates` | 设置页显示的全局层和自动层：`{tool, shell, installations, skipped, selected, conflicts}`；第一次调用把待探测的可执行文件列为 `probeFailed`，宿主运行探针后再调用一次 |

Core 另有 `toolchains.resolveConsumerPreferences` 和 `toolchains.editConsumerPreferences`，读取和编辑 `.lithe/run/local.json` 里 `toolchain.preferences` 的按消费者工具选择，只给宿主自己的运行和构建配置使用。这两个命令不属于插件契约。

### 提供者应答的限制

| 项目 | 规则 |
| --- | --- |
| 应答内容 | `result: {operationID, generation, providerID, installations, statements, watch}`；安装是 `{tool, version, home}`，版本要求是 `{tool, scope, source, request, installation, kind, reader}` |
| 大小 | 最多 1 MiB、512 个安装、256 条要求、256 个监视路径 |
| 路径 | 文件系统事实来自宿主，不来自提供者；主目录不能在任何工作区根目录里，必须符合定义的可执行文件布局 |
| 身份 | JSON-RPC `id`、`operationID`、`generation`、`providerID` 必须和请求一致；数字 `1` 和字符串 `"1"` 是不同的 ID；远程 authority 一律拒绝 |
| 版本要求 | `request` 的写法对 Core 不透明（例如 `lts`），宿主不解释；`installation: null` 表示要求没有被满足 |

提供者可以在 JSON-RPC 错误里使用下面的错误码，`decodeReply` 按这张表校验（一个数字只对应一个语义）：

| JSON-RPC | 语义码 | 含义 |
| --- | --- | --- |
| `-32601` | `plugin.toolchain.methodUnavailable` | 查询方法或能力没有协商 |
| `-32602` | `plugin.toolchain.invalidParams` | 请求格式错误或声明的输入无效 |
| `-32800` | `plugin.toolchain.cancelled` | 调用方取消了请求 |
| `-32070` | `plugin.toolchain.permissionDenied` | 缺少权限、审查或启用状态 |
| `-32071` | `plugin.toolchain.authorityUnsupported` | 没有这个 authority 的适配器 |
| `-32072` | `plugin.toolchain.timeout` | 超过本地截止时间 |
| `-32073` | `plugin.toolchain.providerUnavailable` | 后端启动失败、断开或上游查询失败 |
| `-32074` | `plugin.toolchain.invalidReply` | 应答格式错误，或安装引用了别的工具 |
| `-32075` | `plugin.toolchain.limitExceeded` | 超过应答、扫描或监视预算 |
| `-32076` | `plugin.toolchain.unsafePath` | 绝对路径、包含关系或可执行文件证据不安全 |
| `-32077` | `plugin.toolchain.operationExpired` | 发出查询时上下文或代次已经失效 |

### 按工具定义 ID 启动进程（`serverProcess` 4）

插件用工具定义 ID 启动进程时（`launch.kind: "tool"` 的启动计划，或后端的 `run/start` 带 `{kind: "tool"}`），从不自己写这个工具的 argv。定义的 `launch` 模板说明工具怎样接收入口文件、运行时参数和程序参数，宿主和插件都不按工具名分支。

| 项目 | 规则 |
| --- | --- |
| 声明 | `requires.serverProcess` 为 4 时，清单顶层 `requiresTools` 列出插件启动进程用到的所有工具定义 ID：最多 32 个，短 ID 或限定 ID，不重复，保存时排序。只有版本没有列表，或只有列表没有版本，都是 `plugin.manifest.invalid`。定义来自插件自己或别的已启用插件的 `toolDefinitions` |
| 引用位置 | `runtime` 启动的 `launch.tool`、`runtimeDependencies[].tool`、调试适配器的 `runSelectedRuntime.tool`。引用 `requiresTools` 没列出的 ID 是 `plugin.runtime.undeclared`，带 `field`；版本 1 到 3 仍用宿主的封闭运行时列表，报 `plugin.runtime.unsupported` |
| 启动计划 | 返回各部分而不是命令行：`launch: {kind: "tool", tool, entry, runtimeArguments, minimumVersion?, packagedRuntime?}`；`arguments` 只含插件自己的参数；`runtimeRequirements` 列出宿主要解析的每个工具。`packagedRuntime` 是插件自带运行时的路径，宿主直接把这个文件当作工具的可执行文件，不再查找 |
| `run/start` | `process.executable` 可以是 `{kind: "tool", tool}`（`tool` 是清单 `requiresTools` 列出的工具定义 ID，宿主按四层选择解析），或 `{kind: "packaged", path}`（插件包内文件）；见[插件后端](backend.md) |
| 宿主做什么 | 按四层选择解析每个工具，用 `toolchains.checkVersion` 检查 `minimumVersion`，再用 `toolchains.instantiateLaunch` 展开，然后启动 |
| 解析不到 | 工具解析不到可执行文件是 `plugin.host.toolUnresolved`（JSON-RPC `-32025`）；`details` 是 `{tool, reason}`，`reason` 为 `notFound`（四层里没有候选满足定义和最低版本）或 `unusable`（选中的可执行文件不能用） |

`toolchains.instantiateLaunch` 的 `executable` 必填；`home` 是解析出的工具主目录，`entry` 是绝对文件路径，`runtimeArguments` 和 `arguments` 是字符串列表。文本最多 4096 字节，列表最多 1024 项、每项最多 64 KiB，都不含 NUL。模板里的 `${executable}`、`${home}`、`${entry}`、`${runtimeArguments}`、`${arguments}` 被对应的值替换，列表原位展开；其他占位符原样保留。调用方给了模板带不动的值（给不启动入口的工具一个 `entry`，或给不接受参数的工具参数），或漏了模板需要的 `${home}`、`${entry}`，是 `plugin.toolchain.launchMismatch`；未知或已停用的 `tool` 是 `plugin.toolchain.invalidParams`，`field` 为 `tool`。这个命令不做任何本机 I/O：选可执行文件、探测版本、启动进程都由宿主完成。

### 宿主现状

| 用途 | macOS | Windows |
| --- | --- | --- |
| 设置页显示发现结果 | 两条来源都跑，只读；监视的文件变化时重新加载整个快照；不保存任何东西 | 不支持 |
| 构建工具、插件运行方案的可执行文件 | 候选或运行方案指向插件定义的工具（不在宿主旧的工具链表里）时，单独重新解析一次；设置页的发现结果不会被当作启动许可 | 不支持 |
| Java、语言服务器 | 仍然走原来的适配器 | 不支持 |
| 后端决定启动方式的 `{kind: "tool"}` 应答 | 按用户的工具选择和发现解析；位于项目内的程序被拒绝，因为打开项目不能运行它 | 不支持 |
| 命名冲突 | 设置页对每个被争用的短 ID 显示一条诊断，建议只保留其中一个插件 | 不支持 |
| 保存固定、`toolchainConsumers`、通用运行时启动、提供者热激活 | 尚未提供 | 尚未提供 |

`toolchains.provide` 可以影响某个已定义工具启动哪个程序，但不授权替换任意命令、安装软件、访问网络或执行工作区代码。任何插件都可以定义短 ID 或 `<自己的插件 ID>:<本地 ID>`，提供者也可以回答短 ID，宿主不提供来源证明；短 ID 被多个插件定义时的处理见上文“工具 ID 与冲突”。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Plugin-owned toolchains (Core foundation)](../../contract/v1/README.md#plugin-owned-toolchains-core-foundation) | 契约总入口里的工具链说明和当前宿主状态 |
| [Plugin-owned toolchains: Core foundation](../../contract/v1/toolchains/README.md) | 已实现部分的完整说明 |
| [Manifest admission and registry](../../contract/v1/toolchains/README.md#manifest-admission-and-registry) | 能力和权限要求、定义字段、提供者字段、ID 规则（短 ID 与限定 ID） |
| [Two installation routes](../../contract/v1/toolchains/README.md#two-installation-routes) | 声明式发现和提供者查询 |
| [Commands and envelopes](../../contract/v1/toolchains/README.md#commands-and-envelopes) | 九个 Core 命令、返回格式、注册表限制、`conflicts` |
| [Launching a tool definition](../../contract/v1/toolchains/README.md#launching-a-tool-definition) | `toolchains.instantiateLaunch`：按定义的 `launch` 模板展开启动参数、`launchMismatch` |
| [Tool references (`serverProcess` 4)](../../contract/v1/README.md#tool-references-serverprocess-4) | 清单 `requiresTools`、引用位置、启动计划的各部分、`plugin.host.toolUnresolved` |
| [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) | `run/start` 的 `process.executable`：`{kind: "tool"}` 或 `{kind: "packaged"}` |
| [Provider reply validation](../../contract/v1/toolchains/README.md#provider-reply-validation) | `decodeReply`、应答格式与限制、`failureReply` |
| [Pins and snapshot resolution](../../contract/v1/toolchains/README.md#pins-and-snapshot-resolution) | `local.json`、`resolve`、四层选择、`checkVersion`、`discoveryPlan` |
| [Settings candidates](../../contract/v1/toolchains/README.md#settings-candidates) | `candidates` 和两次调用的探测流程 |
| [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | `executableResolution` 1、2 的候选集 |
| [Provider backend negotiation](../../contract/v1/toolchains/README.md#provider-backend-negotiation) | `litheToolchains` 协商（`initialize` 里的 `experimental.litheToolchains`）、`lsp.toolchainRequest`、超时 |
| [Error replies](../../contract/v1/toolchains/README.md#error-replies) | 错误回复的格式和 11 个稳定失败码的数字码对照 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`toolchains/tool-definition.plugin.json`](../../contract/v1/fixtures/toolchains/tool-definition.plugin.json) | 生产形态的工具定义：`java`，8 个候选（项目/全局设置、项目/全局工具链请求、macOS/Linux 与 Windows 的 `JAVA_HOME`、`java` 命令、已安装的工具链），在 macOS 的 `JavaVirtualMachines` 目录里发现 `*.jdk` |
| [`toolchains/toolchain-provider.plugin.json`](../../contract/v1/fixtures/toolchains/toolchain-provider.plugin.json) | 生产形态的提供者：`mise`，覆盖 `java`、`node`、`maven`、`pnpm`、`go`，监视 6 个文件（`.tool-versions` 用兼容读取器），后端是包内可执行文件 |
| [`run/tool-reference-launch.plugin.json`](../../contract/v1/fixtures/run/tool-reference-launch.plugin.json) | 级别 3、`serverProcess` 4：清单顶层 `requiresTools` 列出插件自己定义的 `interpreter`，语言服务器的 `runtime` 启动按这个工具定义 ID 引用运行时 |

Core 的验收测试在主仓库的 [`toolchain_definitions.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/tests/toolchain_definitions.rs) 和同目录的 `toolchain_pins.rs`、`toolchain_queries.rs`、`toolchain_resolution.rs`。这三个测试共用的固定值文档、查询往返和每个稳定失败码的错误回复内联在 [`toolchain_vectors.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/tests/toolchain_vectors.rs) 里。

### 相关指南

- [为语言插件接入构建、运行与测试](../../guides/build-tool-integration.md)：构建候选集里的 `toolchain` 可执行文件
- [原生构建、测试与运行准备](../../guides/native-build-workflows.md)：运行准备返回的 `executable.kind: "toolchain"`

---

## English

A plugin can tell Lithe what a tool (a JDK, Maven, Node, Go and so on) looks like, where to find it and how to read its version, and a backend can answer which versions this machine has and which version a project asks for. The host has no built-in tool definitions, alias table or tool-name branches: definitions come only from enabled plugins, Core validates and selects, and the host performs only native I/O such as filesystem observation, version probes and backend launches.

| Item | Value |
| --- | --- |
| Manifest fields | `toolDefinitions[]` (tool definitions), `toolchainProviders[]` (toolchain providers); `toolchain` candidates in `executables` candidate sets |
| Capability versions | Definitions: `toolDefinitions` 1 + `executableResolution` 2; providers: `toolchainProviders` 1 + `backend` 2 + `serverProcess` 1; starting processes by tool-definition ID: `serverProcess` 4; all at level 3. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheToolchains` 1: Core offers `experimental.litheToolchains: {versions: [1], providers}` only to a plugin session that declares providers, and the backend answers `{version: 1, providers}`; the request method is `toolchains/query`. See [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | [API-23](../api-catalog.md#apis-usable-now): toolchains and managed processes (partial) |
| Permissions | Definitions need none; providers need `backend.launch` and `toolchains.provide` |
| Activation events | None needed; the host's first query starts a provider, without a page or startup event |
| Backend launch | A provider backend is a packaged executable (`packaged`) or a packaged runtime (`runtime` + `packagedRuntime`), without discovered runtime dependencies |
| Windows | Not supported; the Windows host has no toolchain adapter and does not advertise `toolDefinitions`, `toolchainProviders` or `executableResolution` |
| Official SDKs | None of the three official SDKs implements `litheToolchains` yet; a provider backend handles `toolchains/query` itself |

### Versions compared

| Version | Adds | Core | macOS | Windows | Official SDKs |
| --- | --- | --- | --- | --- | --- |
| `executableResolution` 1 | [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12): ordering and strict selection of setting, environment, command, host toolchain and other candidates | Yes | Yes | No | Not applicable |
| `executableResolution` 2 | Candidates inside tool definitions; a `toolchain` candidate in a build candidate set may name a plugin-defined tool; existing build candidate sets are unchanged | Yes | Yes (read-only settings, Build and run tools) | No | Not applicable |
| `toolDefinitions` 1 | Tool identity, executable layouts, version probe, declarative discovery, home variable, exported environment, `PATH` prefixes, launch template | Yes | Yes (read-only settings, Build and run tools) | No | Not applicable |
| `toolchainProviders` 1 | Provider declarations `{id, displayName, tools, watchFiles?}`; permission-bound reply validation | Yes | Yes (read-only settings, Build and run tools) | No | Not applicable |
| `serverProcess` 4 | A process launch references a tool by tool-definition ID: top-level `requiresTools`, a `launch.kind: "tool"` launch plan, and `toolchains.instantiateLaunch` expanding the definition's `launch` template; see "Starting a process by tool-definition ID" below | Yes | The host currently advertises only `serverProcess` 1 and 3: version 4 is validated by Core but the host does not advertise it yet, so a plugin version that needs it cannot be installed for now | Advertises only `serverProcess` 1 | Not applicable |
| `litheToolchains` 1 | The `toolchains/query` request and reply; the backend selects a nonempty subset of its declared providers | Yes | Yes | No | Not implemented |
| `.lithe/toolchains/local.json` `version` 1 | Machine-local document: legacy `toolchains` entries and per-directory `pins` | Reads and resolves | Read-only, never saved | No | Not applicable |

### Two installation routes

A tool may use either route, both, or neither (relying only on pins, project files, global defaults and login-shell commands).

| Route | Declared by | Fits | Runs |
| --- | --- | --- | --- |
| Declarative discovery | The definition's `discovery`, `homeLayouts`, `homeVariable`, `commands` and `version` | Installations with a fixed directory shape: a single-component `*` glob under the user home, an environment variable or an absolute system directory, an optional home suffix, and a regex over the version probe output | The host expands the `discoveryPlan` scans; no plugin code runs |
| Provider query | `toolchainProviders[]` answering `toolchains/query` | Locations those primitives cannot express: registries, property lists, manager databases, command output, manager-specific version files | The provider's packaged backend |

The declarative primitives are frozen: roots stay `home`, `environment` and `absolute`, and globs stay single-component `*`. A new installation shape is answered by a provider, never by a new root kind or host parser. A provider without `watchFiles` only reports installations, and its replies may not contain project requirements.

### Selection order

| Layer | Rule |
| --- | --- |
| Pin | The nearest ancestor directory's pin, matched by whole path components, so a similarly prefixed sibling never inherits it. An unusable pin fails terminally without fallback |
| Project | The nearest declaring directory; at equal depth wrapper > environment > version statement; for the same file the native reader before the compatible reader. Nearer project requirements mask farther ones |
| Global | Lithe's global default > an unambiguous (or user-chosen) provider global default > login shell |
| Automatic | The highest installed version satisfying the minimum; manually added installations are pin-only and never automatic. Equal highest versions in different homes are a conflict that needs a user choice |

An uninstalled project requirement stays visible as unmet, with its reason, even when a lower layer selects another installation. Conflicts need an explicit user choice, never plugin-ID or path precedence. One operation may also carry `fixed: {home}` or `fixed: {executable}`, which precedes directory pins and fails terminally when invalid.

### Tool IDs and conflicts

- Any plugin may define a short ID (such as `compiler`) or a qualified ID `<own-plugin-id>:<local-id>`. The prefix of a qualified ID must be the defining plugin's own ID, so qualified IDs of different plugins never collide. The host supplies no proof of origin, and a plugin's own claim of one is not read. Providers may answer short IDs.
- A short ID defined by more than one enabled plugin belongs to none of them: the registry registers none of those definitions and registers every other definition as usual; `discoveryPlan` lists no scans or providers for it; and a consumer that resolves it treats it as declared by nobody. Ownership is never decided by arrival order or plugin ID order, because a hostile plugin could take a tool by being loaded first or by choosing a smaller ID.
- The registry output carries `conflicts: [{tool, pluginIDs}]`: one entry per contested tool, sorted by `tool`, with `pluginIDs` being the IDs of all plugins that defined it (also sorted). A host should tell the user which plugins contest the tool; the macOS settings page shows one diagnostic per conflict and advises keeping only one of those plugins enabled.
- A repeated plugin ID in one batch still rejects the whole batch, without replacing earlier owners.

### Core commands

Every command takes UTF-8 JSON with a 4 MiB input budget and performs no native I/O. Success is `{status: "ok", <key>: value}`; refusal is `{status: "rejected", error: {code, message, field?}}`.

| Command | Input | Success key | Purpose |
| --- | --- | --- | --- |
| `toolchains.registry` | `{plugins: [{manifest}]}` | `registry` | Revalidates complete manifests and collects definitions and providers, returning `{definitions, providers, conflicts}`; the host supplies no proof of origin; a short ID contested by several plugins is not registered and is listed in `conflicts: [{tool, pluginIDs}]`; at most 128 plugins, 512 definitions and 256 providers |
| `toolchains.decodeReply` | The query request, the provider reply and host-observed filesystem facts | `snapshot` | Validates one `toolchains/query` reply into `{installations, statements, watch, diagnostics}` |
| `toolchains.failureReply` | `{params, code}` | `reply` | Produces the JSON-RPC error for a query that got no reply; `code` is only `plugin.toolchain.timeout` or `plugin.toolchain.providerUnavailable` |
| `toolchains.resolvePins` | `{document, directory, tool}` | `resolution` | Finds the nearest pin in `local.json` and returns `{pin, sourceDirectory}` |
| `toolchains.resolve` | The complete resolver input for one directory | `resolution` | Runs the four layers; the outcome is `resolved`, `skip` or `terminalFailure`, with `trace`, `unmet` and `conflicts` |
| `toolchains.checkVersion` | `{registry, tool, output, minimumVersion?}` | `version` | Extracts the version with the definition's regex and compares it; at most 8 numeric components and 64 bytes, no aliases interpreted |
| `toolchains.instantiateLaunch` | `{registry, tool, executable, home?, entry?, runtimeArguments, arguments}` | `launch` | Expands the launch argv for the executable the host selected, using the tool definition's `launch` template, and returns `{executable, arguments, versionArguments}`; see "Starting a process by tool-definition ID" |
| `toolchains.discoveryPlan` | `{registry, platform}` | `plan` | Each tool's scan plan and the providers that declare it; at most 256 roots, 4096 entries and 64 ancestors |
| `toolchains.candidates` | A settings page's discovery input | `candidates` | The global and automatic layers a settings page shows: `{tool, shell, installations, skipped, selected, conflicts}`; the first call lists executables to probe as `probeFailed`, and the host calls again after running the probes |

Core also has `toolchains.resolveConsumerPreferences` and `toolchains.editConsumerPreferences`, which read and edit the per-consumer tool choices in `toolchain.preferences` of `.lithe/run/local.json` for the host's own Run and Build configuration. They are not part of the plugin contract.

### Provider reply limits

| Item | Rule |
| --- | --- |
| Content | `result: {operationID, generation, providerID, installations, statements, watch}`; an installation is `{tool, version, home}` and a requirement is `{tool, scope, source, request, installation, kind, reader}` |
| Size | At most 1 MiB, 512 installations, 256 requirements and 256 watch paths |
| Paths | Filesystem facts come from the host, never the provider; a home may not be inside any workspace root and must match the definition's executable layouts |
| Identity | The JSON-RPC `id`, `operationID`, `generation` and `providerID` must match the request; numeric `1` and string `"1"` are different IDs; remote authorities are always refused |
| Requirements | The `request` syntax (such as `lts`) is opaque to Core and the host never interprets it; `installation: null` records an unmet requirement |

A provider may use these codes in JSON-RPC errors, and `decodeReply` checks them against this table (one number, one meaning):

| JSON-RPC | Semantic code | Meaning |
| --- | --- | --- |
| `-32601` | `plugin.toolchain.methodUnavailable` | The query method or capability was not negotiated |
| `-32602` | `plugin.toolchain.invalidParams` | A malformed request or invalid declared input |
| `-32800` | `plugin.toolchain.cancelled` | The caller cancelled the request |
| `-32070` | `plugin.toolchain.permissionDenied` | A permission, review or enabled-state prerequisite is absent |
| `-32071` | `plugin.toolchain.authorityUnsupported` | No adapter for this authority |
| `-32072` | `plugin.toolchain.timeout` | The local deadline passed |
| `-32073` | `plugin.toolchain.providerUnavailable` | The backend failed to start, disconnected or failed its upstream query |
| `-32074` | `plugin.toolchain.invalidReply` | A malformed reply, or an installation that references another tool |
| `-32075` | `plugin.toolchain.limitExceeded` | A reply, scan or watch budget was exceeded |
| `-32076` | `plugin.toolchain.unsafePath` | Unsafe absolute path, containment or executable evidence |
| `-32077` | `plugin.toolchain.operationExpired` | The context or generation was already invalid when the query was sent |

### Starting a process by tool-definition ID (`serverProcess` 4)

A plugin that starts a process by tool-definition ID (a `launch.kind: "tool"` launch plan, or a backend `run/start` with `{kind: "tool"}`) never writes the tool's argv. The definition's `launch` template says how the tool takes an entry file, runtime arguments and program arguments, so neither the host nor the plugin branches on a tool name.

| Item | Rule |
| --- | --- |
| Declaration | With `requires.serverProcess` at 4, the top-level `requiresTools` lists every tool-definition ID the plugin starts a process with: at most 32, short or qualified, without duplicates, stored sorted. The version without the list, or the list without the version, is `plugin.manifest.invalid`. Definitions come from the plugin's own `toolDefinitions` or from another enabled plugin's |
| Where an ID is used | `launch.tool` of a `runtime` launch, `runtimeDependencies[].tool`, and `runSelectedRuntime.tool` of a debug adapter. An ID that `requiresTools` does not list is `plugin.runtime.undeclared` with the `field`; versions 1 to 3 keep the host's closed runtime lists and report `plugin.runtime.unsupported` |
| Launch plan | Returns parts, not a command line: `launch: {kind: "tool", tool, entry, runtimeArguments, minimumVersion?, packagedRuntime?}`; `arguments` holds only the plugin's own arguments; `runtimeRequirements` lists every tool the host must resolve. `packagedRuntime` is the path of a runtime the plugin ships; the host then uses that file as the tool's executable and never looks the tool up |
| `run/start` | `process.executable` can be `{kind: "tool", tool}` (`tool` is a tool-definition ID that the manifest's `requiresTools` lists; the host resolves it through the four layers) or `{kind: "packaged", path}` (a file in the plugin package); see [Plugin backend](backend.md) |
| What the host does | Resolves each tool through the four layers, checks `minimumVersion` with `toolchains.checkVersion`, expands the start with `toolchains.instantiateLaunch`, then starts it |
| Unresolvable tool | A tool that resolves to no executable is `plugin.host.toolUnresolved` (JSON-RPC `-32025`); `details` is `{tool, reason}`, with `reason` being `notFound` (no candidate of the four layers meets the definition and its minimum version) or `unusable` (a chosen executable cannot be used) |

`toolchains.instantiateLaunch` requires `executable`; `home` is the resolved tool home, `entry` an absolute file path, and `runtimeArguments` and `arguments` string lists. A text is at most 4096 bytes and a list at most 1024 items of 64 KiB each, none holding a NUL. In the template, `${executable}`, `${home}`, `${entry}`, `${runtimeArguments}` and `${arguments}` are replaced by the matching value, with a list expanding in place; any other token is passed as written. A value the caller supplies that the template cannot carry (an `entry` for a tool that launches none, or arguments for one that takes none), or a `${home}` or `${entry}` the caller left out, is `plugin.toolchain.launchMismatch`; an unknown or disabled `tool` is `plugin.toolchain.invalidParams` with `field: "tool"`. The command performs no native I/O: selecting the executable, probing the version and starting the process are the host's work.

### Host status

| Use | macOS | Windows |
| --- | --- | --- |
| Settings pages showing detections | Both routes run read-only; a change to a watched file reloads the whole snapshot; nothing is saved | No |
| Executables for Build tools and plugin run plans | A candidate or run plan that names a plugin-defined tool (not one in the host's legacy toolchain table) is resolved afresh; settings detections are never launch permits | No |
| Java and language servers | Still use their previous adapters | No |
| A `{kind: "tool"}` answer to a backend-resolved launch | Resolved with the user's tool choice and discovery; a program inside the project is refused, because opening a project must not run it | No |
| Name conflicts | The settings page shows one diagnostic per contested short ID and advises keeping only one of those plugins | No |
| Saving pins, `toolchainConsumers`, generic runtime launches, live provider activation | Not yet | Not yet |

`toolchains.provide` may influence which program a defined tool launches, but it does not authorize replacing arbitrary commands, installing software, network access or running workspace code. Any plugin may define a short ID or `<own-plugin-id>:<local-id>`, providers may answer short IDs, and the host supplies no proof of origin; what happens when several plugins define the same short ID is described in "Tool IDs and conflicts" above.

### Contract sections

| Section | Content |
| --- | --- |
| [Plugin-owned toolchains (Core foundation)](../../contract/v1/README.md#plugin-owned-toolchains-core-foundation) | The toolchain summary and current host status in the contract entry point |
| [Plugin-owned toolchains: Core foundation](../../contract/v1/toolchains/README.md) | The complete description of the implemented part |
| [Manifest admission and registry](../../contract/v1/toolchains/README.md#manifest-admission-and-registry) | Capability and permission requirements, definition fields, provider fields, ID rules (short and qualified IDs) |
| [Two installation routes](../../contract/v1/toolchains/README.md#two-installation-routes) | Declarative discovery and provider queries |
| [Commands and envelopes](../../contract/v1/toolchains/README.md#commands-and-envelopes) | The nine Core commands, result envelopes, registry limits, `conflicts` |
| [Launching a tool definition](../../contract/v1/toolchains/README.md#launching-a-tool-definition) | `toolchains.instantiateLaunch`: expanding launch argv from the definition's `launch` template, `launchMismatch` |
| [Tool references (`serverProcess` 4)](../../contract/v1/README.md#tool-references-serverprocess-4) | Manifest `requiresTools`, where an ID is used, the parts of a launch plan, `plugin.host.toolUnresolved` |
| [Run and debug sessions (protocol version 3)](../../contract/v1/README.md#run-and-debug-sessions-protocol-version-3) | `process.executable` of `run/start`: `{kind: "tool"}` or `{kind: "packaged"}` |
| [Provider reply validation](../../contract/v1/toolchains/README.md#provider-reply-validation) | `decodeReply`, reply shape and limits, `failureReply` |
| [Pins and snapshot resolution](../../contract/v1/toolchains/README.md#pins-and-snapshot-resolution) | `local.json`, `resolve`, the four layers, `checkVersion`, `discoveryPlan` |
| [Settings candidates](../../contract/v1/toolchains/README.md#settings-candidates) | `candidates` and the two-call probe flow |
| [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | The `executableResolution` 1 and 2 candidate sets |
| [Provider backend negotiation](../../contract/v1/toolchains/README.md#provider-backend-negotiation) | `litheToolchains` negotiation (`experimental.litheToolchains` in `initialize`), `lsp.toolchainRequest`, timeouts |
| [Error replies](../../contract/v1/toolchains/README.md#error-replies) | The error reply format and the numeric codes of the 11 stable failure codes |

### Examples and cases

| File | Content |
| --- | --- |
| [`toolchains/tool-definition.plugin.json`](../../contract/v1/fixtures/toolchains/tool-definition.plugin.json) | A production-shape tool definition: `java` with 8 candidates (project and global settings, project and global toolchain requests, `JAVA_HOME` on macOS/Linux and on Windows, the `java` command, installed toolchains), discovering `*.jdk` in the macOS `JavaVirtualMachines` directories |
| [`toolchains/toolchain-provider.plugin.json`](../../contract/v1/fixtures/toolchains/toolchain-provider.plugin.json) | A production-shape provider: `mise` for `java`, `node`, `maven`, `pnpm` and `go`, watching 6 files (`.tool-versions` with the compatible reader), with a packaged executable backend |
| [`run/tool-reference-launch.plugin.json`](../../contract/v1/fixtures/run/tool-reference-launch.plugin.json) | Level 3, `serverProcess` 4: top-level `requiresTools` lists the plugin's own `interpreter`, and the language server's `runtime` launch references the runtime by that tool-definition ID |

Core's acceptance tests are [`toolchain_definitions.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/tests/toolchain_definitions.rs) and its siblings `toolchain_pins.rs`, `toolchain_queries.rs` and `toolchain_resolution.rs` in the main repository. The pin document, query exchange and one error reply per stable failure code that those three share are inline in [`toolchain_vectors.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/tests/toolchain_vectors.rs).

### Related guides

- [Build, run, and test for language plugins](../../guides/build-tool-integration.md): `toolchain` executables in build candidate sets
- [Native build, test, and run preparation](../../guides/native-build-workflows.md): `executable.kind: "toolchain"` in run preparation
