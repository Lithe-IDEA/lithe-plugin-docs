# 构建提供者（litheBuild） / Build providers (litheBuild)

[中文](#中文) · [English](#english)

## 中文

有些构建工具只能通过自己的 API 给出项目模型（Gradle Tooling API、BEAM 里的 Mix）。这类插件在 `buildSystems[].provider` 声明一个构建提供者：一个和语言服务器同类的长驻 LSP 基础协议进程，回答 `lithe/build/*` 请求。从版本 2 起，任务、测试、运行准备、调试准备、原生配置也可以交给它。宿主仍然负责启动和停止进程、显示界面、运行最终的程序。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `buildSystems[].provider`，以及各声明里的 `source: "provider"` |
| 能力版本 | `buildProviders` 1–11；每个提供者都需要 `buildSystems`，自有进程的提供者还需要 `serverProcess` 1，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheBuild` 1–11：宿主在 `initialize` 里提供 `capabilities.experimental.litheBuild: {"versions": [11, …, 1]}`，提供者回答选中的 `{"version": n}`，见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | [API-29](../api-catalog.md#现在能用的-api)：构建、任务、测试、运行准备 |
| 权限 | `process.build`；自有进程的提供者加 `languageServer.launch`；调试准备加 `debug.launch` |
| 激活事件 | `onBuildSystem:<id>`；提供者在第一次由用户操作触发的构建请求时启动 |
| Windows | 宿主不声明 `buildProviders` |
| 官方 SDK | [Node](../../sdk/node/README.md) 和 [Java](../../sdk/java/README.md) 实现 `litheBuild` 1–6；[Rust](../../sdk/rust/README.md) 不实现 |

### 两种形式

| 形式 | 字段 | 说明 |
| --- | --- | --- |
| 共享 | `{"languageServer": "<languageID>", "executesProjectCode", "mayUseNetwork"}` | 请求发给同一插件里该语言的语言服务器会话 |
| 自有进程 | 语言服务器的启动字段（`launch`、`arguments`、`validationArguments`、`runtimeDependencies`、`initializationOptions`）加 `executesProjectCode`、`mayUseNetwork` | 安装审查列为 `packagedBuildProvider`、`systemBuildProvider` 或 `runtimeBuildProvider`；原生配置（版本 7）只能用这种形式 |

### 版本对照

清单里的 `requires.buildProviders` 和 `initialize` 里协商的 `litheBuild` 用同一套版本号。每个版本都保留之前所有版本；提供者回答的版本低于某个功能所需版本时，宿主在调用前拒绝该功能。

| 版本 | 新增 | 方法 | 契约章节 | 示例 | Node / Java SDK |
| --- | --- | --- | --- | --- | --- |
| 1 | 项目模型和依赖：`model.source`、`dependencies.source` 为 `provider` | `lithe/build/model`、`lithe/build/dependencies` | [Build providers](../../contract/v1/README.md#build-providers) | `build-provider`、`build-provider-language-server` | 支持 |
| 2 | 任务执行：`tasks[].source: "provider"` | `lithe/build/execute`、`lithe/build/progress` | [Provider task execution (buildProviders 2)](../../contract/v1/README.md#provider-task-execution-buildproviders-2) | `build-provider-execution`、`build-execution-protocol.json` | 支持 |
| 3 | 测试执行和测试发现：`test.source: "provider"` | `lithe/build/test`、`lithe/build/discoverTests` | [Provider test execution (buildProviders 3)](../../contract/v1/README.md#provider-test-execution-buildproviders-3)、[Test discovery](../../contract/v1/README.md#test-discovery) | `build-provider-tests`、`build-test-protocol.json`、`build-test-discovery.json` | 支持 |
| 4 | 应用运行准备：`run[].source: "provider"`，只限应用目标 | `lithe/build/prepareRun` | [Provider run preparation (buildProviders 4)](../../contract/v1/README.md#provider-run-preparation-buildproviders-4) | `build-provider-run-preparation`、`build-run-preparation.json` | 支持 |
| 5 | 原生调试准备：运行声明 `debug: {"adapter": "native"}`，同时需要 `debugAdapters` 2 和级别 4 | `lithe/build/prepareDebug` | [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | `build-provider-debug-preparation`、`build-debug-preparation.json` | 支持 |
| 6 | 接收宿主显式声明的只读模型输入（`modelInputs`，同时需要 `buildModelInputs` 1） | 沿用版本 1 的方法 | [Optional read-only model inputs (buildModelInputs 1)](../../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | `build-systems/build-model-consumer` | 支持 |
| 7 | 以来源为准的原生配置：`configuration: {"source": "provider"}`，只限自有进程 | `lithe/build/configuration`、`lithe/build/editConfiguration` | [Optional native configuration (buildProviders 7)](../../contract/v1/README.md#optional-native-configuration-buildproviders-7) | `build-provider-configuration`、`build-configuration.json` | 不支持 |
| 8 | 内联测试报告：`results: {"decoder": "provider"}`，回复里带 `testResults` | 沿用 `lithe/build/test`、`execute` | [Inline provider test reports (buildProviders 8)](../../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) | `build-provider-reports`、`build-test-durations.json`（重复用例耗时求和） | 不支持 |
| 9 | 不透明的语义测试选择器：`selectorKinds`，请求带 `selectorKind`、`selectors` | 沿用 `lithe/build/test` | [Opaque semantic test selectors (buildProviders 9)](../../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) | `build-provider-selectors` | 不支持 |
| 10 | 类型化运行选项，服务和任务的运行准备：`execution: "service"` 或 `"task"` | 扩展 `lithe/build/prepareRun` | [Typed run options and service/task preparation (version 10)](../../contract/v1/README.md#typed-run-options-and-servicetask-preparation-version-10) | `build-provider-run-options`、`build-run-options.json` | 不支持 |
| 11 | 注册表连接调试：服务运行声明 `debug: {"configurationType": "…"}`，由宿主选择已启用的 attach 适配器 | 沿用 `lithe/build/prepareRun` | [Registry connection debugging (buildProviders 11)](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | `build-connection-debug-options.json` | 不支持 |

未注明目录的示例都在 `fixtures/build-providers/` 下。版本 11 没有单独的清单示例。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Build providers](../../contract/v1/README.md#build-providers) | 两种形式、请求参数、`initialize` 协商、进程隔离和生命周期 |
| [Provider task execution (buildProviders 2)](../../contract/v1/README.md#provider-task-execution-buildproviders-2) | 任务执行、进度、取消 |
| [Provider test execution (buildProviders 3)](../../contract/v1/README.md#provider-test-execution-buildproviders-3) | 测试执行、报告、测试发现 |
| [Provider run preparation (buildProviders 4)](../../contract/v1/README.md#provider-run-preparation-buildproviders-4) | 运行准备，以及版本 10、11 的扩展 |
| [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | 原生调试准备和适配器绑定 |
| [Optional native configuration (buildProviders 7)](../../contract/v1/README.md#optional-native-configuration-buildproviders-7) | 原生配置的读取和分阶段编辑 |
| [Inline provider test reports (buildProviders 8)](../../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) | 回复里的 `litheTestResults` 报告 |
| [Opaque semantic test selectors (buildProviders 9)](../../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) | 语义测试选择器 |
| [Model and outputs](../../contract/v1/README.md#model-and-outputs) | 提供者结果要符合的模型、诊断、依赖格式 |

### 文档格式

| 格式 | 版本 | 用在 | 示例 |
| --- | --- | --- | --- |
| `litheBuildModel` | 1 | 版本 1 的模型结果 | [`build-plans.json`](../../contract/v1/fixtures/build-systems/build-plans.json) |
| `litheTestResults` | 1 | 版本 3 的测试报告、版本 8 的内联报告 | 契约正文内联示例，见 [Inline provider test reports](../../contract/v1/README.md#inline-provider-test-reports-buildproviders-8)；重复用例的耗时求和规则见 [`build-test-durations.json`](../../contract/v1/fixtures/build-providers/build-test-durations.json) |
| `litheTestDiscovery` | 1 | 版本 3 的测试发现 | [`build-test-discovery.json`](../../contract/v1/fixtures/build-providers/build-test-discovery.json) |
| `litheBuildConfiguration` | 1 | 版本 7 的原生配置 | [`build-configuration.json`](../../contract/v1/fixtures/build-providers/build-configuration.json) |
| `launchOptions` 对象 | — | 版本 10、11 的运行选项；示例文件外层标为 `format: "litheBuildRunOptions"`, `version: 1` | [`build-run-options.json`](../../contract/v1/fixtures/build-providers/build-run-options.json)、[`build-connection-debug-options.json`](../../contract/v1/fixtures/build-providers/build-connection-debug-options.json) |

所有格式版本见[文档格式版本](../capabilities-and-versions.md#文档格式版本)。

### 相关指南

- [原生构建、测试与运行准备](../../guides/native-build-workflows.md)
- [为语言插件接入构建、运行与测试](../../guides/build-tool-integration.md)

---

## English

Some build tools expose their project model only through their own API (the Gradle Tooling API, Mix inside the BEAM). Such a plugin declares a build provider in `buildSystems[].provider`: a long-lived LSP base-protocol process of the same kind as a language server, which answers `lithe/build/*` requests. From version 2 on, tasks, tests, run preparation, debug preparation, and native configuration can be handed to it too. The host still starts and stops the process, draws the UI, and runs the final program.

| Item | Value |
| --- | --- |
| Manifest fields | `buildSystems[].provider`, and `source: "provider"` in individual declarations |
| Capability versions | `buildProviders` 1–11; every provider needs `buildSystems`, and an own-process provider also needs `serverProcess` 1; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheBuild` 1–11: the host offers `capabilities.experimental.litheBuild: {"versions": [11, …, 1]}` in `initialize`, and the provider answers the chosen `{"version": n}`; see [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | [API-29](../api-catalog.md#apis-usable-now): build, tasks, tests, run preparation |
| Permissions | `process.build`; `languageServer.launch` for an own-process provider; `debug.launch` for debug preparation |
| Activation events | `onBuildSystem:<id>`; a provider starts on the first build request, which only a user action sends |
| Windows | `buildProviders` is not advertised |
| Official SDKs | [Node](../../sdk/node/README.md) and [Java](../../sdk/java/README.md) implement `litheBuild` 1–6; [Rust](../../sdk/rust/README.md) does not |

### Two forms

| Form | Fields | Notes |
| --- | --- | --- |
| Shared | `{"languageServer": "<languageID>", "executesProjectCode", "mayUseNetwork"}` | Requests go to that language's server session in the same plugin |
| Own process | The language-server launch fields (`launch`, `arguments`, `validationArguments`, `runtimeDependencies`, `initializationOptions`) plus `executesProjectCode` and `mayUseNetwork` | Listed in the install review as `packagedBuildProvider`, `systemBuildProvider`, or `runtimeBuildProvider`; native configuration (version 7) requires this form |

### Versions compared

`requires.buildProviders` in the manifest and the `litheBuild` version negotiated in `initialize` share one numbering. Each version keeps every earlier one; when the provider answers a version below what a feature needs, the host refuses that feature before calling it.

| Version | Adds | Methods | Contract section | Example | Node / Java SDK |
| --- | --- | --- | --- | --- | --- |
| 1 | Project model and dependencies: `model.source` and `dependencies.source` set to `provider` | `lithe/build/model`, `lithe/build/dependencies` | [Build providers](../../contract/v1/README.md#build-providers) | `build-provider`, `build-provider-language-server` | Yes |
| 2 | Task execution: `tasks[].source: "provider"` | `lithe/build/execute`, `lithe/build/progress` | [Provider task execution (buildProviders 2)](../../contract/v1/README.md#provider-task-execution-buildproviders-2) | `build-provider-execution`, `build-execution-protocol.json` | Yes |
| 3 | Test execution and discovery: `test.source: "provider"` | `lithe/build/test`, `lithe/build/discoverTests` | [Provider test execution (buildProviders 3)](../../contract/v1/README.md#provider-test-execution-buildproviders-3), [Test discovery](../../contract/v1/README.md#test-discovery) | `build-provider-tests`, `build-test-protocol.json`, `build-test-discovery.json` | Yes |
| 4 | Application run preparation: `run[].source: "provider"`, application targets only | `lithe/build/prepareRun` | [Provider run preparation (buildProviders 4)](../../contract/v1/README.md#provider-run-preparation-buildproviders-4) | `build-provider-run-preparation`, `build-run-preparation.json` | Yes |
| 5 | Native debug preparation: a run declares `debug: {"adapter": "native"}`; also needs `debugAdapters` 2 and level 4 | `lithe/build/prepareDebug` | [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | `build-provider-debug-preparation`, `build-debug-preparation.json` | Yes |
| 6 | Accepts read-only model inputs the host declares explicitly (`modelInputs`; also needs `buildModelInputs` 1) | The version 1 methods | [Optional read-only model inputs (buildModelInputs 1)](../../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | `build-systems/build-model-consumer` | Yes |
| 7 | Source-backed native configuration: `configuration: {"source": "provider"}`, own-process providers only | `lithe/build/configuration`, `lithe/build/editConfiguration` | [Optional native configuration (buildProviders 7)](../../contract/v1/README.md#optional-native-configuration-buildproviders-7) | `build-provider-configuration`, `build-configuration.json` | No |
| 8 | Inline test reports: `results: {"decoder": "provider"}`, replies carry `testResults` | `lithe/build/test`, `execute` | [Inline provider test reports (buildProviders 8)](../../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) | `build-provider-reports`, `build-test-durations.json` (summing repeated case durations) | No |
| 9 | Opaque semantic test selectors: `selectorKinds`; requests carry `selectorKind` and `selectors` | `lithe/build/test` | [Opaque semantic test selectors (buildProviders 9)](../../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) | `build-provider-selectors` | No |
| 10 | Typed run options; service and task run preparation: `execution: "service"` or `"task"` | Extends `lithe/build/prepareRun` | [Typed run options and service/task preparation (version 10)](../../contract/v1/README.md#typed-run-options-and-servicetask-preparation-version-10) | `build-provider-run-options`, `build-run-options.json` | No |
| 11 | Registry connection debugging: a service run declares `debug: {"configurationType": "…"}` and the host picks an enabled attach adapter | `lithe/build/prepareRun` | [Registry connection debugging (buildProviders 11)](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | `build-connection-debug-options.json` | No |

Examples without a directory are in `fixtures/build-providers/`. Version 11 has no manifest example of its own.

### Contract sections

| Section | Content |
| --- | --- |
| [Build providers](../../contract/v1/README.md#build-providers) | The two forms, request params, `initialize` negotiation, process isolation and lifecycle |
| [Provider task execution (buildProviders 2)](../../contract/v1/README.md#provider-task-execution-buildproviders-2) | Task execution, progress, cancellation |
| [Provider test execution (buildProviders 3)](../../contract/v1/README.md#provider-test-execution-buildproviders-3) | Test execution, reports, discovery |
| [Provider run preparation (buildProviders 4)](../../contract/v1/README.md#provider-run-preparation-buildproviders-4) | Run preparation, and the version 10 and 11 extensions |
| [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | Native debug preparation and adapter bindings |
| [Optional native configuration (buildProviders 7)](../../contract/v1/README.md#optional-native-configuration-buildproviders-7) | Reading and staged editing of native configuration |
| [Inline provider test reports (buildProviders 8)](../../contract/v1/README.md#inline-provider-test-reports-buildproviders-8) | `litheTestResults` reports in replies |
| [Opaque semantic test selectors (buildProviders 9)](../../contract/v1/README.md#opaque-semantic-test-selectors-buildproviders-9) | Semantic test selectors |
| [Model and outputs](../../contract/v1/README.md#model-and-outputs) | The model, diagnostics, and dependency formats a provider result must follow |

### Document formats

| Format | Version | Used by | Example |
| --- | --- | --- | --- |
| `litheBuildModel` | 1 | Version 1 model results | [`build-plans.json`](../../contract/v1/fixtures/build-systems/build-plans.json) |
| `litheTestResults` | 1 | Version 3 test reports, version 8 inline reports | Inline in the contract, see [Inline provider test reports](../../contract/v1/README.md#inline-provider-test-reports-buildproviders-8); the rule for summing repeated case durations is in [`build-test-durations.json`](../../contract/v1/fixtures/build-providers/build-test-durations.json) |
| `litheTestDiscovery` | 1 | Version 3 test discovery | [`build-test-discovery.json`](../../contract/v1/fixtures/build-providers/build-test-discovery.json) |
| `litheBuildConfiguration` | 1 | Version 7 native configuration | [`build-configuration.json`](../../contract/v1/fixtures/build-providers/build-configuration.json) |
| `launchOptions` object | — | Version 10 and 11 run options; the fixture files wrap it as `format: "litheBuildRunOptions"`, `version: 1` | [`build-run-options.json`](../../contract/v1/fixtures/build-providers/build-run-options.json), [`build-connection-debug-options.json`](../../contract/v1/fixtures/build-providers/build-connection-debug-options.json) |

All format versions are in [document format versions](../capabilities-and-versions.md#document-format-versions).

### Related guides

- [Native build, test, and run preparation](../../guides/native-build-workflows.md)
- [Build, run, and test for language plugins](../../guides/build-tool-integration.md)
