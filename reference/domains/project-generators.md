# 项目生成器 / Project generators

[中文](#中文) · [English](#english)

## 中文

插件可以在宿主的"新建项目"窗口里加入口：某种语言的普通项目，或框架、脚手架生成器。窗口归宿主：入口列表和搜索、项目名称和位置、"创建 Git 仓库"选项、空项目、"创建"按钮。生成器只声明自己的字段、结果需要的语言和构建系统、生成项目的步骤。Core 计算表单和生成计划，宿主执行，宿主代码不按生成器 ID 分支。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `projectGenerators[]`：`id`、`displayName`、`section`、`order`、`icon`、`fields`、`requirements`、`steps`、`openFiles` |
| 能力版本 | `projectGenerators` 1，级别 3。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | 没有进程协议；生成器只是数据，由 Core 命令计算 |
| API 编号 | [API-33](../api-catalog.md#现在能用的-api)：项目生成器 |
| 权限 | `project.generate` |
| 激活事件 | 不需要 |
| Windows | 不支持；Windows 宿主不声明 `projectGenerators` |

### 版本对照

| 版本 | 最低级别 | 新增 | Core | macOS | Windows |
| --- | --- | --- | --- | --- | --- |
| `projectGenerators` 1 | 3 | 两个分区（`project`、`generator`）；五种字段（`text`、`enum`、`boolean`、`multiSelect`、`runtime`）；条件、模板；三种步骤（`files`、`command`、`archive`）；五个 Core 命令 | 支持 | 支持 | 不支持 |

### 字段、步骤与命令

| 部分 | 内容 |
| --- | --- |
| `section` | `project` 放在"新建项目"下（某种语言的普通项目），`generator` 放在"生成器"下（框架或脚手架）；按分区、`order`（0–1000，默认 100）、名称排序 |
| 字段类型 | `text`（`default` 模板、`pattern`、`maxLength`、`maximumVersion`）、`enum`（1–16 个选项）、`boolean`、`multiSelect`（1–128 个选项，可分组）、`runtime`（宿主发现的安装，目前是 `java`） |
| 条件 | `when`、`visibleWhen`：`equals`、`notEquals`、`oneOf`；隐藏字段不校验，条件里视为不存在 |
| 清单模板 | `${project.name}`、`${field.<id>}`、`${field.<id>.version}`、`${field.<id>.major}`，修饰符 `path`、`package`、`lower` |
| 模板文件 | 用 `{{ … }}`，因为 POM、Gradle 脚本里本来就有 `${…}`；`dot-` 开头的路径段变成点文件（`dot-gitignore` → `.gitignore`） |
| `files` 步骤 | 把包内模板目录写进项目，保留可执行位；最多 1024 个文件、单个 1 MiB、合计 8 MiB |
| `command` 步骤 | 像构建工具一样解析可执行文件，在暂存目录里运行；`timeoutSeconds` 默认 600、最多 1800；`collect` 把子目录内容上移到项目根 |
| `archive` 步骤 | 下载 zip 并用 `project.extractArchive` 解压；必须是 HTTPS（`localhost`、`127.0.0.1`、`[::1]` 可用 HTTP），不带凭据；最多 20,000 个条目、解压后 512 MiB |
| Core 命令 | `project.generatorCatalog`、`project.generatorForm`、`project.generationPlan`、`project.checkName`、`project.extractArchive` |

步骤在暂存目录里按顺序执行，结束后宿主把它移到最终位置（目标已存在时拒绝）、可选记录 `java` 运行时为项目 JDK、按需创建 Git 仓库并打开项目。失败或取消会删掉暂存目录，不留下任何东西（`plugin.project.generationFailed`）。`requirements` 只用来提示启用或去市场安装相应插件，不会阻止创建。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Project generators](../../contract/v1/README.md#project-generators) | 宿主和生成器各自负责的部分、清单字段 |
| [Fields](../../contract/v1/README.md#fields) | 字段类型、属性、项目名称规则 |
| [Conditions](../../contract/v1/README.md#conditions) | `when`、`visibleWhen` |
| [Templates](../../contract/v1/README.md#templates-1) | 清单模板、模板文件、点文件 |
| [Steps](../../contract/v1/README.md#steps) | 三种步骤和限制 |
| [Commands](../../contract/v1/README.md#commands-1) | 五个 Core 命令、表单与计划格式、失败清理 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`project-generators/project-generator.plugin.json`](../../contract/v1/fixtures/project-generators/project-generator.plugin.json) | 三个生成器：`app`（`project` 分区，按构建系统条件写模板文件）、`archetype`（运行脚手架命令）、`initializr`（从可配置服务器下载压缩包） |
| [`project-generators/project-plans.json`](../../contract/v1/fixtures/project-generators/project-plans.json) | 10 个 `project.*` 命令的标准答案：表单默认值与派生、计划写模板、运行时过旧、运行命令、下载压缩包、拒绝非 HTTPS 服务器、项目名称检查 |
| [`project-generators/templates/`](../../contract/v1/fixtures/project-generators/templates/) | `app` 的模板目录：`app/README.md`、`app/dot-gitignore`、`main/Main.java`（Java 示例）；`.gitattributes` 关闭换行转换，保证模板逐字节输出 |

共享示例里没有生成器的清单拒绝用例；计划层面的拒绝（运行时过旧、非 HTTPS 服务器、名称带斜杠）在 `project-plans.json` 里。

---

## English

A plugin can add entries to the host's New Project window: a plain project of a language, or a framework or scaffolding generator. The window belongs to the host: the entry list and its search, the project name and location, the "create Git repository" option, the empty project, and the Create button. A generator declares only its own fields, the languages and build systems the result needs, and the steps that produce it. Core computes the form and the generation plan, the host executes it, and no host code branches on a generator ID.

| Item | Value |
| --- | --- |
| Manifest field | `projectGenerators[]`: `id`, `displayName`, `section`, `order`, `icon`, `fields`, `requirements`, `steps`, `openFiles` |
| Capability versions | `projectGenerators` 1, at level 3. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | No process protocol; a generator is data that Core commands evaluate |
| API IDs | [API-33](../api-catalog.md#apis-usable-now): project generators |
| Permissions | `project.generate` |
| Activation events | None needed |
| Windows | Not supported; the Windows host does not advertise `projectGenerators` |

### Versions compared

| Version | Lowest level | Adds | Core | macOS | Windows |
| --- | --- | --- | --- | --- | --- |
| `projectGenerators` 1 | 3 | Two sections (`project`, `generator`); five field types (`text`, `enum`, `boolean`, `multiSelect`, `runtime`); conditions and templates; three step kinds (`files`, `command`, `archive`); five Core commands | Yes | Yes | No |

### Fields, steps, and commands

| Part | Content |
| --- | --- |
| `section` | `project` lists the entry under "New Project" (a plain project of a language); `generator` lists it under "Generators" (a framework or scaffolding tool); sorted by section, `order` (0–1000, default 100), then name |
| Field types | `text` (`default` template, `pattern`, `maxLength`, `maximumVersion`), `enum` (1–16 options), `boolean`, `multiSelect` (1–128 options, optionally grouped), `runtime` (a host-discovered installation, currently `java`) |
| Conditions | `when` and `visibleWhen`: `equals`, `notEquals`, `oneOf`; a hidden field is not validated and reads as absent in conditions |
| Manifest templates | `${project.name}`, `${field.<id>}`, `${field.<id>.version}`, `${field.<id>.major}`, with the modifiers `path`, `package`, and `lower` |
| Template files | Use `{{ … }}`, because POMs and Gradle scripts already contain `${…}`; a path segment starting with `dot-` becomes a dotfile (`dot-gitignore` → `.gitignore`) |
| `files` step | Writes a packaged template directory into the project, keeping the executable bit; at most 1024 files, 1 MiB each, 8 MiB in total |
| `command` step | Resolves the executable like a build tool and runs it in the staging directory; `timeoutSeconds` defaults to 600, at most 1800; `collect` moves a subdirectory's contents up into the project root |
| `archive` step | Downloads a zip and extracts it with `project.extractArchive`; HTTPS only (HTTP allowed for `localhost`, `127.0.0.1`, `[::1]`), no credentials; at most 20,000 entries and 512 MiB extracted |
| Core commands | `project.generatorCatalog`, `project.generatorForm`, `project.generationPlan`, `project.checkName`, `project.extractArchive` |

Steps run in order in a staging directory. Afterwards the host moves it to its final location (refusing an existing one), may record a `java` runtime as the project's JDK, creates the Git repository when asked, and opens the project. A failure or cancellation removes the staging directory and leaves nothing behind (`plugin.project.generationFailed`). `requirements` only prompt the user to enable or install the providing plugins; they never block creation.

### Contract sections

| Section | Content |
| --- | --- |
| [Project generators](../../contract/v1/README.md#project-generators) | What the host and the generator each own; manifest fields |
| [Fields](../../contract/v1/README.md#fields) | Field types, properties, project name rules |
| [Conditions](../../contract/v1/README.md#conditions) | `when` and `visibleWhen` |
| [Templates](../../contract/v1/README.md#templates-1) | Manifest templates, template files, dotfiles |
| [Steps](../../contract/v1/README.md#steps) | The three step kinds and their limits |
| [Commands](../../contract/v1/README.md#commands-1) | The five Core commands, form and plan shapes, cleanup on failure |

### Examples and cases

| File | Content |
| --- | --- |
| [`project-generators/project-generator.plugin.json`](../../contract/v1/fixtures/project-generators/project-generator.plugin.json) | Three generators: `app` (the `project` section, writing template files conditioned on the build system), `archetype` (running a scaffolding command), `initializr` (downloading an archive from a configurable server) |
| [`project-generators/project-plans.json`](../../contract/v1/fixtures/project-generators/project-plans.json) | Golden answers for 10 `project.*` command cases: form defaults and derived values, a plan writing templates, a runtime that is too old, running a command, downloading an archive, refusing a non-HTTPS server, project name checks |
| [`project-generators/templates/`](../../contract/v1/fixtures/project-generators/templates/) | The `app` template directory: `app/README.md`, `app/dot-gitignore`, `main/Main.java` (a Java example); `.gitattributes` disables line-ending conversion so templates are emitted byte for byte |

The shared fixtures have no manifest refusal cases for generators; plan-level refusals (a runtime that is too old, a non-HTTPS server, a name with a slash) are in `project-plans.json`.
