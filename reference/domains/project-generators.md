# 项目生成器 / Project generators

[中文](#中文) · [English](#english)

## 中文

插件可以在宿主的"新建项目"窗口里加入口：某种语言的普通项目，或框架、脚手架生成器。窗口归宿主：入口列表和搜索、项目名称和位置、"创建 Git 仓库"选项、空项目、"创建"按钮。生成器只声明自己的字段、结果需要的语言和构建系统、生成项目的步骤。Core 计算表单和生成计划，宿主执行，宿主代码不按生成器 ID 分支。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `projectGenerators[]`：`id`、`displayName`、`section`、`order`、`icon`、`fields`、`requirements`、`steps`、`openFiles` |
| 能力版本 | `projectGenerators` 1、2，级别 3。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | 没有进程协议；生成器只是数据，由 Core 命令计算 |
| API 编号 | [API-33](../api-catalog.md#现在能用的-api)：项目生成器 |
| 权限 | `project.generate` |
| 激活事件 | 不需要 |
| Windows | 不支持；Windows 宿主不声明 `projectGenerators` |

### 版本对照

| 版本 | 最低级别 | 新增 | Core | macOS | Windows |
| --- | --- | --- | --- | --- | --- |
| `projectGenerators` 1 | 3 | 两个分区（`project`、`generator`）；五种字段（`text`、`enum`、`boolean`、`multiSelect`、`runtime`）；条件、模板；三种步骤（`files`、`command`、`archive`）；五个 Core 命令 | 支持 | 支持 | 不支持 |
| `projectGenerators` 2 | 3 | `tool` 字段（插件声明的工具定义的已有安装）；`${field.<id>.executable}`；计划返回 `toolPins`，宿主存为项目的工具固定；表单输入 `tools`、结果 `installations`、问题码 `toolTooOld`；目录条目 `tools`。不能再用 `runtime` 字段；版本 1 不能用 `tool` 字段 | 支持 | 支持 | 不支持 |

### 字段、步骤与命令

| 部分 | 内容 |
| --- | --- |
| `section` | `project` 放在"新建项目"下（某种语言的普通项目），`generator` 放在"生成器"下（框架或脚手架）；按分区、`order`（0–1000，默认 100）、名称排序 |
| 字段类型 | `text`（`default` 模板、`pattern`、`maxLength`、`maximumVersion`）、`enum`（1–16 个选项）、`boolean`、`multiSelect`（1–128 个选项，可分组）、`runtime`（宿主发现的安装，目前是 `java`，只用于版本 1）、`tool`（版本 2：插件声明的工具定义的已有安装，按主目录记录；属性 `tool`、`minimumVersion`、`pinAs`、`optional`；没有 `default`，优先保留用户的选择，否则选最新版本，版本比较忽略末尾的 0） |
| 条件 | `when`、`visibleWhen`：`equals`、`notEquals`、`oneOf`；隐藏字段不校验，条件里视为不存在 |
| 清单模板 | `${project.name}`、`${field.<id>}`、`${field.<id>.version}`、`${field.<id>.major}`（只用于 `runtime`）、`${field.<id>.executable}`（只用于 `tool`：所选安装里的可执行文件路径），修饰符 `path`、`package`、`lower` |
| 模板文件 | 用 `{{ … }}`，因为 POM、Gradle 脚本里本来就有 `${…}`；`dot-` 开头的路径段变成点文件（`dot-gitignore` → `.gitignore`） |
| `files` 步骤 | 把包内模板目录写进项目，保留可执行位；最多 1024 个文件、单个 1 MiB、合计 8 MiB |
| `command` 步骤 | 像构建工具一样解析可执行文件，在暂存目录里运行；`timeoutSeconds` 默认 600、最多 1800；`collect` 把子目录内容上移到项目根 |
| `archive` 步骤 | 下载 zip 并用 `project.extractArchive` 解压；必须是 HTTPS（`localhost`、`127.0.0.1`、`[::1]` 可用 HTTP），不带凭据；最多 20,000 个条目、解压后 512 MiB |
| Core 命令 | `project.generatorCatalog`、`project.generatorForm`、`project.generationPlan`、`project.checkName`、`project.extractArchive` |

步骤在暂存目录里按顺序执行，结束后宿主把它移到最终位置（目标已存在时拒绝）、可选记录 `java` 运行时为项目 JDK（只用于版本 1）、把 `toolPins` 存为项目的工具固定（和工具设置页用的是同一个存储）、按需创建 Git 仓库并打开项目。失败或取消会删掉暂存目录，不留下任何东西（`plugin.project.generationFailed`）。`requirements` 只用来提示启用或去市场安装相应插件，不会阻止创建。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Project generators](../../contract/v1/README.md#project-generators) | 宿主和生成器各自负责的部分、清单字段 |
| [Fields](../../contract/v1/README.md#fields) | 字段类型、属性、项目名称规则 |
| [Tool fields](../../contract/v1/README.md#tool-fields) | `tool` 字段的属性、初始选择、最低版本，以及和 `runtime` 字段按版本互斥 |
| [Conditions](../../contract/v1/README.md#conditions) | `when`、`visibleWhen` |
| [Templates](../../contract/v1/README.md#templates-1) | 清单模板、模板文件、点文件 |
| [Steps](../../contract/v1/README.md#steps) | 三种步骤和限制 |
| [Commands](../../contract/v1/README.md#commands-1) | 五个 Core 命令、表单与计划格式、失败清理 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`project-generators/project-generator.plugin.json`](../../contract/v1/fixtures/project-generators/project-generator.plugin.json) | 三个生成器：`app`（`project` 分区，按构建系统条件写模板文件）、`archetype`（运行脚手架命令）、`initializr`（从可配置服务器下载压缩包） |
| [`project-generators/project-plans.json`](../../contract/v1/fixtures/project-generators/project-plans.json) | 22 个 `project.*` 命令的标准答案：表单默认值与派生、计划写模板、运行时过旧、运行命令、下载压缩包、拒绝非 HTTPS 服务器、项目名称检查；其中 12 个工具字段用例用 `manifest` 指定 `project-generator-tool-field.plugin.json`（选最新安装、保留用户选择、低于最低版本、没有安装、展开并固定、可选字段不固定、末尾 0 视为同一版本等） |
| [`project-generators/project-generator-tool-field.plugin.json`](../../contract/v1/fixtures/project-generators/project-generator-tool-field.plugin.json) | 两个用 `tool` 字段的生成器（`projectGenerators` 2）：`python-app`（`project` 分区，必填 `python` 工具字段，最低 3.11、固定到项目，用 `${field.python.executable}` 运行 `uv venv`）、`node-app`（`generator` 分区，可选 `node` 工具字段） |
| [`project-generators/templates/`](../../contract/v1/fixtures/project-generators/templates/) | `app` 的模板目录：`app/README.md`、`app/dot-gitignore`、`main/Main.java`（Java 示例），以及 `python-app` 用的 `tool-field/README.md`；`.gitattributes` 关闭换行转换，保证模板逐字节输出 |

生成器的清单拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json) 里，共 9 个，都和工具字段有关：版本 1 用 `tool` 字段、版本 2 用 `runtime` 字段、`tool` 缺失或不是工具定义 ID、`minimumVersion` 不是点分数字、`pinAs` 取值未知、工具字段带 `default`、非工具字段用 `.executable`、非工具字段带 `tool` 属性。计划层面的拒绝（运行时或工具过旧、非 HTTPS 服务器、名称带斜杠）在 `project-plans.json` 里。

---

## English

A plugin can add entries to the host's New Project window: a plain project of a language, or a framework or scaffolding generator. The window belongs to the host: the entry list and its search, the project name and location, the "create Git repository" option, the empty project, and the Create button. A generator declares only its own fields, the languages and build systems the result needs, and the steps that produce it. Core computes the form and the generation plan, the host executes it, and no host code branches on a generator ID.

| Item | Value |
| --- | --- |
| Manifest field | `projectGenerators[]`: `id`, `displayName`, `section`, `order`, `icon`, `fields`, `requirements`, `steps`, `openFiles` |
| Capability versions | `projectGenerators` 1 and 2, at level 3. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | No process protocol; a generator is data that Core commands evaluate |
| API IDs | [API-33](../api-catalog.md#apis-usable-now): project generators |
| Permissions | `project.generate` |
| Activation events | None needed |
| Windows | Not supported; the Windows host does not advertise `projectGenerators` |

### Versions compared

| Version | Lowest level | Adds | Core | macOS | Windows |
| --- | --- | --- | --- | --- | --- |
| `projectGenerators` 1 | 3 | Two sections (`project`, `generator`); five field types (`text`, `enum`, `boolean`, `multiSelect`, `runtime`); conditions and templates; three step kinds (`files`, `command`, `archive`); five Core commands | Yes | Yes | No |
| `projectGenerators` 2 | 3 | `tool` fields (an existing installation of a plugin-declared tool definition); `${field.<id>.executable}`; the plan returns `toolPins`, which the host saves as the project's tool pins; form input `tools`, result `installations`, the `toolTooOld` problem; catalog entry `tools`. `runtime` fields are no longer allowed; version 1 does not allow `tool` fields | Yes | Yes | No |

### Fields, steps, and commands

| Part | Content |
| --- | --- |
| `section` | `project` lists the entry under "New Project" (a plain project of a language); `generator` lists it under "Generators" (a framework or scaffolding tool); sorted by section, `order` (0–1000, default 100), then name |
| Field types | `text` (`default` template, `pattern`, `maxLength`, `maximumVersion`), `enum` (1–16 options), `boolean`, `multiSelect` (1–128 options, optionally grouped), `runtime` (a host-discovered installation, currently `java`; version 1 only), `tool` (version 2: an existing installation of a plugin-declared tool definition, recorded by home directory; properties `tool`, `minimumVersion`, `pinAs`, `optional`; no `default`: the user's choice is kept, otherwise the newest version, and trailing zeros do not count) |
| Conditions | `when` and `visibleWhen`: `equals`, `notEquals`, `oneOf`; a hidden field is not validated and reads as absent in conditions |
| Manifest templates | `${project.name}`, `${field.<id>}`, `${field.<id>.version}`, `${field.<id>.major}` (runtime fields only), `${field.<id>.executable}` (tool fields only: the executable in the chosen installation), with the modifiers `path`, `package`, and `lower` |
| Template files | Use `{{ … }}`, because POMs and Gradle scripts already contain `${…}`; a path segment starting with `dot-` becomes a dotfile (`dot-gitignore` → `.gitignore`) |
| `files` step | Writes a packaged template directory into the project, keeping the executable bit; at most 1024 files, 1 MiB each, 8 MiB in total |
| `command` step | Resolves the executable like a build tool and runs it in the staging directory; `timeoutSeconds` defaults to 600, at most 1800; `collect` moves a subdirectory's contents up into the project root |
| `archive` step | Downloads a zip and extracts it with `project.extractArchive`; HTTPS only (HTTP allowed for `localhost`, `127.0.0.1`, `[::1]`), no credentials; at most 20,000 entries and 512 MiB extracted |
| Core commands | `project.generatorCatalog`, `project.generatorForm`, `project.generationPlan`, `project.checkName`, `project.extractArchive` |

Steps run in order in a staging directory. Afterwards the host moves it to its final location (refusing an existing one), may record a `java` runtime as the project's JDK (version 1 only), saves the `toolPins` as the project's tool pins (the same store the tool settings page edits), creates the Git repository when asked, and opens the project. A failure or cancellation removes the staging directory and leaves nothing behind (`plugin.project.generationFailed`). `requirements` only prompt the user to enable or install the providing plugins; they never block creation.

### Contract sections

| Section | Content |
| --- | --- |
| [Project generators](../../contract/v1/README.md#project-generators) | What the host and the generator each own; manifest fields |
| [Fields](../../contract/v1/README.md#fields) | Field types, properties, project name rules |
| [Tool fields](../../contract/v1/README.md#tool-fields) | `tool` field properties, the initial selection, minimum versions, and how `runtime` and `tool` fields are split by version |
| [Conditions](../../contract/v1/README.md#conditions) | `when` and `visibleWhen` |
| [Templates](../../contract/v1/README.md#templates-1) | Manifest templates, template files, dotfiles |
| [Steps](../../contract/v1/README.md#steps) | The three step kinds and their limits |
| [Commands](../../contract/v1/README.md#commands-1) | The five Core commands, form and plan shapes, cleanup on failure |

### Examples and cases

| File | Content |
| --- | --- |
| [`project-generators/project-generator.plugin.json`](../../contract/v1/fixtures/project-generators/project-generator.plugin.json) | Three generators: `app` (the `project` section, writing template files conditioned on the build system), `archetype` (running a scaffolding command), `initializr` (downloading an archive from a configurable server) |
| [`project-generators/project-plans.json`](../../contract/v1/fixtures/project-generators/project-plans.json) | Golden answers for 22 `project.*` command cases: form defaults and derived values, a plan writing templates, a runtime that is too old, running a command, downloading an archive, refusing a non-HTTPS server, project name checks; 12 tool field cases name `project-generator-tool-field.plugin.json` in `manifest` (the newest installation, keeping the user's choice, below the minimum, no installation, expanding and pinning, an optional field left unpinned, trailing zeros as the same release, and more) |
| [`project-generators/project-generator-tool-field.plugin.json`](../../contract/v1/fixtures/project-generators/project-generator-tool-field.plugin.json) | Two generators with `tool` fields (`projectGenerators` 2): `python-app` (the `project` section; a required `python` tool field, at least 3.11, pinned to the project, running `uv venv` with `${field.python.executable}`) and `node-app` (the `generator` section; an optional `node` tool field) |
| [`project-generators/templates/`](../../contract/v1/fixtures/project-generators/templates/) | The `app` template directory: `app/README.md`, `app/dot-gitignore`, `main/Main.java` (a Java example), plus `tool-field/README.md` for `python-app`; `.gitattributes` disables line-ending conversion so templates are emitted byte for byte |

Generator manifest refusals are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json): 9 cases, all about tool fields (a `tool` field under version 1, a `runtime` field under version 2, a missing `tool` or one that is not a tool definition ID, a `minimumVersion` that is not dotted numbers, an unknown `pinAs`, a `default` on a tool field, `.executable` on a non-tool field, a `tool` property on a non-tool field). Plan-level refusals (a runtime or tool that is too old, a non-HTTPS server, a name with a slash) are in `project-plans.json`.
