# 声明式构建系统 / Declarative build systems

[中文](#中文) · [English](#english)

## 中文

`buildSystems` 用数据描述一个项目级构建工具（Cargo、Go、Gradle、Maven 等）：怎么识别项目根目录、找哪个可执行文件、怎么得到项目模型、有哪些任务和运行入口、哪些文件变化要重新同步。所有构建工具共用同一套模型和命令：Core 校验声明、检测实例、解码模型、展开每一条命令行，宿主只执行 Core 规划好的进程并把输出交回。所以社区插件加一个构建工具不需要改宿主，宿主代码也不按构建系统 ID 分支。模型需要程序计算时，再加一个[构建提供者](build-providers.md)。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `buildSystems[]`：`detection`、`invocation`、`options`、`model`、`tasks`、`run`、`sync`、`layout`、`languageImport`、`modelSharing`、`modelInputs` |
| 能力版本 | `buildSystems` 1–4；相关的 `executableResolution` 1–2、`buildModelInputs` 1、`settings` 2，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | 没有进程协议；`model.source: "provider"` 时走 [`litheBuild`](build-providers.md) |
| API 编号 | [API-29](../api-catalog.md#现在能用的-api)：构建、任务、测试、运行准备 |
| 权限 | `process.build` |
| 激活事件 | `onBuildSystem:<id>` |
| Windows | 宿主不声明 `buildSystems`，构建插件在 Windows 上安装前被拒绝 |

### 版本对照

| 版本 | 新增 | 契约章节 | 示例 |
| --- | --- | --- | --- |
| `buildSystems` 1 | 识别、可执行文件候选、选项、模型（`static`、`command` 标准输出、`command` 输出文件、`hostIntegration`、`provider`）、任务、运行入口、同步、布局、Java 语言服务器的项目导入 | [API level 3: build systems](../../contract/v1/README.md#api-level-3-build-systems) | `build-static-file`、`build-command-stdout`、`build-command-output-file`、`build-host-integration` |
| `buildSystems` 2 | `invocation.executableSetting`。只为已发布的 Cargo 0.1.1 预览版保留，已弃用；新插件用 `executableResolution` 1 加 `buildSystems` 1 | [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | — |
| `buildSystems` 3 | 选项可以声明 `scope: "instance"`，按检测到的每个根目录分别保存 | [API level 3: build systems](../../contract/v1/README.md#api-level-3-build-systems) 字段表之后 | `build-instance-options` |
| `buildSystems` 4 | 项目任务的 `preparation`：按用途（如 `run`）和内容根角色绑定运行前要执行的任务；宿主不猜任务名 | [Declarative project preparation (buildSystems 4)](../../contract/v1/README.md#declarative-project-preparation-buildsystems-4) | `build-providers/build-provider-preparation` |
| `executableResolution` 1 | 可执行文件候选的第五种来源 `setting`，以及 `id`、`priority` 等排序元数据；配置的路径失败时不回退 | [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | `build-tool-path` |
| `executableResolution` 2 | 工具定义候选，以及项目/子项目的工具链绑定（Core；macOS 只读设置页） | 同上，以及[工具链](toolchains.md) | `build-toolchains` |
| `buildModelInputs` 1 | 生产者用 `modelSharing` 公开只读模型（项目、目标、内容根），消费者用 `modelInputs` 读取；消费者还需要 `buildProviders` 6 | [Optional read-only model inputs (buildModelInputs 1)](../../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | `build-model-producer`、`build-model-consumer` |

### 模型来源对照

| `model.source` | 怎么得到模型 | 示例 |
| --- | --- | --- |
| `static` | 读取项目里的文件 | `build-static-file`（Cargo） |
| `command`（标准输出） | 运行命令，解码标准输出 | `build-command-stdout`（Go） |
| `command`（输出文件） | 运行命令，解码它写出的文件 | `build-command-output-file`（Gradle） |
| `hostIntegration` | 交给宿主集成；只限官方插件，目前只有 `maven`，其他是 `plugin.integration.unsupported` | `build-host-integration`（Maven） |
| `provider` | 由插件的构建提供者回答 `lithe/build/model` | [构建提供者](build-providers.md) |

这些示例以 Cargo、Go、Gradle、Maven 为真实输入，但不是那些插件本身。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [API level 3: build systems](../../contract/v1/README.md#api-level-3-build-systems) | 字段表：识别、可执行文件、选项、模型、任务、运行、同步、布局、项目导入 |
| [Presentation](../../contract/v1/README.md#presentation) | 宿主绘制的构建窗口，插件不需要自带界面 |
| [Templates](../../contract/v1/README.md#templates) | 命令行模板变量 |
| [Model and outputs](../../contract/v1/README.md#model-and-outputs) | 统一模型、诊断、测试结果和依赖的格式 |
| [Optional read-only model inputs (buildModelInputs 1)](../../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | 插件之间共享只读模型 |
| [Declarative project preparation (buildSystems 4)](../../contract/v1/README.md#declarative-project-preparation-buildsystems-4) | 运行前准备任务 |
| [Shared executable candidates (`executableResolution` 1/2)](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | 可执行文件候选、排序、严格选择，以及 `buildSystems` 2 的兼容说明 |
| [Host integrations](../../contract/v1/README.md#host-integrations) | 构建系统的宿主集成 |
| [Commands](../../contract/v1/README.md#commands) | `build.*` Core 命令：检测、规划、解码、同步指纹 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`build-systems/build-static-file.plugin.json`](../../contract/v1/fixtures/build-systems/build-static-file.plugin.json) | `static` 模型（Cargo） |
| [`build-systems/build-command-stdout.plugin.json`](../../contract/v1/fixtures/build-systems/build-command-stdout.plugin.json) | 命令标准输出模型（Go） |
| [`build-systems/build-command-output-file.plugin.json`](../../contract/v1/fixtures/build-systems/build-command-output-file.plugin.json) | 命令输出文件模型（Gradle） |
| [`build-systems/build-host-integration.plugin.json`](../../contract/v1/fixtures/build-systems/build-host-integration.plugin.json) | 宿主集成模型（Maven） |
| [`build-systems/build-instance-options.plugin.json`](../../contract/v1/fixtures/build-systems/build-instance-options.plugin.json) | `buildSystems` 3：按实例保存的选项 |
| [`build-systems/build-tool-path.plugin.json`](../../contract/v1/fixtures/build-systems/build-tool-path.plugin.json) | `executableResolution` 1：用 `file` 设置选择构建工具 |
| [`build-systems/build-toolchains.plugin.json`](../../contract/v1/fixtures/build-systems/build-toolchains.plugin.json) | `buildSystems` 3 加 `executableResolution` 2：工具链绑定 |
| [`build-systems/build-model-producer.plugin.json`](../../contract/v1/fixtures/build-systems/build-model-producer.plugin.json) | `buildModelInputs` 1：公开模型的生产者 |
| [`build-systems/build-model-consumer.plugin.json`](../../contract/v1/fixtures/build-systems/build-model-consumer.plugin.json) | `buildModelInputs` 1 加 `buildProviders` 6：读取模型的消费者 |
| [`build-systems/build-plans.json`](../../contract/v1/fixtures/build-systems/build-plans.json) | 65 个 `build.*` 标准答案：每个宿主从上面的声明得到的计划和解码结果 |
| [`build-providers/build-provider-preparation.plugin.json`](../../contract/v1/fixtures/build-providers/build-provider-preparation.plugin.json) | `buildSystems` 4：运行前准备任务 |

构建相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。

### 相关指南

- [为语言插件接入构建、运行与测试](../../guides/build-tool-integration.md)
- [原生构建、测试与运行准备](../../guides/native-build-workflows.md)

---

## English

`buildSystems` describes a project-level build tool (Cargo, Go, Gradle, Maven, …) as data: how to recognize a project root, which executable to use, how to get the project model, which tasks and run entries exist, and which file changes require a re-sync. Every build tool shares the same model and commands: Core validates the declaration, detects instances, decodes the model, and expands every command line; the host only runs the processes Core planned and passes their output back. A community plugin therefore adds a build tool without any host change, and no host code branches on a build-system ID. When the model must be computed by a program, add a [build provider](build-providers.md).

| Item | Value |
| --- | --- |
| Manifest fields | `buildSystems[]`: `detection`, `invocation`, `options`, `model`, `tasks`, `run`, `sync`, `layout`, `languageImport`, `modelSharing`, `modelInputs` |
| Capability versions | `buildSystems` 1–4; related `executableResolution` 1–2, `buildModelInputs` 1, `settings` 2; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | No process protocol; `model.source: "provider"` uses [`litheBuild`](build-providers.md) |
| API IDs | [API-29](../api-catalog.md#apis-usable-now): build, tasks, tests, run preparation |
| Permissions | `process.build` |
| Activation events | `onBuildSystem:<id>` |
| Windows | `buildSystems` is not advertised; build plugins are refused before installation |

### Versions compared

| Version | Adds | Contract section | Example |
| --- | --- | --- | --- |
| `buildSystems` 1 | Detection, executable candidates, options, models (`static`, `command` stdout, `command` output file, `hostIntegration`, `provider`), tasks, run entries, sync, layout, Java language-server project import | [API level 3: build systems](../../contract/v1/README.md#api-level-3-build-systems) | `build-static-file`, `build-command-stdout`, `build-command-output-file`, `build-host-integration` |
| `buildSystems` 2 | `invocation.executableSetting`. Kept only for the published Cargo 0.1.1 preview and deprecated; new plugins use `executableResolution` 1 with `buildSystems` 1 | [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | — |
| `buildSystems` 3 | Options may declare `scope: "instance"`, stored separately per detected root | After the field table in [API level 3: build systems](../../contract/v1/README.md#api-level-3-build-systems) | `build-instance-options` |
| `buildSystems` 4 | Project-task `preparation`: binds the task to run first by purpose (such as `run`) and content-root role; the host never guesses task names | [Declarative project preparation (buildSystems 4)](../../contract/v1/README.md#declarative-project-preparation-buildsystems-4) | `build-providers/build-provider-preparation` |
| `executableResolution` 1 | A fifth executable source, `setting`, plus ordering metadata such as `id` and `priority`; a configured path that fails never falls back | [Shared executable candidates](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | `build-tool-path` |
| `executableResolution` 2 | Tool-definition candidates and project/subproject toolchain bindings (Core; macOS read-only settings) | Same, and [Toolchains](toolchains.md) | `build-toolchains` |
| `buildModelInputs` 1 | A producer publishes a read-only model (projects, targets, content roots) with `modelSharing`; a consumer reads it with `modelInputs` and also needs `buildProviders` 6 | [Optional read-only model inputs (buildModelInputs 1)](../../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | `build-model-producer`, `build-model-consumer` |

### Model sources compared

| `model.source` | How the model is obtained | Example |
| --- | --- | --- |
| `static` | Read a project file | `build-static-file` (Cargo) |
| `command` (stdout) | Run a command and decode its standard output | `build-command-stdout` (Go) |
| `command` (output file) | Run a command and decode the file it writes | `build-command-output-file` (Gradle) |
| `hostIntegration` | Handed to a host integration; official plugins only, currently only `maven`, otherwise `plugin.integration.unsupported` | `build-host-integration` (Maven) |
| `provider` | The plugin's build provider answers `lithe/build/model` | [Build providers](build-providers.md) |

The examples use Cargo, Go, Gradle, and Maven as realistic inputs; they are not those plugins.

### Contract sections

| Section | Content |
| --- | --- |
| [API level 3: build systems](../../contract/v1/README.md#api-level-3-build-systems) | Field table: detection, executables, options, model, tasks, run, sync, layout, project import |
| [Presentation](../../contract/v1/README.md#presentation) | The host-drawn build window; the plugin brings no UI |
| [Templates](../../contract/v1/README.md#templates) | Command-line template variables |
| [Model and outputs](../../contract/v1/README.md#model-and-outputs) | The shared model, diagnostics, test results, and dependency formats |
| [Optional read-only model inputs (buildModelInputs 1)](../../contract/v1/README.md#optional-read-only-model-inputs-buildmodelinputs-1) | Sharing read-only models between plugins |
| [Declarative project preparation (buildSystems 4)](../../contract/v1/README.md#declarative-project-preparation-buildsystems-4) | Tasks that run before a launch |
| [Shared executable candidates (`executableResolution` 1/2)](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | Executable candidates, ordering, strict selection, and `buildSystems` 2 compatibility |
| [Host integrations](../../contract/v1/README.md#host-integrations) | Host integrations for build systems |
| [Commands](../../contract/v1/README.md#commands) | `build.*` Core commands: detection, plans, decoding, sync fingerprints |

### Examples and cases

| File | Content |
| --- | --- |
| [`build-systems/build-static-file.plugin.json`](../../contract/v1/fixtures/build-systems/build-static-file.plugin.json) | `static` model (Cargo) |
| [`build-systems/build-command-stdout.plugin.json`](../../contract/v1/fixtures/build-systems/build-command-stdout.plugin.json) | Command stdout model (Go) |
| [`build-systems/build-command-output-file.plugin.json`](../../contract/v1/fixtures/build-systems/build-command-output-file.plugin.json) | Command output-file model (Gradle) |
| [`build-systems/build-host-integration.plugin.json`](../../contract/v1/fixtures/build-systems/build-host-integration.plugin.json) | Host-integration model (Maven) |
| [`build-systems/build-instance-options.plugin.json`](../../contract/v1/fixtures/build-systems/build-instance-options.plugin.json) | `buildSystems` 3: per-instance options |
| [`build-systems/build-tool-path.plugin.json`](../../contract/v1/fixtures/build-systems/build-tool-path.plugin.json) | `executableResolution` 1: a `file` setting selects the build tool |
| [`build-systems/build-toolchains.plugin.json`](../../contract/v1/fixtures/build-systems/build-toolchains.plugin.json) | `buildSystems` 3 with `executableResolution` 2: toolchain bindings |
| [`build-systems/build-model-producer.plugin.json`](../../contract/v1/fixtures/build-systems/build-model-producer.plugin.json) | `buildModelInputs` 1: a model producer |
| [`build-systems/build-model-consumer.plugin.json`](../../contract/v1/fixtures/build-systems/build-model-consumer.plugin.json) | `buildModelInputs` 1 with `buildProviders` 6: a model consumer |
| [`build-systems/build-plans.json`](../../contract/v1/fixtures/build-systems/build-plans.json) | 65 golden `build.*` results: the plans and decoded outputs every host gets from the declarations above |
| [`build-providers/build-provider-preparation.plugin.json`](../../contract/v1/fixtures/build-providers/build-provider-preparation.plugin.json) | `buildSystems` 4: tasks that run before a launch |

Build rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json).

### Related guides

- [Build, run, and test for language plugins](../../guides/build-tool-integration.md)
- [Native build, test, and run preparation](../../guides/native-build-workflows.md)
