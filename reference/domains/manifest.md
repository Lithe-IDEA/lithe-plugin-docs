# 清单与打包 / Manifest and packaging

[中文](#中文) · [English](#english)

## 中文

每个插件包的根目录有一份 `plugin.json`。它说明插件是谁、按哪个插件 API 级别和能力版本编写、需要哪些权限、在什么时候启动。宿主遇到不认识的字段会拒绝整个包，而不是忽略其中一部分。其他功能页只讲各自的清单字段，本页讲所有插件共用的部分。

| 项目 | 内容 |
| --- | --- |
| 文件 | `plugin.json`（[`plugin-manifest.schema.json`](../../contract/v1/plugin-manifest.schema.json)）；发布者签名 `lithe-plugin-signature.json`（见[包信任](package-trust.md)） |
| 基础字段 | `schemaVersion`（固定为 `2`）、`id`、`displayName`、`version`、`pluginAPIVersion`、`requires`、`vendor`、`permissions`、`activationEvents` |
| 版本 | `pluginAPIVersion` 1–4，见[插件 API 级别](../capabilities-and-versions.md#插件-api-级别)。基础字段只随 `pluginAPIVersion` 变化；级别 3 起各功能字段按[能力版本](../capabilities-and-versions.md#能力版本requires)单独变化 |
| API 编号 | 不在 API-01–44 内，属于安装和打包层面的契约 |
| Windows | 级别 1 可用；级别 2 由 Core 校验；级别 3 起按 Windows 宿主能力表准入 |

### 清单字段归属

| 清单字段 | 最低级别 | `requires` 里的能力 | 功能页 |
| --- | --- | --- | --- |
| `languages` | 1 | `languages` | [语言](languages.md) |
| `languageServers` | 1 | `languageServers`、`serverProcess` | [语言](languages.md) |
| `executionProviders` | 1 | `executionProviders` | [语言](languages.md) |
| `settings`、`settingActions` | 2 | `settings` | [设置](settings.md) |
| `ui` | 2 | `ui` | [界面](ui.md) |
| `buildSystems` | 3 | `buildSystems`，可选 `executableResolution`、`buildModelInputs` | [声明式构建](build-systems.md) |
| `buildSystems[].provider` | 3 | `buildProviders`，自有进程时加 `serverProcess` | [构建提供者](build-providers.md) |
| `vcsProviders` | 3 | `vcsProviders`、`serverProcess` | [VCS](vcs.md) |
| `backend` | 3 | `backend`、`serverProcess` | [插件后端](backend.md) |
| `projectGenerators` | 3 | `projectGenerators` | [项目生成器](project-generators.md) |
| `toolDefinitions`、`toolchainProviders` | 3 | `toolDefinitions`、`toolchainProviders`、`executableResolution` | [工具链](toolchains.md) |
| `keymapImportProviders` | 3 | `keymapImportProviders`、`backend`、`serverProcess` | [快捷键导入](keymap-import.md) |
| `vendor.bundledSignatureRequirement` | 3 | `packageTrust` | [包信任](package-trust.md) |
| `debugAdapters` | 4 | `debugAdapters`，`runSelectedRuntime` 需要 `serverProcess` 2 | [调试适配器](debug-adapters.md) |

级别 1、2 的清单不写 `requires`；级别 3 起必须写，而且要**恰好**列出用到的能力。

### 权限与激活事件

| 权限 | 什么时候必须声明 | 功能页 |
| --- | --- | --- |
| `languageServer.launch` | 有 `languageServers` 或 `vcsProviders`，或构建提供者启动自己的进程 | [语言](languages.md)、[VCS](vcs.md)、[构建提供者](build-providers.md) |
| `process.run`、`process.test` | 有提供者声明了 `run`、`test` | [语言](languages.md) |
| `process.build` | `buildSystems` 不为空 | [声明式构建](build-systems.md) |
| `process.vcs` | `vcsProviders` 不为空 | [VCS](vcs.md) |
| `debug.launch`、`debug.attach` | 有 `debugAdapters`；有适配器在 `requests` 里列了 `attach` 时才加 `debug.attach` | [调试适配器](debug-adapters.md) |
| `project.generate` | `projectGenerators` 不为空 | [项目生成器](project-generators.md) |
| `ui.contribute`、`ui.web` | 声明了 `ui`；有工具窗口显示网页时加 `ui.web` | [界面](ui.md) |
| `backend.launch` | 声明了 `backend` | [插件后端](backend.md) |
| `keymaps.provide`、`keymaps.readSelectedSources` | `keymapImportProviders` 不为空 | [快捷键导入](keymap-import.md) |
| `toolchains.provide` | 声明了 `toolchainProviders` | [工具链](toolchains.md) |
| `credentials.store`、`workspace.*`、`documents.read`、`editor.*`、`preview.open`、`browser.open`、`window.messages`、`terminal.create` | 后端调用对应的宿主方法 | [API 总表](../api-catalog.md#已公开的宿主方法) |

| 激活事件 | 含义 |
| --- | --- |
| `onLanguage:<id>` | 项目里打开该语言的文档时启动语言服务器 |
| `onRun:<id>`、`onTest:<id>` | 用户运行该语言的文件、列出或运行测试时激活运行/测试提供者 |
| `onBuildSystem:<id>` | 构建系统第一次计划、建模或解码之前激活 |
| `onVcsRoot:<id>` | 工作区里检测到该 VCS 的根目录后启动 VCS 提供者 |
| `onDebug:<id>` | 调试会话需要该适配器时启动（级别 4） |
| `onView:<window-id>`、`onCommand:<command-id>` | 打开工具窗口、执行命令时激活界面模块；`ui` 4 的后端命令也用 `onCommand` 启动后端 |
| `onWorkspaceOpen` | 工作区打开后启动 `backend` 2 进程；唯一可选的事件 |

每个提供者都要有自己的事件，每个事件都要对应一个提供者。没有“每次启动都运行”的事件（`onStartupFinished`、`*`），声明了会被拒绝为 `plugin.activation.invalid`。启用插件只是允许激活，事件发生之前什么都不会启动。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Package layout](../../contract/v1/README.md#package-layout) | 包的目录结构；路径规则；不允许符号链接；安装后的包只读 |
| [Manifest](../../contract/v1/README.md#manifest) | 全部顶层字段；未知字段一律拒绝（`plugin.manifest.invalid`） |
| [API levels](../../contract/v1/README.md#api-levels) | 级别 1–4 各自新增的内容 |
| [Capability versions (level 3)](../../contract/v1/README.md#capability-versions-level-3) | `requires`、宿主能力表 `hostCapabilities`、跨级别和版本的发布规则 |
| [Permissions and activation](../../contract/v1/README.md#permissions-and-activation) | 权限表与激活事件表 |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | 结果与稳定错误码 |
| [Windows mapping](../../contract/v1/README.md#windows-mapping) | 旧版 Windows 扩展清单到本契约的字段映射 |
| [Existing community store and Lua template](../../contract/v1/README.md#existing-community-store-and-lua-template) | 把 Lua 社区模板从级别 1 升到级别 2 的步骤 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`manifest/hello-lithe.plugin.json`](../../contract/v1/fixtures/manifest/hello-lithe.plugin.json) | 最小的级别 2 界面插件：一个工具窗口、一个命令。ID 在保留的测试命名空间里，不能直接发布 |
| [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json) | 174 个拒绝用例，覆盖所有功能。每个用例给某个合法 fixture 打补丁，并写明稳定的拒绝码和字段 |

### 相关指南

- [从零构建一个完整生命周期的 LSP 插件](../../guides/lsp-plugin-lifecycle.md)：清单、打包、状态目录、安装、发布
- [插件接入统一日志](../../guides/plugin-logging.md)

---

## English

Every plugin package has a `plugin.json` at its root. It says who the plugin is, which plugin API level and capability versions it was written against, which permissions it needs, and when it starts. A host that meets an unknown field refuses the whole package instead of ignoring part of it. The other domain pages cover their own manifest fields; this page covers what every plugin shares.

| Item | Value |
| --- | --- |
| Files | `plugin.json` ([`plugin-manifest.schema.json`](../../contract/v1/plugin-manifest.schema.json)); publisher signature `lithe-plugin-signature.json` (see [Package trust](package-trust.md)) |
| Base fields | `schemaVersion` (always `2`), `id`, `displayName`, `version`, `pluginAPIVersion`, `requires`, `vendor`, `permissions`, `activationEvents` |
| Versions | `pluginAPIVersion` 1–4; see [Plugin API levels](../capabilities-and-versions.md#plugin-api-levels). Base fields change only with `pluginAPIVersion`; from level 3 on, each domain field changes with its own [capability version](../capabilities-and-versions.md#capability-versions-requires) |
| API IDs | Outside API-01–44; this is an installation and packaging contract |
| Windows | Level 1 works; level 2 is validated by Core; from level 3 on, admission follows the Windows host capability table |

### Which page owns each manifest field

| Manifest field | Lowest level | Capabilities in `requires` | Domain page |
| --- | --- | --- | --- |
| `languages` | 1 | `languages` | [Languages](languages.md) |
| `languageServers` | 1 | `languageServers`, `serverProcess` | [Languages](languages.md) |
| `executionProviders` | 1 | `executionProviders` | [Languages](languages.md) |
| `settings`, `settingActions` | 2 | `settings` | [Settings](settings.md) |
| `ui` | 2 | `ui` | [UI](ui.md) |
| `buildSystems` | 3 | `buildSystems`, optionally `executableResolution` and `buildModelInputs` | [Declarative build systems](build-systems.md) |
| `buildSystems[].provider` | 3 | `buildProviders`, plus `serverProcess` for an own process | [Build providers](build-providers.md) |
| `vcsProviders` | 3 | `vcsProviders`, `serverProcess` | [VCS](vcs.md) |
| `backend` | 3 | `backend`, `serverProcess` | [Plugin backend](backend.md) |
| `projectGenerators` | 3 | `projectGenerators` | [Project generators](project-generators.md) |
| `toolDefinitions`, `toolchainProviders` | 3 | `toolDefinitions`, `toolchainProviders`, `executableResolution` | [Toolchains](toolchains.md) |
| `keymapImportProviders` | 3 | `keymapImportProviders`, `backend`, `serverProcess` | [Shortcut import](keymap-import.md) |
| `vendor.bundledSignatureRequirement` | 3 | `packageTrust` | [Package trust](package-trust.md) |
| `debugAdapters` | 4 | `debugAdapters`; `runSelectedRuntime` needs `serverProcess` 2 | [Debug adapters](debug-adapters.md) |

Level 1 and 2 manifests have no `requires`. From level 3 on it is required and must list **exactly** the capabilities the manifest uses.

### Permissions and activation events

| Permission | Required when | Domain page |
| --- | --- | --- |
| `languageServer.launch` | `languageServers` or `vcsProviders` is not empty, or a build provider starts its own process | [Languages](languages.md), [VCS](vcs.md), [Build providers](build-providers.md) |
| `process.run`, `process.test` | a provider declares `run` or `test` | [Languages](languages.md) |
| `process.build` | `buildSystems` is not empty | [Declarative build systems](build-systems.md) |
| `process.vcs` | `vcsProviders` is not empty | [VCS](vcs.md) |
| `debug.launch`, `debug.attach` | `debugAdapters` is not empty; `debug.attach` only when an adapter lists `attach` in `requests` | [Debug adapters](debug-adapters.md) |
| `project.generate` | `projectGenerators` is not empty | [Project generators](project-generators.md) |
| `ui.contribute`, `ui.web` | `ui` is declared; `ui.web` when a tool window shows web content | [UI](ui.md) |
| `backend.launch` | `backend` is declared | [Plugin backend](backend.md) |
| `keymaps.provide`, `keymaps.readSelectedSources` | `keymapImportProviders` is not empty | [Shortcut import](keymap-import.md) |
| `toolchains.provide` | `toolchainProviders` is declared | [Toolchains](toolchains.md) |
| `credentials.store`, `workspace.*`, `documents.read`, `editor.*`, `preview.open`, `browser.open`, `window.messages`, `terminal.create` | the backend calls the matching host methods | [API catalog](../api-catalog.md#public-host-methods) |

| Activation event | Meaning |
| --- | --- |
| `onLanguage:<id>` | Start the language server when a document of the language opens in a project |
| `onRun:<id>`, `onTest:<id>` | Activate the run or test provider when the user runs a file, or lists or runs tests |
| `onBuildSystem:<id>` | Activate the build system before its first plan, model, or decode |
| `onVcsRoot:<id>` | Start the VCS provider once a root of its VCS is detected |
| `onDebug:<id>` | Start the debug adapter when a debug session needs it (level 4) |
| `onView:<window-id>`, `onCommand:<command-id>` | Activate the UI module when a tool window opens or a command runs; `ui` 4 backend commands also start the backend through `onCommand` |
| `onWorkspaceOpen` | Start a `backend` 2 process once a workspace is open; the only optional event |

Each provider needs its event and each event needs a provider. There is no event that starts a plugin on every launch (`onStartupFinished`, `*`); declaring one is `plugin.activation.invalid`. Enabling a plugin only permits activation; nothing starts until its event occurs.

### Contract sections

| Section | Content |
| --- | --- |
| [Package layout](../../contract/v1/README.md#package-layout) | Package structure, path rules, no symbolic links, installed packages are read-only |
| [Manifest](../../contract/v1/README.md#manifest) | Every top-level field; unknown fields are refused (`plugin.manifest.invalid`) |
| [API levels](../../contract/v1/README.md#api-levels) | What levels 1–4 each add |
| [Capability versions (level 3)](../../contract/v1/README.md#capability-versions-level-3) | `requires`, the `hostCapabilities` table, and publishing rules across levels and versions |
| [Permissions and activation](../../contract/v1/README.md#permissions-and-activation) | Permission and activation event tables |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | Outcomes and stable error codes |
| [Windows mapping](../../contract/v1/README.md#windows-mapping) | How the legacy Windows extension manifest maps onto this contract |
| [Existing community store and Lua template](../../contract/v1/README.md#existing-community-store-and-lua-template) | Steps to move the community Lua template from level 1 to level 2 |

### Examples and cases

| File | Content |
| --- | --- |
| [`manifest/hello-lithe.plugin.json`](../../contract/v1/fixtures/manifest/hello-lithe.plugin.json) | The smallest level 2 UI plugin: one tool window and one command. Its ID is in the reserved test namespace and cannot be published as is |
| [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json) | 174 rejection cases across every domain. Each patches a valid fixture and names the stable rejection code and field |

### Related guides

- [Building an LSP plugin end to end](../../guides/lsp-plugin-lifecycle.md): manifest, packaging, state directories, installation, publishing
- [Plugin logging](../../guides/plugin-logging.md)
