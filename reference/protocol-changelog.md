# 插件协议更新记录

本页记录插件协议和 API 的每次修改：哪天、哪个 PR、谁改的、改了什么。只有中文。

- **收录范围**：Lithe 主仓库 `preview` 分支上的 PR，只要它修改了下面三处中的任意一处：插件契约 `shared/contracts/plugins/v1/`、Core 能力表 [`capabilities.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/plugins/capabilities.rs)、插件函数协议 [`plugin_function.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/lsp/interface/plugin_function.rs)。三处都没改、只改宿主内部实现的 PR 不收录。
- **日期**：PR 合入 `preview` 的日期。
- **修改人**：原 PR 的作者（GitHub 账号）。登记这些 PR 的同步 PR 会在描述里 @ 他们，请他们核对自己那一行。
- **记录到**：`preview` 的 [`5d38352a1`](https://github.com/1lck/Lithe-IDEA/commit/5d38352a1757a70fd5fa297d7aa3e43bb2e33607)（[#1752](https://github.com/1lck/Lithe-IDEA/pull/1752)），即本仓库 `contract/v1/` 当前副本的来源。
- **每一版的内容和宿主现状**：见[能力与协议版本对照](capabilities-and-versions.md)。字段和行为以 [`contract/v1/README.md`](../contract/v1/README.md) 为准，官方 SDK 的发版见 [SDK 发版历史](capabilities-and-versions.md#sdk-发版历史)。

时间线的“协议 / API”一列里，**加粗**的是这个 PR 新增的版本号；没有加粗的，是同一版本内的补充、宿主接入或说明修正。

## 按协议查首次出现

### 插件 API 级别与文档格式

| 名称 | 版本 | 日期 | PR |
| --- | --- | --- | --- |
| 插件 API 级别 `pluginAPIVersion` | 1 | 2026-10-14 | [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) |
| 插件 API 级别 `pluginAPIVersion` | 2 | 2026-10-14 | [#1214](https://github.com/1lck/Lithe-IDEA/pull/1214)；工作台 UI 由 [#1206](https://github.com/1lck/Lithe-IDEA/pull/1206) 加入 |
| 插件 API 级别 `pluginAPIVersion` | 3（`requires`） | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| 插件 API 级别 `pluginAPIVersion` | 4（调试适配器） | 2026-10-26 | [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) |
| 插件清单 `schemaVersion` | 2 | 2026-10-14 | [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) |
| 市场索引 `schemaVersion` | 1 | 2026-10-14 | [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) |
| 原生 UI 文档 `schemaVersion` | 1 | 2026-10-24 | [#1423](https://github.com/1lck/Lithe-IDEA/pull/1423) |
| `litheBuildModel`、`litheDependencies`、`litheTestResults` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `litheTestDiscovery` | 1 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `litheBuildConfiguration` | 1 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| 本机工具链文档 `.lithe/toolchains/local.json` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |

### 能力版本（`requires`）

`requires` 由 #1318 引入，首批 8 个能力都从版本 1 开始。其中 `languages`、`languageServers`、`serverProcess`、`executionProviders`、`settings`、`ui` 的内容在此之前已经随 API 级别 1、2 提供，#1318 只是给它们定了能力版本号。

| 能力 | 版本 | 日期 | PR |
| --- | --- | --- | --- |
| `languages` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `languages` | 2 | 2026-10-31 | [#1617](https://github.com/1lck/Lithe-IDEA/pull/1617) |
| `languageServers` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `languageServers` | 2 | 2026-10-21 | [#1395](https://github.com/1lck/Lithe-IDEA/pull/1395) |
| `serverProcess` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `serverProcess` | 2 | 2026-10-26 | [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) |
| `executionProviders` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `settings` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `settings` | 2 | 2026-10-21 | [#1395](https://github.com/1lck/Lithe-IDEA/pull/1395) |
| `ui` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `ui` | 2 | 2026-10-18 | [#1334](https://github.com/1lck/Lithe-IDEA/pull/1334) |
| `ui` | 3 | 2026-10-24 | [#1423](https://github.com/1lck/Lithe-IDEA/pull/1423) |
| `ui` | 4 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| `buildSystems` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `buildSystems` | 2 | 2026-10-23 | [#1420](https://github.com/1lck/Lithe-IDEA/pull/1420) |
| `buildSystems` | 3、4 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| `buildProviders` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `buildProviders` | 2–6 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `buildProviders` | 7–11 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| `buildModelInputs` | 1 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `executableResolution` | 1 | 2026-10-23 | [#1420](https://github.com/1lck/Lithe-IDEA/pull/1420) |
| `executableResolution` | 2 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |
| `vcsProviders` | 1 | 2026-10-23 | [#1430](https://github.com/1lck/Lithe-IDEA/pull/1430) |
| `projectGenerators` | 1 | 2026-10-24 | [#1452](https://github.com/1lck/Lithe-IDEA/pull/1452) |
| `projectGenerators` | 2 | 2026-11-05 | [#1740](https://github.com/1lck/Lithe-IDEA/pull/1740) |
| `backend` | 1 | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| `backend` | 2 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| `debugAdapters` | 1 | 2026-10-26 | [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) |
| `debugAdapters` | 2 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `packageTrust` | 1 | 2026-10-28 | [#1483](https://github.com/1lck/Lithe-IDEA/pull/1483) |
| `toolDefinitions` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |
| `toolchainProviders` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |
| `keymapImportProviders` | 1 | 2026-11-03 | [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) |

### 进程协议版本

| 协议 | 版本 | 日期 | PR |
| --- | --- | --- | --- |
| `litheFunctions` | 1 | 2026-10-20 | [#1358](https://github.com/1lck/Lithe-IDEA/pull/1358) |
| `litheFunctions` | 2（宿主服务） | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| `litheFunctions` | 3（工作区上下文） | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| `litheBuild` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) 定义；macOS 宿主由 [#1356](https://github.com/1lck/Lithe-IDEA/pull/1356) 接入 |
| `litheBuild` | 2–6 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `litheBuild` | 7–11 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| `litheVcs` | 1 | 2026-10-23 | [#1430](https://github.com/1lck/Lithe-IDEA/pull/1430) |
| `litheToolchains` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) 实现；正式契约由 [#1752](https://github.com/1lck/Lithe-IDEA/pull/1752) 写入 |
| `litheKeymapImport` | 1 | 2026-11-03 | [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) |
| `litheLogging` | 1 | 2026-11-04 | [#1699](https://github.com/1lck/Lithe-IDEA/pull/1699) |

## 时间线（新的在上）

### 2026-11-05

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1752](https://github.com/1lck/Lithe-IDEA/pull/1752) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `litheToolchains` 1：写入正式契约 | 删除工具链设计草案 `toolchains/proposal/` 和 `fixtures/toolchains/proposal/`。草案里已实现的部分写进正式契约 `toolchains/README.md`：新增“Provider backend negotiation”一节（`initialize` 里的 `experimental.litheToolchains` 协商、`lsp.toolchainRequest`、超时）和“Error replies”小节（错误回复格式与 11 个稳定失败码的数字码对照）。Core 测试改用内联数据 `toolchain_vectors.rs`。协议行为没有变化。 |
| [#1740](https://github.com/1lck/Lithe-IDEA/pull/1740) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`projectGenerators` 2**（API-33） | 生成器新增 `tool` 字段：从插件声明的工具定义的已有安装里选一个（`tool`、`minimumVersion`、`pinAs`、`optional`），模板可用 `${field.<id>.executable}`；`pinAs: "project"` 时计划返回 `toolPins`，宿主把它存为新项目的工具固定。表单输入加 `tools`，结果字段带 `installations`，新增问题码 `toolTooOld`；目录条目带 `tools`（声明该工具的插件）。版本 2 不能用 `runtime` 字段，版本 1 不能用 `tool` 字段。macOS 声明 1、2，Windows 不声明。新增示例 `project-generator-tool-field.plugin.json`、12 个计划用例和 9 个清单拒绝用例。 |
| [#1747](https://github.com/1lck/Lithe-IDEA/pull/1747) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 清单 schema：`requires.buildSystems` | schema 的枚举补齐到 1–4（Core 从 #1558 起已经支持 3、4，schema 当时只写到 3）。fixtures 按功能分目录，工具链提案移到 `toolchains/proposal/`，契约 README 的能力版本表随之修正。 |

### 2026-11-04

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1724](https://github.com/1lck/Lithe-IDEA/pull/1724) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 包签名 | 新增 `fixtures/signature-vectors.json`，是发布者签名的跨平台测试向量：RFC 8032 测试密钥、带非 ASCII 文件名的固定包和它的规范化载荷。同一 PR 的语言服务器启动失败清理、归档解压加固属于宿主行为，没有改字段。 |
| [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) | [@Mucheen](https://github.com/Mucheen) | **`buildProviders` 7–11**、**`litheBuild` 7–11**、**`buildSystems` 3、4**、**`litheBuildConfiguration` 1** | 7 基于真实来源的原生配置，8 内联测试报告，9 不透明语义测试选择器，10 类型化运行选项与服务/任务运行准备，11 注册表连接调试。`buildSystems` 3 是实例级选项，4 是声明式项目准备角色。 |
| [#1720](https://github.com/1lck/Lithe-IDEA/pull/1720) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `litheLogging` 1：声明式插件 | 声明式贡献没有自己的进程，发不了 `lithe/log`，改由执行方在 `plugins/<id>/host` 通道代记 `plugins.operation.finished` 和 `plugins.operation.failed`。每次启动语言服务器或调试适配器、每次运行构建模型或依赖命令，各记为一次操作。失败时带一个错误码，例如 `host.planFailed`、`host.executableUnavailable`、`host.launchFailed`。 |
| [#1699](https://github.com/1lck/Lithe-IDEA/pull/1699) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`litheLogging` 1** | 插件拥有的每个进程都可以协商 `capabilities.experimental.litheLogging`，然后用通知 `lithe/log` 发结构化诊断。参数只接受固定字段，未知字段会被拒绝。新增 `fixtures/plugin-logging.json`。 |

### 2026-11-03

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) | [@1lck](https://github.com/1lck) | **`keymapImportProviders` 1**、**`litheKeymapImport` 1**、函数协议 3 的宿主方法 | 快捷键导入提供者，需要 `backend` 2 和对应权限。后端在 `initialize` 中选出要启用的提供者，宿主向它发 `lithe/keymap/import`。单独授权后，后端可以调用 `lithe/host/keymapImport/listInputs` 和 `readChunk`。 |
| [#1676](https://github.com/1lck/Lithe-IDEA/pull/1676) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 启动模板 `${logDir}` | 新增模板变量 `${logDir}`，是当前会话 `plugins/<id>/files/` 的绝对路径，由宿主显式传入 `logDirectory`。语言服务器和插件后端可用，`validationArguments` 不可用。`logDirectory` 缺失、为空或不是绝对路径时返回 `plugin.launch.invalidRequest`。 |

### 2026-11-02

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1661](https://github.com/1lck/Lithe-IDEA/pull/1661) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `toolDefinitions` 1、`toolchainProviders` 1：宿主现状 | macOS 开始声明这两个能力，用于只读的工具链设置页。Windows 不声明。保存固定版本、`toolchainConsumers` 仍未实现。 |
| [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 函数协议 3：插件原生终端 | 新增权限 `terminal.create`（需要 `backend` 2），授予宿主方法 `lithe/host/terminal/create`、`show`、`close`、`reopen`。后端创建和管理自己的原生终端，reopen 时保留有界的显示历史。 |
| [#1650](https://github.com/1lck/Lithe-IDEA/pull/1650) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 函数协议说明 | 契约改为说明函数协议 1–3，并按实际情况描述官方 SDK 实现的协议。不改字段。 |
| [#1646](https://github.com/1lck/Lithe-IDEA/pull/1646) | [@FxRayHughes](https://github.com/FxRayHughes) | 清单 schema：`settings` 2、`languageServers` 2 | schema 补上 Core 从 #1395 起已经支持的这两个版本。 |

### 2026-11-01

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1643](https://github.com/1lck/Lithe-IDEA/pull/1643) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 契约文档链接 | 工具链提案的链接和 schema `$id` 检查改为符合 lithe-plugin-docs 的同步规则。不改字段。 |
| [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) | [@FxRayHughes](https://github.com/FxRayHughes) | **`toolDefinitions` 1**、**`toolchainProviders` 1**、**`executableResolution` 2**、**`litheToolchains` 1**（提案） | 插件注入的工具链框架。工具定义描述工具的身份、目录布局、发现方式和候选，需要 `executableResolution` 2。工具链提供者需要 `backend` 2，通过 `litheToolchains` 协议回答 `toolchains/query`。同时新增 `proposals/toolchains/` 提案和本机工具链文档 `.lithe/toolchains/local.json` 版本 1。 |

### 2026-10-31

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1617](https://github.com/1lck/Lithe-IDEA/pull/1617) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`languages` 2**、函数协议 3：编辑器动作 | `languages[].grammar` 是包内的 Monarch 语法，`languages[].configuration` 是编辑器配置，用到其中任一项都要声明 `languages` 2。后端用 `lithe/host/editor/registerActions` 注册不依赖语言服务器的编辑器动作，用户触发时宿主按普通插件函数调用。 |

### 2026-10-30

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`backend` 2**、**`ui` 4**、**`litheFunctions` 3** | `backend` 2：后端可以在 `onWorkspaceOpen` 或插件命令时启动，通过函数协议接收工作区上下文，并可读取 `workspace.metadata`。`ui` 4：不打开工具窗口的后端命令。函数协议 3：工作区上下文与能力发现（`lithe/host/getCapabilities`、`lithe/host/workspace/*`）、宿主通知订阅，以及公共的文档、文件、窗口方法。 |

### 2026-10-28

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) | [@Mucheen](https://github.com/Mucheen) | **`buildProviders` 2–6**、**`litheBuild` 2–6**、**`buildModelInputs` 1**、**`debugAdapters` 2**、**`litheTestDiscovery` 1** | 2 任务执行（`lithe/build/execute`），3 测试执行和测试发现（`lithe/build/test`、`lithe/build/discoverTests`），4 应用运行准备（`lithe/build/prepareRun`），5 原生调试准备（`lithe/build/prepareDebug`），6 只读模型输入。`buildModelInputs` 1 用于在已启用的工作区插件之间共享公开模型。`debugAdapters` 2 支持绑定调试目标的参数和环境变量。 |
| [#1483](https://github.com/1lck/Lithe-IDEA/pull/1483) | [@Mucheen](https://github.com/Mucheen) | **`packageTrust` 1** | 新增 `vendor.bundledSignatureRequirement`：随宿主发布、由发布者签名的插件副本按来源信任。Java Support 从此单独分发。 |

### 2026-10-27

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1525](https://github.com/1lck/Lithe-IDEA/pull/1525) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `debugAdapters` 1：远程 attach | `attach` 配置的 `targetBindings` 可以带 `connection`（远端主机和端口），`debug.attach` 权限也覆盖远程 attach。适配器仍在本机运行，只有被调试进程在远端。 |
| [#1520](https://github.com/1lck/Lithe-IDEA/pull/1520) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `debugAdapters` 1：`tcpLanguageServer` 传输 | 新增传输方式 `tcpLanguageServer`：调试适配器由插件自己的语言服务器承载，例如 JDT LS 里的 Java Debug Server。`endpoint` 用 `languageID` 指定语言服务器，用 `command` 指定启动命令。 |
| [#1512](https://github.com/1lck/Lithe-IDEA/pull/1512) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `debugAdapters` 1：macOS 宿主 | macOS 开始运行调试适配器，插件可热激活。新增 `configurations.launch.executable`：调试目标是适配器不能直接调试的源文件（C、C++、Swift）时，从插件的文件设置或最多 8 个候选路径找可执行文件。新增 `console.commandPrefix` 和 `console.commands[]`，供调试控制台使用。 |

### 2026-10-26

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **插件 API 级别 4**、**`debugAdapters` 1**、**`serverProcess` 2** | 新增 `debugAdapters[]`，用来声明 DAP 适配器的启动、传输、配置和权限，需要 API 级别 4。`serverProcess` 2 新增 `runSelectedRuntime`，只用于调试适配器。 |

### 2026-10-24

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) | [@1lck](https://github.com/1lck) | **`backend` 1**、**`litheFunctions` 2** | 插件后端：插件自己的长驻进程，为自己的网页提供服务。函数协议 2 加入宿主服务，即凭据 `lithe/host/credentials/get`、`set`、`delete`；宿主把插件权限允许的服务列表传给后端。 |
| [#1424](https://github.com/1lck/Lithe-IDEA/pull/1424) | [@Mucheen](https://github.com/Mucheen) | `ui` 2：Windows 宿主 | Windows 声明式插件宿主上线，用 WebView2 实现包内网页并声明 `ui` 1、2，不声明 `ui` 3。 |
| [#1423](https://github.com/1lck/Lithe-IDEA/pull/1423) | [@arkleselect](https://github.com/arkleselect) | **`ui` 3**、**原生 UI 文档 schema 1** | 宿主渲染的原生控件，新增 `native-ui.schema.json`。 |
| [#1452](https://github.com/1lck/Lithe-IDEA/pull/1452) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`projectGenerators` 1** | 新建项目窗口里的项目生成器，可以用模板文件、脚手架命令或归档下载生成项目，需要 `project.generate` 权限。 |

### 2026-10-23

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1430](https://github.com/1lck/Lithe-IDEA/pull/1430) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`vcsProviders` 1**、**`litheVcs` 1** | VCS 提供者。协议在 `initialize` 的 `experimental.litheVcs` 中协商，定义见主仓库的 `shared/contracts/vcs-provider-v1.md`。 |
| [#1442](https://github.com/1lck/Lithe-IDEA/pull/1442) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 契约发布 | 契约开始发布到 lithe-plugin-docs，fixtures 按能力改名，SDK 移出主仓库。不改字段。 |
| [#1420](https://github.com/1lck/Lithe-IDEA/pull/1420) | [@Mucheen](https://github.com/Mucheen) | **`buildSystems` 2**、**`executableResolution` 1** | 构建系统可以把可执行文件路径交给插件设置来配置。共享的可执行文件候选：统一的候选排序和严格选择。 |

### 2026-10-21

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1395](https://github.com/1lck/Lithe-IDEA/pull/1395) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`languageServers` 2**、**`settings` 2** | `workspaceConfiguration` 用于向语言服务器推送工作区配置。设置 2 新增放置位置（`page`、`section`）、作用域、生效方式，`multiSelect`、`text`、`file`、`directory` 类型和页面按钮；插件可以注册设置页。 |
| [#1382](https://github.com/1lck/Lithe-IDEA/pull/1382) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `litheBuild` 1 | 构建提供者的请求可以带 `refresh`（用户要求重新加载）和 `tool.executable`（宿主解析出的构建工具绝对路径）。两者都是可选字段，协议仍是版本 1。 |

### 2026-10-20

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1379](https://github.com/1lck/Lithe-IDEA/pull/1379) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 官方 Java SDK | Java 后端 SDK 首发，当时从主仓库发布。契约补充：探测 `java` 运行时版本用 `-version`，Java 8 也能识别。 |
| [#1366](https://github.com/1lck/Lithe-IDEA/pull/1366) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 网页调用插件函数 | macOS 把每个构建提供者进程作为所属插件的会话启动。插件网页的桥接新增 `functions.list`、`functions.invoke`、`functions.cancel`、`functions.progress`。 |
| [#1365](https://github.com/1lck/Lithe-IDEA/pull/1365) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 官方 Node SDK | Node 后端 SDK 首发，实现函数协议和 `litheBuild` 1。 |
| [#1358](https://github.com/1lck/Lithe-IDEA/pull/1358) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`litheFunctions` 1** | 插件函数协议：注册、撤回、调用、结果、进度、取消，在 `initialize` 的 `capabilities.experimental.litheFunctions` 中协商，由 Core 负责路由。 |
| [#1356](https://github.com/1lck/Lithe-IDEA/pull/1356) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `litheBuild` 1：macOS 宿主 | macOS 开始启动构建提供者并发送 `lithe/build/*` 请求。每个提供者是独立进程，工作区关闭、插件停用或插件包被替换前会停止。 |

### 2026-10-19

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1345](https://github.com/1lck/Lithe-IDEA/pull/1345) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 社区插件安装 | 社区插件除了从第三方市场安装，还可以从本地 ZIP 或解包目录安装：宿主先取私有快照，再请用户确认安装审查。包里有签名文档时仍然必须通过验证。 |
| [#1339](https://github.com/1lck/Lithe-IDEA/pull/1339) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `ui` 2：宿主现状 | macOS 用 WKWebView 实现包内网页并声明 `ui` 1、2。Windows 当时只声明 `ui` 1，`ui` 2 插件返回 `plugin.api.unsupported`，直到 #1424。 |

### 2026-10-18

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1334](https://github.com/1lck/Lithe-IDEA/pull/1334) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`ui` 2** | 工具窗口中的包内网页。 |
| [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **插件 API 级别 3**（`requires`）、**`buildSystems` 1**、**`buildProviders` 1**、**`litheBuild` 1**、**`litheBuildModel` / `litheDependencies` / `litheTestResults` 1** | 从级别 3 起，插件要按能力声明版本，首批 8 个能力都是版本 1（见上方能力表）；市场索引里级别 3 的版本必须带 `requires`。新增声明式构建工具插件，以及构建提供者协议 `litheBuild` 1（`lithe/build/model`、`lithe/build/dependencies`）。 |

### 2026-10-15

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1206](https://github.com/1lck/Lithe-IDEA/pull/1206) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 插件 API 级别 2：`ui` 与 `ui.contribute` | 级别 2 加入第一阶段工作台 UI（宿主渲染的工具窗口和命令）和 `ui.contribute` 权限。`languages` 不再必填，插件可以只提供 UI。 |

### 2026-10-14

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1214](https://github.com/1lck/Lithe-IDEA/pull/1214) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **插件 API 级别 2** | 新增启动模板（`arguments` 可用变量）、`runtimeDependencies` 的 `java`、`initializationOptions`、`hostIntegration`（仅官方插件）、包内或已发现运行时启动（`runtime`、`packagedRuntime`）和插件设置。Java Support 改为声明式官方插件。 |
| [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) | [@1lck](https://github.com/1lck) | **契约 v1**：**插件 API 级别 1**、**清单 `schemaVersion` 2**、**市场索引 `schemaVersion` 1** | 契约 v1 进入 `preview`：语言、语言服务器（`packaged` 和 `systemCommand` 启动）、`node` 和 `go` 运行时依赖、运行和测试提供者，以及市场索引和标准用例。 |

## 维护

每次在本仓库同步新的契约版本（`node scripts/contract.mjs sync --revision <commit>`）时，都要在同一个 PR 里更新本页：

1. 在 Lithe 主仓库列出两次同步之间改动契约的 PR：

   ```sh
   git log origin/preview --first-parent --format='%h %cd %s' --date=short <上次记录到的 commit>..<本次 commit> -- shared/contracts/plugins/v1 rust/lithe-core/src/plugins/capabilities.rs rust/lithe-core/src/lsp/interface/plugin_function.rs
   ```

2. 按日期把这些 PR 补到时间线最上面。新增的版本号要加粗，并写进“按协议查首次出现”的表。
3. “修改人”填原 PR 作者的 GitHub 账号，写成 `[@账号](https://github.com/账号)`。账号可以这样查：

   ```sh
   gh pr view <PR 编号> --repo 1lck/Lithe-IDEA --json author --jq .author.login
   ```

4. 把开头的“记录到”改成本次同步的 commit。
5. 同时更新[能力与协议版本对照](capabilities-and-versions.md)和受影响的功能页。
6. 在同步 PR 的描述里列出本次登记的 PR，并直接写 `@账号` 提到每位修改人，请他们核对自己那一行。本页里的 `[@账号](…)` 只是链接，不会通知任何人。
