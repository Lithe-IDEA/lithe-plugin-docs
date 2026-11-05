# 能力与协议版本对照 / Capability and protocol versions

[中文](#中文) · [English](#english)

## 中文

本页把插件会碰到的所有版本号放在一起：每个能力、每个进程协议、每个文档格式的**全部版本**，以及 Rust Core、macOS、Windows、三个官方 SDK 各自支持到哪一版。规范以 [`contract/v1/README.md`](../contract/v1/README.md) 为准；本页与契约不一致时以契约为准，并请提 Issue。本页以主仓库最新 `preview` 为基准：Core、macOS、Windows 列按 `preview` 上的代码核对（2026-11-05，`644afdaac`），契约链接对应 [`contract.lock.json`](../contract.lock.json) 锁定的修订。Lithe 稳定版可能还没有其中的部分版本，发布插件前以目标版本的发行说明为准。

### 四类版本号

| 版本号 | 写在哪里 | 谁检查 | 不满足时 |
| --- | --- | --- | --- |
| 插件 API 级别 `pluginAPIVersion` | `plugin.json` | Core | 安装审查拒绝 |
| 能力版本 `requires.<能力>` | `plugin.json`（级别 3 起必填） | Core 的支持表，再和宿主的 `hostCapabilities` 取交集 | `plugin.api.unsupported`，`field` 为 `requires.<能力>`；市场不提供这个插件版本 |
| 进程协议版本 `experimental.lithe*` | 不写进清单；进程启动时在 LSP `initialize` 里协商 | 宿主给出版本列表，后端从中选一个 | 该协议不启用，例如不向这个进程发构建请求 |
| 文档格式版本 | 各文档自己的 `schemaVersion` 或 `version` 字段 | Core | 解析失败 |

能力版本决定**能不能安装**；协议版本决定**进程启动后能说什么**。两者分开演进：后端支持一个新的协议版本，不需要提高清单里的任何版本号。

发布规则（契约 [API levels](../contract/v1/README.md#api-levels) 一节）：

- 级别和版本只增不改，已发布的版本不改变含义，宿主一直支持旧版本。
- 插件只声明自己用到的**最低**级别和每个能力的**最低**版本。没用到新内容就提高版本，只会让旧宿主装不上。
- 提高级别或能力版本必须发新的插件版本；市场索引保留上一个旧要求的版本，旧宿主仍有版本可装。
- 支持某个级别或版本的宿主先发布，声明它的插件后发布。

### 插件 API 级别

| 级别 | 新增 | Core | macOS | Windows |
| --- | --- | --- | --- | --- |
| `1` | 语言、`packaged` 和 `systemCommand` 语言服务器、`node` 和 `go` 运行时依赖、运行和测试提供者 | ✅ | ✅ | ✅ |
| `2` | 用包内或已发现运行时启动（`runtime`）、宿主集成、`java` 运行时、启动模板、`initializationOptions`、插件设置、第一阶段工作台 UI（`ui`） | ✅ | ✅ | 由 Core 校验¹ |
| `3` | 按能力声明版本（`requires`）、`buildSystems`、`vcsProviders` | ✅ | ✅ | 按下表的 Windows 列准入 |
| `4` | `debugAdapters`、`serverProcess` 2（`runSelectedRuntime`） | ✅ | ✅ | 按下表的 Windows 列准入 |

- Core：`SUPPORTED_PLUGIN_API_VERSIONS = [1, 2, 3, 4]`（[`declarative.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/plugins/declarative.rs)）。macOS：`PluginManifest.supportedDeclarativeAPIVersions = [1, 2, 3, 4]`。
- 级别 1、2 不看宿主能力表；级别 3 起，安装和市场筛选使用宿主能力表。
- ¹ Windows 通过 Core 校验级别 2 的清单。按契约 [Current Windows availability](../contract/v1/README.md#current-windows-availability)，Windows 声明 `ui` 1、2（工具窗口、命令、包内网页），不声明 3、4；级别 3 起的插件按宿主能力表准入。市场的 `plugins.marketplace.validateIndex` 目前只在 `darwin-*` 平台把级别 2 的版本标为可用，同一版本在 Windows 上显示 `plugin.index.apiUnsupported`。

### 能力版本（`requires`）

每一行是一个能力的一个版本。Core 列是 Core 能校验的版本（[`capabilities.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/plugins/capabilities.rs) 的 `SUPPORTED_CAPABILITIES`），macOS 和 Windows 列是宿主真正能运行并在安装时声明的版本（[macOS `PluginManifest.hostCapabilities`](https://github.com/1lck/Lithe-IDEA/blob/preview/macos/Sources/LitheModuleAPI/Plugins/PluginTypes.swift)、[Windows `host_capabilities()`](https://github.com/1lck/Lithe-IDEA/blob/preview/windows/tauri/src-tauri/src/plugin_host/mod.rs)）。Core 能校验不等于宿主能运行。

| 能力 | 版本 | 这一版新增 | Core | macOS | Windows | 功能页 |
| --- | --- | --- | --- | --- | --- | --- |
| `languages` | 1 | `languages[]`：语言 ID、文件扩展名等 | ✅ | ✅ | ✅ | [语言](domains/languages.md) |
| `languages` | 2 | [`grammar` 与 `configuration`](../contract/v1/README.md#grammars-and-editor-configuration-languages-2)：包内 Monarch 语法和编辑器配置 | ✅ | ✅ | — | [语言](domains/languages.md) |
| `languageServers` | 1 | `languageServers[]`（启动字段之外的部分） | ✅ | ✅ | ✅ | [语言](domains/languages.md) |
| `languageServers` | 2 | [`workspaceConfiguration`](../contract/v1/README.md#workspace-configuration-languageservers-2)：向语言服务器推送工作区配置 | ✅ | ✅ | ✅ | [语言](domains/languages.md) |
| `serverProcess` | 1 | 进程启动字段：`launch`、`arguments`、`validationArguments`、`runtimeDependencies`、`initializationOptions`；语言服务器、自有进程的构建提供者、VCS 提供者、插件后端共用 | ✅ | ✅ | ✅ | [语言](domains/languages.md) |
| `serverProcess` | 2 | `runSelectedRuntime`，供[调试适配器](../contract/v1/README.md#api-level-4-debug-adapters)使用；需要级别 4 | ✅ | — | — | [调试适配器](domains/debug-adapters.md) |
| `executionProviders` | 1 | `executionProviders[]`：运行和测试提供者 | ✅ | ✅ | — | [语言](domains/languages.md) |
| `settings` | 1 | `settings[]` 与 `settingActions[]` | ✅ | ✅ | ✅ | [设置](domains/settings.md) |
| `settings` | 2 | [放置位置、作用域、生效方式、页面按钮](../contract/v1/README.md#settings-v2-placement-scope-effect-and-page-buttons) | ✅ | ✅ | ✅ | [设置](domains/settings.md) |
| `ui` | 1 | [第一阶段工作台 UI](../contract/v1/README.md#api-level-2-first-stage-host-owned-workbench-ui)：宿主渲染的工具窗口和命令 | ✅ | ✅ | ✅ | [界面](domains/ui.md) |
| `ui` | 2 | [包内网页](../contract/v1/README.md#web-content-ui-2) | ✅ | ✅ | ✅ | [界面](domains/ui.md) |
| `ui` | 3 | [宿主渲染的原生控件](../contract/v1/README.md#host-rendered-native-content-ui-3-native-schema-1)（原生 UI 文档 schema 1） | ✅ | ✅ | — | [界面](domains/ui.md) |
| `ui` | 4 | [不打开工具窗口的后端命令](../contract/v1/README.md#backend-only-commands-ui-4)，需要 `backend` 2 | ✅ | ✅ | — | [插件后端](domains/backend.md) |
| `buildSystems` | 1 | [声明式构建](../contract/v1/README.md#api-level-3-build-systems)：检测、选项、项目模型、任务、运行入口、测试、诊断、依赖、同步 | ✅ | ✅ | — | [声明式构建](domains/build-systems.md) |
| `buildSystems` | 2 | 可执行文件设置适配 | ✅ | ✅ | — | [声明式构建](domains/build-systems.md) |
| `buildSystems` | 3 | 实例级选项 | ✅ | ✅ | — | [声明式构建](domains/build-systems.md) |
| `buildSystems` | 4 | [声明式项目准备角色](../contract/v1/README.md#declarative-project-preparation-buildsystems-4) | ✅ | ✅ | — | [声明式构建](domains/build-systems.md) |
| `buildProviders` | 1 | [构建提供者](../contract/v1/README.md#build-providers)：后端回答模型和依赖请求，可以是自有进程或共享语言服务器 | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 2 | [任务执行](../contract/v1/README.md#provider-task-execution-buildproviders-2) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 3 | [测试执行](../contract/v1/README.md#provider-test-execution-buildproviders-3)与[测试发现](../contract/v1/README.md#test-discovery) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 4 | [应用运行准备](../contract/v1/README.md#provider-run-preparation-buildproviders-4) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 5 | [原生调试准备](../contract/v1/README.md#native-debug-preparation-version-5) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 6 | [只读模型输入](../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 7 | [基于真实来源的原生配置](../contract/v1/README.md#optional-native-configuration-buildproviders-7) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 8 | [内联测试报告](../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 9 | [不透明语义测试选择器](../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 10 | [类型化运行选项与服务/任务运行准备](../contract/v1/README.md#typed-run-options-and-servicetask-preparation-version-10) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildProviders` | 11 | [注册表连接调试](../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `buildModelInputs` | 1 | [已启用工作区插件之间共享公开模型](../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1)（可选） | ✅ | ✅ | — | [构建提供者](domains/build-providers.md) |
| `vcsProviders` | 1 | [`vcsProviders[]`](../contract/v1/README.md#api-level-3-vcs-providers) | ✅ | ✅ | — | [VCS](domains/vcs.md) |
| `debugAdapters` | 1 | [`debugAdapters[]`](../contract/v1/README.md#api-level-4-debug-adapters)；需要级别 4 | ✅ | ✅ | — | [调试适配器](domains/debug-adapters.md) |
| `debugAdapters` | 2 | 调试目标的参数和环境变量 | ✅ | ✅ | — | [调试适配器](domains/debug-adapters.md) |
| `projectGenerators` | 1 | [`projectGenerators[]`](../contract/v1/README.md#project-generators)：新建项目窗口里的条目 | ✅ | ✅ | — | [项目生成器](domains/project-generators.md) |
| `backend` | 1 | [插件后端](../contract/v1/README.md#plugin-backend)：插件自己的长驻进程，从插件页面启动 | ✅ | ✅ | — | [插件后端](domains/backend.md) |
| `backend` | 2 | [无页面启动和工作区上下文](../contract/v1/README.md#backend-2-workspace-contexts) | ✅ | ✅ | — | [插件后端](domains/backend.md) |
| `packageTrust` | 1 | `vendor.bundledSignatureRequirement`：随宿主发布的发布者签名插件副本按来源信任 | ✅ | ✅ | — | [包信任](domains/package-trust.md) |
| `toolDefinitions` | 1 | [工具身份、布局、发现和四层候选](../contract/v1/toolchains/README.md) | ✅ | ✅（只读设置、构建与运行工具） | — | [工具链](domains/toolchains.md) |
| `toolchainProviders` | 1 | [工具链提供者声明和按权限校验应答](../contract/v1/toolchains/README.md) | ✅ | ✅（只读设置、构建与运行工具） | — | [工具链](domains/toolchains.md) |
| `executableResolution` | 1 | [共享可执行文件候选](../contract/v1/README.md#shared-executable-candidates-executableresolution-12)：候选排序和严格选择 | ✅ | ✅ | — | [工具链](domains/toolchains.md) |
| `executableResolution` | 2 | 工具定义候选、项目/子项目工具链注册表绑定；不改变已有构建候选集 | ✅ | ✅（只读设置、构建与运行工具） | — | [工具链](domains/toolchains.md) |
| `keymapImportProviders` | 1 | [快捷键导入提供者](../contract/v1/README.md#shortcut-import-providers-core-foundation) | ✅ | ✅ | — | [快捷键导入](domains/keymap-import.md) |

`requires` 必须**恰好**列出用到的能力：漏一个或多列一个都是 `plugin.manifest.invalid`。每个语言服务器、每个自有进程的构建提供者、每个 VCS 提供者和 `backend` 都要带上 `serverProcess`。

### 进程协议版本

这些协议在进程启动时协商，不写进清单。宿主在 `initialize` 的 `capabilities.experimental.<协议>` 里给出 `{ "versions": [...] }`，后端回答 `{ "version": N }`。回答了宿主没给出的版本，或格式不对，该协议整个不启用。

| 协议 | 版本 | 这一版新增 | 宿主向谁提供 | 契约章节 |
| --- | --- | --- | --- | --- |
| `litheFunctions` | 1 | 插件函数：注册、撤回、调用、结果、进度、取消；不提供宿主服务，所有 `lithe/host/*` 请求返回 `plugin.host.unavailable` | 插件拥有的会话 | [Plugin functions](../contract/v1/README.md#plugin-functions) |
| `litheFunctions` | 2 | 宿主服务：凭据 `credentials/get`、`set`、`delete` | 同上 | [Host services](../contract/v1/README.md#host-services-protocol-version-2) |
| `litheFunctions` | 3 | 工作区上下文与能力发现、宿主通知、公共文档/文件/窗口方法、独立编辑器动作、插件自有原生终端 | 同上 | [Protocol 3](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) |
| `litheBuild` | 1 | `lithe/build/model` 与 `lithe/build/dependencies` | 构建提供者 | [Build providers](../contract/v1/README.md#build-providers) |
| `litheBuild` | 2 | 类型化任务执行（`lithe/build/execute`） | 同上 | [Task execution](../contract/v1/README.md#provider-task-execution-buildproviders-2) |
| `litheBuild` | 3 | 类型化测试执行和发现（`lithe/build/test`、`lithe/build/discoverTests`） | 同上 | [Test execution](../contract/v1/README.md#provider-test-execution-buildproviders-3) |
| `litheBuild` | 4 | 应用运行准备（`lithe/build/prepareRun`） | 同上 | [Run preparation](../contract/v1/README.md#provider-run-preparation-buildproviders-4) |
| `litheBuild` | 5 | 原生调试准备（`lithe/build/prepareDebug`） | 同上 | [Debug preparation](../contract/v1/README.md#native-debug-preparation-version-5) |
| `litheBuild` | 6 | 接收宿主显式声明的只读模型输入 | 同上 | [Model inputs](../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) |
| `litheBuild` | 7 | 基于真实来源的原生配置 | 同上 | [Native configuration](../contract/v1/README.md#optional-native-configuration-buildproviders-7) |
| `litheBuild` | 8 | 内联测试报告 | 同上 | [Inline test reports](../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) |
| `litheBuild` | 9 | 不透明语义测试选择器 | 同上 | [Test selectors](../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) |
| `litheBuild` | 10 | 类型化运行选项与服务/任务运行准备 | 同上 | [Run options](../contract/v1/README.md#typed-run-options-and-servicetask-preparation-version-10) |
| `litheBuild` | 11 | 注册表连接调试 | 同上 | [Connection debugging](../contract/v1/README.md#registry-connection-debugging-buildproviders-11) |
| `litheVcs` | 1 | VCS 提供者的能力和请求 | VCS 提供者 | [VCS providers](../contract/v1/README.md#api-level-3-vcs-providers) |
| `litheKeymapImport` | 1 | 后端从声明的提供者中选出要启用的子集；`lithe/keymap/import` | 声明了 `keymapImportProviders` 的插件会话 | [Shortcut import](../contract/v1/README.md#shortcut-import-providers-core-foundation) |
| `litheToolchains` | 1 | `toolchains/query`；后端从声明的提供者中选出子集 | 工具链提供者（macOS；Windows 未接入） | [Toolchains protocol](../contract/v1/toolchains/proposal/protocol.md) |
| `litheLogging` | 1 | 结构化诊断通知 `lithe/log` | 每个插件拥有的进程 | [Structured diagnostics](../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) |

- `litheBuild` 各版本新增的内容与同号的 `buildProviders` 能力版本一致。
- `litheFunctions` 3 内部还有两个子版本号：宿主服务版本 `HOST_SERVICES_VERSION = 2`、上下文版本 `CONTEXT_VERSION = 3`，SDK 用它们判断能调用哪些宿主方法；运行时以 `lithe/host/getCapabilities` 的返回为准。
- 宿主侧：上表的协议都由 Rust Core 实现；macOS 运行全部会话类型。Windows 没有插件后端宿主，也不运行构建提供者和 VCS 提供者。

### 宿主与官方 SDK 对照

“Core 提供”是宿主在 `initialize` 里给出的版本列表；SDK 列是该 SDK 能回答的版本。后端会选两边都支持的最高版本。

| 协议 | Core 提供 | Node SDK 0.3.0 | Java SDK 0.3.0 | Rust SDK 0.2.0 |
| --- | --- | --- | --- | --- |
| `litheFunctions` | 1、2、3 | 3 | 3 | 3、2、1 |
| `litheBuild` | 1–11 | 1–6 | 1–6 | — |
| `litheVcs` | 1 | 1 | — | — |
| `litheKeymapImport` | 1 | 1 | 1 | 1 |
| `litheToolchains` | 1 | — | — | — |
| `litheLogging` | 1 | 1（必需） | 1（必需） | 1（必需） |

- Node 和 Java SDK 的构建方法按协商结果启用：`model`、`dependencies` 需要 1，`execute` 需要 2，`test`、`discoverTests` 需要 3，`prepareRun` 需要 4，`prepareDebug` 需要 5。版本 7–11 的内容目前没有 SDK 封装。
- 当前 SDK 要求宿主提供 `litheLogging` 1，否则拒绝初始化；用这些 SDK 构建的后端只能在提供该协议的 Lithe 版本中启动。

### SDK 发版历史

| 日期 | 版本 | `litheFunctions` | `litheBuild` | `litheVcs` | `litheKeymapImport` | `litheLogging` | 主要变化 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2026-10-20 | [Java 0.1.0](https://github.com/1lck/Lithe-IDEA/releases/tag/sdk-java-v0.1.0) | 1 | 1 | — | — | — | 首个版本，从主仓库发布 |
| 2026-10-23 | [Node 0.1.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-node-v0.1.0) | 1 | 1 | — | — | — | 首个版本 |
| 2026-11-02 | [Node 0.2.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-node-v0.2.0) | 3 | 1–6 | 1 | — | — | 函数协议 2、3；构建协议 2–6 的类型化处理器；VCS 提供者 |
| 2026-11-02 | [Java 0.2.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-java-v0.2.0) | 3 | 1–6 | — | — | — | 函数协议 2、3；构建协议 2–6 的类型化处理器 |
| 2026-11-02 | [Rust 0.1.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-rust-v0.1.0) | 3、2、1 | — | — | — | — | 首个版本 |
| 2026-11-05 | [Node 0.3.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-node-v0.3.0) | 3 | 1–6 | 1 | 1 | 1（必需） | 快捷键导入提供者、原生终端、类型化诊断；移除自由文本 `log` |
| 2026-11-05 | [Java 0.3.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-java-v0.3.0) | 3 | 1–6 | — | 1 | 1（必需） | 同 Node 0.3.0 |
| 2026-11-05 | [Rust 0.2.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-rust-v0.2.0) | 3、2、1 | — | — | 1 | 1（必需） | 同 Node 0.3.0 |

### 文档格式版本

| 文档 | 版本字段 | 当前版本 | 定义 |
| --- | --- | --- | --- |
| 插件清单 `plugin.json` | `schemaVersion` | 2 | [`plugin-manifest.schema.json`](../contract/v1/plugin-manifest.schema.json) |
| 市场索引 | `schemaVersion` | 1 | [`plugin-index.schema.json`](../contract/v1/plugin-index.schema.json) |
| 原生 UI 文档（`ui` 3） | `schemaVersion` | 1 | [`native-ui.schema.json`](../contract/v1/native-ui.schema.json) |
| 构建模型 | `format: "litheBuildModel"` | 1 | [Model and outputs](../contract/v1/README.md#model-and-outputs) |
| 依赖树 | `format: "litheDependencies"` | 1 | [Model and outputs](../contract/v1/README.md#model-and-outputs) |
| 测试结果 | `format: "litheTestResults"` | 1 | [Model and outputs](../contract/v1/README.md#model-and-outputs) |
| 测试发现 | `format: "litheTestDiscovery"` | 1 | [Test discovery](../contract/v1/README.md#test-discovery) |
| 原生构建配置 | `format: "litheBuildConfiguration"` | 1 | [Native configuration](../contract/v1/README.md#optional-native-configuration-buildproviders-7) |
| 本机工具链文档 `.lithe/toolchains/local.json` | `version` | 1 | [Pins and snapshot resolution](../contract/v1/toolchains/README.md#pins-and-snapshot-resolution)；读取不会创建文件，保存固定尚未实现 |

---

## English

This page puts every version number a plugin meets in one place: **every version** of each capability, each process protocol, and each document format, and how far Rust Core, macOS, Windows, and the three official SDKs support each. [`contract/v1/README.md`](../contract/v1/README.md) is normative; where this page disagrees with it, the contract wins, and please open an issue. This page tracks the latest `preview` of the Lithe repository: the Core, macOS, and Windows columns are checked against the code on `preview` (2026-11-05, `644afdaac`), and contract links follow the revision pinned in [`contract.lock.json`](../contract.lock.json). A stable Lithe release may lack some of these versions, so check your target version's release notes before publishing.

### Four kinds of version

| Version | Where it is written | Who checks it | When it is not met |
| --- | --- | --- | --- |
| Plugin API level `pluginAPIVersion` | `plugin.json` | Core | The install review refuses the package |
| Capability version `requires.<capability>` | `plugin.json` (required from level 3) | Core's support table, intersected with the host's `hostCapabilities` | `plugin.api.unsupported` with `field: "requires.<capability>"`; the marketplace does not offer that plugin version |
| Process protocol version `experimental.lithe*` | Not in the manifest; negotiated in LSP `initialize` when the process starts | The host offers a list, the backend picks one | The protocol is not enabled, for example no build request goes to that process |
| Document format version | The document's own `schemaVersion` or `version` field | Core | Decoding fails |

A capability version decides **whether a plugin installs**; a protocol version decides **what a running process can say**. They evolve separately: a backend that supports a new protocol version does not raise any version in its manifest.

Publishing rules (contract section [API levels](../contract/v1/README.md#api-levels)):

- Levels and versions only add. A released version never changes meaning, and hosts keep supporting older versions.
- A plugin declares the **lowest** level, and for each capability the **lowest** version, it needs. Raising one without using what it adds only cuts off older hosts.
- Raising a level or capability version needs a new plugin version; the index keeps the last version with the previous requirements so older hosts still have one to install.
- A host release that supports a level or version ships before any plugin that declares it.

### Plugin API levels

| Level | Adds | Core | macOS | Windows |
| --- | --- | --- | --- | --- |
| `1` | Languages, `packaged` and `systemCommand` language servers, `node` and `go` runtime dependencies, run and test providers | ✅ | ✅ | ✅ |
| `2` | `runtime` launches with a packaged or discovered runtime, host integrations, the `java` runtime, launch templates, `initializationOptions`, plugin settings, first-stage workbench UI (`ui`) | ✅ | ✅ | Validated by Core¹ |
| `3` | Per-capability versions (`requires`), `buildSystems`, `vcsProviders` | ✅ | ✅ | Admitted by the Windows column below |
| `4` | `debugAdapters`, `serverProcess` 2 (`runSelectedRuntime`) | ✅ | ✅ | Admitted by the Windows column below |

- Core: `SUPPORTED_PLUGIN_API_VERSIONS = [1, 2, 3, 4]` ([`declarative.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/plugins/declarative.rs)). macOS: `PluginManifest.supportedDeclarativeAPIVersions = [1, 2, 3, 4]`.
- Levels 1 and 2 ignore the host capability table; from level 3, installation and marketplace filtering use it.
- ¹ Windows validates level 2 manifests through Core. Per the contract's [Current Windows availability](../contract/v1/README.md#current-windows-availability), Windows advertises `ui` 1 and 2 (tool windows, commands, package web content) and not 3 or 4; plugins at level 3 and above are admitted by the host capability table. The marketplace's `plugins.marketplace.validateIndex` currently marks level 2 versions available only on `darwin-*` platforms; on Windows the same version shows `plugin.index.apiUnsupported`.

### Capability versions (`requires`)

Each row is one version of one capability. The Core column is what Core can validate (`SUPPORTED_CAPABILITIES` in [`capabilities.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/plugins/capabilities.rs)); the macOS and Windows columns are what each host actually runs and advertises at installation ([macOS `PluginManifest.hostCapabilities`](https://github.com/1lck/Lithe-IDEA/blob/preview/macos/Sources/LitheModuleAPI/Plugins/PluginTypes.swift), [Windows `host_capabilities()`](https://github.com/1lck/Lithe-IDEA/blob/preview/windows/tauri/src-tauri/src/plugin_host/mod.rs)). Validating a capability is not running it.

| Capability | Version | Adds | Core | macOS | Windows | Domain page |
| --- | --- | --- | --- | --- | --- | --- |
| `languages` | 1 | `languages[]`: language IDs, file extensions, and so on | ✅ | ✅ | ✅ | [Languages](domains/languages.md) |
| `languages` | 2 | [`grammar` and `configuration`](../contract/v1/README.md#grammars-and-editor-configuration-languages-2): packaged Monarch grammars and editor configuration | ✅ | ✅ | — | [Languages](domains/languages.md) |
| `languageServers` | 1 | `languageServers[]` apart from their launch fields | ✅ | ✅ | ✅ | [Languages](domains/languages.md) |
| `languageServers` | 2 | [`workspaceConfiguration`](../contract/v1/README.md#workspace-configuration-languageservers-2): workspace configuration pushed to the server | ✅ | ✅ | ✅ | [Languages](domains/languages.md) |
| `serverProcess` | 1 | Launch fields `launch`, `arguments`, `validationArguments`, `runtimeDependencies`, `initializationOptions`, shared by language servers, own-process build providers, VCS providers, and the plugin backend | ✅ | ✅ | ✅ | [Languages](domains/languages.md) |
| `serverProcess` | 2 | `runSelectedRuntime` for [debug adapters](../contract/v1/README.md#api-level-4-debug-adapters); needs level 4 | ✅ | — | — | [Debug adapters](domains/debug-adapters.md) |
| `executionProviders` | 1 | `executionProviders[]`: run and test providers | ✅ | ✅ | — | [Languages](domains/languages.md) |
| `settings` | 1 | `settings[]` and `settingActions[]` | ✅ | ✅ | ✅ | [Settings](domains/settings.md) |
| `settings` | 2 | [Placement, scope, effect, page buttons](../contract/v1/README.md#settings-v2-placement-scope-effect-and-page-buttons) | ✅ | ✅ | ✅ | [Settings](domains/settings.md) |
| `ui` | 1 | [First-stage workbench UI](../contract/v1/README.md#api-level-2-first-stage-host-owned-workbench-ui): host-rendered tool windows and commands | ✅ | ✅ | ✅ | [UI](domains/ui.md) |
| `ui` | 2 | [Package web content](../contract/v1/README.md#web-content-ui-2) | ✅ | ✅ | ✅ | [UI](domains/ui.md) |
| `ui` | 3 | [Host-rendered native controls](../contract/v1/README.md#host-rendered-native-content-ui-3-native-schema-1) (native UI schema 1) | ✅ | ✅ | — | [UI](domains/ui.md) |
| `ui` | 4 | [Backend-only commands](../contract/v1/README.md#backend-only-commands-ui-4) without a tool window; needs `backend` 2 | ✅ | ✅ | — | [Plugin backend](domains/backend.md) |
| `buildSystems` | 1 | [Declarative builds](../contract/v1/README.md#api-level-3-build-systems): detection, options, project model, tasks, run entries, tests, diagnostics, dependencies, sync | ✅ | ✅ | — | [Build systems](domains/build-systems.md) |
| `buildSystems` | 2 | Executable setting adapter | ✅ | ✅ | — | [Build systems](domains/build-systems.md) |
| `buildSystems` | 3 | Instance-scoped options | ✅ | ✅ | — | [Build systems](domains/build-systems.md) |
| `buildSystems` | 4 | [Declarative project preparation roles](../contract/v1/README.md#declarative-project-preparation-buildsystems-4) | ✅ | ✅ | — | [Build systems](domains/build-systems.md) |
| `buildProviders` | 1 | [Build providers](../contract/v1/README.md#build-providers): a backend answers model and dependency requests, in its own process or a shared language server | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 2 | [Task execution](../contract/v1/README.md#provider-task-execution-buildproviders-2) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 3 | [Test execution](../contract/v1/README.md#provider-test-execution-buildproviders-3) and [test discovery](../contract/v1/README.md#test-discovery) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 4 | [Application run preparation](../contract/v1/README.md#provider-run-preparation-buildproviders-4) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 5 | [Native debug preparation](../contract/v1/README.md#native-debug-preparation-version-5) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 6 | [Read-only model inputs](../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 7 | [Source-backed native configuration](../contract/v1/README.md#optional-native-configuration-buildproviders-7) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 8 | [Inline test reports](../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 9 | [Opaque semantic test selectors](../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 10 | [Typed run options and service/task run preparation](../contract/v1/README.md#typed-run-options-and-servicetask-preparation-version-10) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildProviders` | 11 | [Registry connection debugging](../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `buildModelInputs` | 1 | [Optional public model sharing between enabled workspace plugins](../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | ✅ | ✅ | — | [Build providers](domains/build-providers.md) |
| `vcsProviders` | 1 | [`vcsProviders[]`](../contract/v1/README.md#api-level-3-vcs-providers) | ✅ | ✅ | — | [VCS](domains/vcs.md) |
| `debugAdapters` | 1 | [`debugAdapters[]`](../contract/v1/README.md#api-level-4-debug-adapters); needs level 4 | ✅ | ✅ | — | [Debug adapters](domains/debug-adapters.md) |
| `debugAdapters` | 2 | Debug target arguments and environment | ✅ | ✅ | — | [Debug adapters](domains/debug-adapters.md) |
| `projectGenerators` | 1 | [`projectGenerators[]`](../contract/v1/README.md#project-generators): entries of the New Project window | ✅ | ✅ | — | [Project generators](domains/project-generators.md) |
| `backend` | 1 | [Plugin backend](../contract/v1/README.md#plugin-backend): the plugin's own long-lived process, started from its pages | ✅ | ✅ | — | [Plugin backend](domains/backend.md) |
| `backend` | 2 | [Start without a page, and workspace contexts](../contract/v1/README.md#backend-2-workspace-contexts) | ✅ | ✅ | — | [Plugin backend](domains/backend.md) |
| `packageTrust` | 1 | `vendor.bundledSignatureRequirement`: origin-scoped trust for a host-sealed bundled copy of a publisher-signed plugin | ✅ | ✅ | — | [Package trust](domains/package-trust.md) |
| `toolDefinitions` | 1 | [Tool identities, layouts, discovery, and four-layer candidates](../contract/v1/toolchains/README.md) | ✅ | ✅ (read-only settings, Build and run tools) | — | [Toolchains](domains/toolchains.md) |
| `toolchainProviders` | 1 | [Provider declarations and permission-bound answer validation](../contract/v1/toolchains/README.md) | ✅ | ✅ (read-only settings, Build and run tools) | — | [Toolchains](domains/toolchains.md) |
| `executableResolution` | 1 | [Shared executable candidates](../contract/v1/README.md#shared-executable-candidates-executableresolution-12): candidate ordering and strict selection | ✅ | ✅ | — | [Toolchains](domains/toolchains.md) |
| `executableResolution` | 2 | Tool-definition candidates and project/subproject toolchain registry bindings, without changing existing build candidate sets | ✅ | ✅ (read-only settings, Build and run tools) | — | [Toolchains](domains/toolchains.md) |
| `keymapImportProviders` | 1 | [Shortcut import providers](../contract/v1/README.md#shortcut-import-providers-core-foundation) | ✅ | ✅ | — | [Shortcut import](domains/keymap-import.md) |

`requires` lists **exactly** the capabilities a manifest uses: leaving one out or listing an unused one is `plugin.manifest.invalid`. Every language server, own-process build provider, VCS provider, and `backend` also needs `serverProcess`.

### Process protocol versions

These protocols are negotiated when a process starts and never appear in the manifest. The host offers `{ "versions": [...] }` under `capabilities.experimental.<protocol>` in `initialize`; the backend answers `{ "version": N }`. An answer with a version the host did not offer, or a malformed answer, leaves the whole protocol disabled.

| Protocol | Version | Adds | Offered to | Contract section |
| --- | --- | --- | --- | --- |
| `litheFunctions` | 1 | Plugin functions: register, withdraw, invoke, results, progress, cancellation; no host services, every `lithe/host/*` request is `plugin.host.unavailable` | Plugin-owned sessions | [Plugin functions](../contract/v1/README.md#plugin-functions) |
| `litheFunctions` | 2 | Host services: credentials `credentials/get`, `set`, `delete` | Same | [Host services](../contract/v1/README.md#host-services-protocol-version-2) |
| `litheFunctions` | 3 | Workspace contexts and capability discovery, host notifications, public document/file/window methods, independent editor actions, plugin-owned native terminals | Same | [Protocol 3](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) |
| `litheBuild` | 1 | `lithe/build/model` and `lithe/build/dependencies` | Build providers | [Build providers](../contract/v1/README.md#build-providers) |
| `litheBuild` | 2 | Typed task execution (`lithe/build/execute`) | Same | [Task execution](../contract/v1/README.md#provider-task-execution-buildproviders-2) |
| `litheBuild` | 3 | Typed test execution and discovery (`lithe/build/test`, `lithe/build/discoverTests`) | Same | [Test execution](../contract/v1/README.md#provider-test-execution-buildproviders-3) |
| `litheBuild` | 4 | Application run preparation (`lithe/build/prepareRun`) | Same | [Run preparation](../contract/v1/README.md#provider-run-preparation-buildproviders-4) |
| `litheBuild` | 5 | Native debug preparation (`lithe/build/prepareDebug`) | Same | [Debug preparation](../contract/v1/README.md#native-debug-preparation-version-5) |
| `litheBuild` | 6 | Accepts the host's explicitly declared read-only model inputs | Same | [Model inputs](../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) |
| `litheBuild` | 7 | Source-backed native configuration | Same | [Native configuration](../contract/v1/README.md#optional-native-configuration-buildproviders-7) |
| `litheBuild` | 8 | Inline test reports | Same | [Inline test reports](../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) |
| `litheBuild` | 9 | Opaque semantic test selectors | Same | [Test selectors](../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) |
| `litheBuild` | 10 | Typed run options and service/task run preparation | Same | [Run options](../contract/v1/README.md#typed-run-options-and-servicetask-preparation-version-10) |
| `litheBuild` | 11 | Registry connection debugging | Same | [Connection debugging](../contract/v1/README.md#registry-connection-debugging-buildproviders-11) |
| `litheVcs` | 1 | VCS provider capabilities and requests | VCS providers | [VCS providers](../contract/v1/README.md#api-level-3-vcs-providers) |
| `litheKeymapImport` | 1 | The backend selects a subset of its declared providers; `lithe/keymap/import` | Plugin sessions with declared `keymapImportProviders` | [Shortcut import](../contract/v1/README.md#shortcut-import-providers-core-foundation) |
| `litheToolchains` | 1 | `toolchains/query`; the backend selects a subset of its declared providers | Toolchain providers (macOS; Windows not connected) | [Toolchains protocol](../contract/v1/toolchains/proposal/protocol.md) |
| `litheLogging` | 1 | Structured diagnostic notification `lithe/log` | Every plugin-owned process | [Structured diagnostics](../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) |

- Each `litheBuild` version adds what the `buildProviders` capability version of the same number adds.
- `litheFunctions` 3 carries two sub-versions: host services `HOST_SERVICES_VERSION = 2` and contexts `CONTEXT_VERSION = 3`, which SDKs use to decide which host methods they may call; at runtime, `lithe/host/getCapabilities` is authoritative.
- Hosts: Rust Core implements every protocol above, and macOS runs every session kind. Windows has no plugin backend host and runs no build or VCS providers.

### Host and official SDKs

“Core offers” is the list the host sends in `initialize`; the SDK columns are the versions each SDK can answer. A backend picks the highest version both sides support.

| Protocol | Core offers | Node SDK 0.3.0 | Java SDK 0.3.0 | Rust SDK 0.2.0 |
| --- | --- | --- | --- | --- |
| `litheFunctions` | 1, 2, 3 | 3 | 3 | 3, 2, 1 |
| `litheBuild` | 1–11 | 1–6 | 1–6 | — |
| `litheVcs` | 1 | 1 | — | — |
| `litheKeymapImport` | 1 | 1 | 1 | 1 |
| `litheToolchains` | 1 | — | — | — |
| `litheLogging` | 1 | 1 (required) | 1 (required) | 1 (required) |

- The Node and Java SDKs enable each build method from the negotiated version: `model` and `dependencies` need 1, `execute` 2, `test` and `discoverTests` 3, `prepareRun` 4, `prepareDebug` 5. Versions 7–11 have no SDK wrappers yet.
- The current SDKs require the host to offer `litheLogging` 1 and refuse to initialize otherwise, so a backend built on them starts only in a Lithe release that offers it.

### SDK release history

| Date | Release | `litheFunctions` | `litheBuild` | `litheVcs` | `litheKeymapImport` | `litheLogging` | Main changes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2026-10-20 | [Java 0.1.0](https://github.com/1lck/Lithe-IDEA/releases/tag/sdk-java-v0.1.0) | 1 | 1 | — | — | — | First release, published from the Lithe repository |
| 2026-10-23 | [Node 0.1.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-node-v0.1.0) | 1 | 1 | — | — | — | First release |
| 2026-11-02 | [Node 0.2.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-node-v0.2.0) | 3 | 1–6 | 1 | — | — | Function protocols 2 and 3; typed handlers for build protocols 2–6; VCS providers |
| 2026-11-02 | [Java 0.2.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-java-v0.2.0) | 3 | 1–6 | — | — | — | Function protocols 2 and 3; typed handlers for build protocols 2–6 |
| 2026-11-02 | [Rust 0.1.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-rust-v0.1.0) | 3, 2, 1 | — | — | — | — | First release |
| 2026-11-05 | [Node 0.3.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-node-v0.3.0) | 3 | 1–6 | 1 | 1 | 1 (required) | Shortcut import providers, native terminals, typed diagnostics; free-text `log` removed |
| 2026-11-05 | [Java 0.3.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-java-v0.3.0) | 3 | 1–6 | — | 1 | 1 (required) | Same as Node 0.3.0 |
| 2026-11-05 | [Rust 0.2.0](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases/tag/sdk-rust-v0.2.0) | 3, 2, 1 | — | — | 1 | 1 (required) | Same as Node 0.3.0 |

### Document format versions

| Document | Version field | Current version | Definition |
| --- | --- | --- | --- |
| Plugin manifest `plugin.json` | `schemaVersion` | 2 | [`plugin-manifest.schema.json`](../contract/v1/plugin-manifest.schema.json) |
| Marketplace index | `schemaVersion` | 1 | [`plugin-index.schema.json`](../contract/v1/plugin-index.schema.json) |
| Native UI document (`ui` 3) | `schemaVersion` | 1 | [`native-ui.schema.json`](../contract/v1/native-ui.schema.json) |
| Build model | `format: "litheBuildModel"` | 1 | [Model and outputs](../contract/v1/README.md#model-and-outputs) |
| Dependency tree | `format: "litheDependencies"` | 1 | [Model and outputs](../contract/v1/README.md#model-and-outputs) |
| Test results | `format: "litheTestResults"` | 1 | [Model and outputs](../contract/v1/README.md#model-and-outputs) |
| Test discovery | `format: "litheTestDiscovery"` | 1 | [Test discovery](../contract/v1/README.md#test-discovery) |
| Native build configuration | `format: "litheBuildConfiguration"` | 1 | [Native configuration](../contract/v1/README.md#optional-native-configuration-buildproviders-7) |
| Machine-local toolchain document `.lithe/toolchains/local.json` | `version` | 1 | [Pins and snapshot resolution](../contract/v1/toolchains/README.md#pins-and-snapshot-resolution); reading never creates the file, and saving pins is not implemented |
