# 语言、语言服务器与运行/测试 / Languages, language servers, run and test

[中文](#中文) · [English](#english)

## 中文

让 Lithe 认识一种语言：识别文件，用包内 Monarch 语法高亮，启动 LSP 语言服务器，提供“运行当前文件”和测试。全部是数据声明，宿主负责进程生命周期、LSP 会话和停止进程，插件不需要写宿主代码。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `languages`、`languageServers`、`executionProviders` |
| 能力版本 | `languages` 1–2、`languageServers` 1–2、`serverProcess` 1–2、`executionProviders` 1，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | LSP；所有插件进程都可以协商 [`litheLogging` 1](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | [API-25](../api-catalog.md#现在能用的-api)：语言、语法、LSP |
| 权限 | `languageServer.launch`；声明 `run`、`test` 时加 `process.run`、`process.test` |
| 激活事件 | `onLanguage:<id>`、`onRun:<id>`、`onTest:<id>` |
| Windows | 级别 3 清单按宿主能力表准入：`languages` 1、`languageServers` 1–2、`serverProcess` 1。不支持 `languages` 2 的语法，也不声明 `executionProviders` |

### 各级别的变化

| 版本 | 新增 |
| --- | --- |
| 级别 1 | 语言、`packaged` 和 `systemCommand` 语言服务器、`node` 和 `go` 运行时依赖、运行和测试提供者 |
| 级别 2 | `runtime` 启动（包内运行时或宿主发现的运行时）、`java` 运行时、启动模板、`initializationOptions`、宿主集成（只限官方插件） |
| `languages` 2 | 包内 Monarch 语法 `grammar` 与编辑器配置 `configuration`；只有语法的插件不需要任何进程 |
| `languageServers` 2 | `workspaceConfiguration`：Core 按设置展开后回答 `workspace/configuration`，`immediate` 设置变化时不重启即可推送 |
| `serverProcess` 2 | `runSelectedRuntime`，只给调试适配器用，见[调试适配器](debug-adapters.md) |

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Languages](../../contract/v1/README.md#languages) | 语言 ID、扩展名、文件名、项目标记、依赖路径 |
| [Grammars and editor configuration (`languages` 2)](../../contract/v1/README.md#grammars-and-editor-configuration-languages-2) | Monarch 语法的上限和拒绝规则、注释与括号配置、只在可终止的 worker 中运行 |
| [Language servers](../../contract/v1/README.md#language-servers) | 启动方式、参数、探测参数、运行时依赖 |
| [Runtime launch (API 2)](../../contract/v1/README.md#runtime-launch-api-2) | 用 `java` 或 `node` 启动包内入口文件，可以自带运行时 |
| [Host integrations (API 2)](../../contract/v1/README.md#host-integrations-api-2) | 数据表达不了的行为由宿主代码实现，只限官方插件按名称声明 |
| [Launch templates (API 2)](../../contract/v1/README.md#launch-templates-api-2) | `${pluginDir}`、`${stateDir}`、`${workspaceStateDir}` 等模板变量 |
| [Workspace configuration (`languageServers` 2)](../../contract/v1/README.md#workspace-configuration-languageservers-2) | `workspaceConfiguration` 的展开与推送 |
| [Execution providers](../../contract/v1/README.md#execution-providers) | `run`、`test` 的可执行文件、参数模板、测试发现 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`languages/language-packaged-server.plugin.json`](../../contract/v1/fixtures/languages/language-packaged-server.plugin.json) | 级别 1：包内语言服务器加运行/测试提供者（PHP） |
| [`languages/language-system-command.plugin.json`](../../contract/v1/fixtures/languages/language-system-command.plugin.json) | 级别 1：用系统命令启动的语言服务器加运行/测试提供者（Go） |
| [`languages/node-runtime.plugin.json`](../../contract/v1/fixtures/languages/node-runtime.plugin.json) | 级别 2：用 `node` 运行时启动，带插件设置 |
| [`languages/language-host-integration.plugin.json`](../../contract/v1/fixtures/languages/language-host-integration.plugin.json) | 级别 2：声明宿主集成的语言服务器（Java） |
| [`languages/language-grammar.plugin.json`](../../contract/v1/fixtures/languages/language-grammar.plugin.json) | 级别 3、`languages` 2：只有语法、不启动进程的插件 |
| [`languages/grammars/http.monarch.json`](../../contract/v1/fixtures/languages/grammars/http.monarch.json) | 完整的 Monarch 语法示例 |
| [`languages/command-plans.json`](../../contract/v1/fixtures/languages/command-plans.json) | 35 个标准答案：运行/测试模板和语言服务器启动计划的展开结果 |
| [`languages/runtime-versions.json`](../../contract/v1/fixtures/languages/runtime-versions.json) | 12 个标准答案：`plugins.declarative.checkRuntimeVersion` 的运行时版本判断 |

语言相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。

### 相关指南

- [从零构建一个完整生命周期的 LSP 插件](../../guides/lsp-plugin-lifecycle.md)
- [为语言插件接入构建、运行与测试](../../guides/build-tool-integration.md)

---

## English

Teach Lithe a language: recognize its files, highlight them with a packaged Monarch grammar, start an LSP language server, and offer "run current file" and tests. Everything is declared as data. The host owns the process lifecycle, the LSP session, and stopping the process; the plugin writes no host code.

| Item | Value |
| --- | --- |
| Manifest fields | `languages`, `languageServers`, `executionProviders` |
| Capability versions | `languages` 1–2, `languageServers` 1–2, `serverProcess` 1–2, `executionProviders` 1; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | LSP; every plugin process may negotiate [`litheLogging` 1](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | [API-25](../api-catalog.md#apis-usable-now): languages, grammars, LSP |
| Permissions | `languageServer.launch`; `process.run` and `process.test` when `run` or `test` is declared |
| Activation events | `onLanguage:<id>`, `onRun:<id>`, `onTest:<id>` |
| Windows | Level 3 manifests are admitted by the host table: `languages` 1, `languageServers` 1–2, `serverProcess` 1. No `languages` 2 grammars, and `executionProviders` is not advertised |

### What each version adds

| Version | Adds |
| --- | --- |
| Level 1 | Languages, `packaged` and `systemCommand` language servers, `node` and `go` runtime dependencies, run and test providers |
| Level 2 | `runtime` launches (packaged or host-discovered runtime), the `java` runtime, launch templates, `initializationOptions`, host integrations (official plugins only) |
| `languages` 2 | Packaged Monarch `grammar` and editor `configuration`; a grammar-only plugin needs no process |
| `languageServers` 2 | `workspaceConfiguration`: Core expands it from settings, answers `workspace/configuration`, and pushes `immediate` setting changes without a restart |
| `serverProcess` 2 | `runSelectedRuntime`, used only by debug adapters; see [Debug adapters](debug-adapters.md) |

### Contract sections

| Section | Content |
| --- | --- |
| [Languages](../../contract/v1/README.md#languages) | Language IDs, extensions, file names, project markers, dependency paths |
| [Grammars and editor configuration (`languages` 2)](../../contract/v1/README.md#grammars-and-editor-configuration-languages-2) | Monarch limits and rejections, comment and bracket configuration, worker-only execution |
| [Language servers](../../contract/v1/README.md#language-servers) | Launch kinds, arguments, probe arguments, runtime dependencies |
| [Runtime launch (API 2)](../../contract/v1/README.md#runtime-launch-api-2) | Start a packaged entry with `java` or `node`, optionally with a packaged runtime |
| [Host integrations (API 2)](../../contract/v1/README.md#host-integrations-api-2) | Behavior data cannot express lives in host code; only official plugins opt in by name |
| [Launch templates (API 2)](../../contract/v1/README.md#launch-templates-api-2) | Template variables such as `${pluginDir}`, `${stateDir}`, `${workspaceStateDir}` |
| [Workspace configuration (`languageServers` 2)](../../contract/v1/README.md#workspace-configuration-languageservers-2) | Expanding and pushing `workspaceConfiguration` |
| [Execution providers](../../contract/v1/README.md#execution-providers) | `run` and `test` executables, argument templates, test discovery |

### Examples and cases

| File | Content |
| --- | --- |
| [`languages/language-packaged-server.plugin.json`](../../contract/v1/fixtures/languages/language-packaged-server.plugin.json) | Level 1: packaged language server with run and test providers (PHP) |
| [`languages/language-system-command.plugin.json`](../../contract/v1/fixtures/languages/language-system-command.plugin.json) | Level 1: system-command language server with run and test providers (Go) |
| [`languages/node-runtime.plugin.json`](../../contract/v1/fixtures/languages/node-runtime.plugin.json) | Level 2: launched with the `node` runtime, with plugin settings |
| [`languages/language-host-integration.plugin.json`](../../contract/v1/fixtures/languages/language-host-integration.plugin.json) | Level 2: a language server that declares a host integration (Java) |
| [`languages/language-grammar.plugin.json`](../../contract/v1/fixtures/languages/language-grammar.plugin.json) | Level 3, `languages` 2: a grammar-only plugin that starts no process |
| [`languages/grammars/http.monarch.json`](../../contract/v1/fixtures/languages/grammars/http.monarch.json) | A complete Monarch grammar |
| [`languages/command-plans.json`](../../contract/v1/fixtures/languages/command-plans.json) | 35 golden expansions of run/test templates and language-server launch plans |
| [`languages/runtime-versions.json`](../../contract/v1/fixtures/languages/runtime-versions.json) | 12 golden outcomes of `plugins.declarative.checkRuntimeVersion` |

Language rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json).

### Related guides

- [Building an LSP plugin end to end](../../guides/lsp-plugin-lifecycle.md)
- [Build, run, and test for language plugins](../../guides/build-tool-integration.md)
