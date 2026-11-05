# 工具链 / Toolchains

[中文](#中文) · [English](#english)

## 中文

插件可以告诉 Lithe 一个工具（JDK、Maven、Node、Go 等）长什么样、去哪里找、怎么判断版本，也可以用后端回答"这台机器装了哪些版本、这个项目要哪个版本"。宿主没有内置任何工具定义、别名表或按工具名的分支：工具定义只来自已启用的插件，Core 负责校验和选择，宿主只做文件系统观察、运行版本探针和启动后端这些本机 I/O。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `toolDefinitions[]`（工具定义）、`toolchainProviders[]`（工具链提供者）；`executables` 候选集里的 `toolchain` 候选 |
| 能力版本 | 工具定义：`toolDefinitions` 1 + `executableResolution` 2；提供者：`toolchainProviders` 1 + `backend` 2 + `serverProcess` 1；都是级别 3。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
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

### Core 命令

所有命令都只处理 UTF-8 JSON，输入上限 4 MiB，不做任何本机 I/O。成功返回 `{status: "ok", <键>: 值}`，拒绝返回 `{status: "rejected", error: {code, message, field?}}`。

| 命令 | 输入 | 成功时的键 | 用途 |
| --- | --- | --- | --- |
| `toolchains.registry` | `{plugins: [{manifest, verifiedOfficial}]}` | `registry` | 重新校验完整清单，汇总定义和提供者；最多 128 个插件、512 个定义、256 个提供者 |
| `toolchains.decodeReply` | 查询请求、提供者应答、宿主观察到的文件系统事实 | `snapshot` | 校验一次 `toolchains/query` 应答，得到 `{installations, statements, watch, diagnostics}` |
| `toolchains.failureReply` | `{params, code}` | `reply` | 查询没有应答时生成对应的 JSON-RPC 错误；`code` 只能是 `plugin.toolchain.timeout` 或 `plugin.toolchain.providerUnavailable` |
| `toolchains.resolvePins` | `{document, directory, tool}` | `resolution` | 在 `local.json` 里找最近的固定，返回 `{pin, sourceDirectory}` |
| `toolchains.resolve` | 某个目录的完整解析输入 | `resolution` | 走四层选择，结果是 `resolved`、`skip` 或 `terminalFailure`，附带 `trace`、`unmet`、`conflicts` |
| `toolchains.checkVersion` | `{registry, tool, output, minimumVersion?}` | `version` | 用定义的版本正则取版本并比较；最多 8 段数字、64 字节，不解释别名 |
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

### 宿主现状

| 用途 | macOS | Windows |
| --- | --- | --- |
| 设置页显示发现结果 | 两条来源都跑，只读；监视的文件变化时重新加载整个快照；不保存任何东西 | 不支持 |
| 构建工具、插件运行方案的可执行文件 | 候选或运行方案指向插件定义的工具（不在宿主旧的工具链表里）时，单独重新解析一次；设置页的发现结果不会被当作启动许可 | 不支持 |
| Java、语言服务器 | 仍然走原来的适配器 | 不支持 |
| 保存固定、`toolchainConsumers`、通用运行时启动、提供者热激活 | 尚未提供 | 尚未提供 |

`toolchains.provide` 可以影响某个已定义工具启动哪个程序，但不授权替换任意命令、安装软件、访问网络或执行工作区代码。短定义 ID（例如 `java`）要求经过独立验证的官方来源；社区插件的定义 ID 写成 `<自己的插件 ID>:<本地 ID>`，社区提供者可以回答短 ID。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Plugin-owned toolchains (Core foundation)](../../contract/v1/README.md#plugin-owned-toolchains-core-foundation) | 契约总入口里的工具链说明和当前宿主状态 |
| [Plugin-owned toolchains: Core foundation](../../contract/v1/toolchains/README.md) | 已实现部分的完整说明 |
| [Manifest admission and registry](../../contract/v1/toolchains/README.md#manifest-admission-and-registry) | 能力和权限要求、定义字段、提供者字段、ID 规则 |
| [Two installation routes](../../contract/v1/toolchains/README.md#two-installation-routes) | 声明式发现和提供者查询 |
| [Commands and envelopes](../../contract/v1/toolchains/README.md#commands-and-envelopes) | 八个 Core 命令、返回格式、注册表限制 |
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
| Capability versions | Definitions: `toolDefinitions` 1 + `executableResolution` 2; providers: `toolchainProviders` 1 + `backend` 2 + `serverProcess` 1; all at level 3. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
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

### Core commands

Every command takes UTF-8 JSON with a 4 MiB input budget and performs no native I/O. Success is `{status: "ok", <key>: value}`; refusal is `{status: "rejected", error: {code, message, field?}}`.

| Command | Input | Success key | Purpose |
| --- | --- | --- | --- |
| `toolchains.registry` | `{plugins: [{manifest, verifiedOfficial}]}` | `registry` | Revalidates complete manifests and collects definitions and providers; at most 128 plugins, 512 definitions and 256 providers |
| `toolchains.decodeReply` | The query request, the provider reply and host-observed filesystem facts | `snapshot` | Validates one `toolchains/query` reply into `{installations, statements, watch, diagnostics}` |
| `toolchains.failureReply` | `{params, code}` | `reply` | Produces the JSON-RPC error for a query that got no reply; `code` is only `plugin.toolchain.timeout` or `plugin.toolchain.providerUnavailable` |
| `toolchains.resolvePins` | `{document, directory, tool}` | `resolution` | Finds the nearest pin in `local.json` and returns `{pin, sourceDirectory}` |
| `toolchains.resolve` | The complete resolver input for one directory | `resolution` | Runs the four layers; the outcome is `resolved`, `skip` or `terminalFailure`, with `trace`, `unmet` and `conflicts` |
| `toolchains.checkVersion` | `{registry, tool, output, minimumVersion?}` | `version` | Extracts the version with the definition's regex and compares it; at most 8 numeric components and 64 bytes, no aliases interpreted |
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

### Host status

| Use | macOS | Windows |
| --- | --- | --- |
| Settings pages showing detections | Both routes run read-only; a change to a watched file reloads the whole snapshot; nothing is saved | No |
| Executables for Build tools and plugin run plans | A candidate or run plan that names a plugin-defined tool (not one in the host's legacy toolchain table) is resolved afresh; settings detections are never launch permits | No |
| Java and language servers | Still use their previous adapters | No |
| Saving pins, `toolchainConsumers`, generic runtime launches, live provider activation | Not yet | Not yet |

`toolchains.provide` may influence which program a defined tool launches, but it does not authorize replacing arbitrary commands, installing software, network access or running workspace code. Short definition IDs (such as `java`) need independently verified official origin; a community plugin writes definition IDs as `<own-plugin-id>:<local-id>`, and community providers may answer short IDs.

### Contract sections

| Section | Content |
| --- | --- |
| [Plugin-owned toolchains (Core foundation)](../../contract/v1/README.md#plugin-owned-toolchains-core-foundation) | The toolchain summary and current host status in the contract entry point |
| [Plugin-owned toolchains: Core foundation](../../contract/v1/toolchains/README.md) | The complete description of the implemented part |
| [Manifest admission and registry](../../contract/v1/toolchains/README.md#manifest-admission-and-registry) | Capability and permission requirements, definition fields, provider fields, ID rules |
| [Two installation routes](../../contract/v1/toolchains/README.md#two-installation-routes) | Declarative discovery and provider queries |
| [Commands and envelopes](../../contract/v1/toolchains/README.md#commands-and-envelopes) | The eight Core commands, result envelopes, registry limits |
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

Core's acceptance tests are [`toolchain_definitions.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/tests/toolchain_definitions.rs) and its siblings `toolchain_pins.rs`, `toolchain_queries.rs` and `toolchain_resolution.rs` in the main repository. The pin document, query exchange and one error reply per stable failure code that those three share are inline in [`toolchain_vectors.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/tests/toolchain_vectors.rs).

### Related guides

- [Build, run, and test for language plugins](../../guides/build-tool-integration.md): `toolchain` executables in build candidate sets
- [Native build, test, and run preparation](../../guides/native-build-workflows.md): `executable.kind: "toolchain"` in run preparation
