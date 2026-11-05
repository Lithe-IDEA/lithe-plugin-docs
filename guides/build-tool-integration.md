# 为语言插件接入构建、运行与测试 / Adding build, run, and test to a language plugin

[中文](#中文) · [English](#english)

## 中文

LSP 服务器负责语言分析，构建工具负责项目导入、编译、依赖解析与产物。
一个实用的语言插件应把这两条链路分别验证，再验证它们在保存、切换项目和停止时能配合工作。
先完成[LSP 插件生命周期教程](lsp-plugin-lifecycle.md)，再按本篇增加执行能力。
本篇的可操作示例以 macOS 宿主为范围。Windows 宿主的能力表不声明 `executionProviders`、
`buildSystems` 和 `buildProviders`，构建插件在 Windows 上安装前即被拒绝；
不能凭 Core 校验通过推断 Windows 已有相同的运行/测试能力。

### 1. 先选择当前能发布的接入方式

API 3 的构建系统（`buildSystems`）和构建提供者（`buildProviders`）已经在 Core 和 macOS 宿主中实现，
Windows 不支持。各能力的版本和宿主对照见[构建系统](../reference/domains/build-systems.md)、
[构建提供者](../reference/domains/build-providers.md)和[版本对照](../reference/capabilities-and-versions.md)。
发布前仍要核实目标宿主版本支持你声明的能力版本。

| 需求 | 当前接入方式 | 能做什么、边界在哪里 |
| --- | --- | --- |
| 补全、跳转、诊断 | API 1/2 `languageServers` | 按 LSP 提供能力，不自动声明编译和测试 |
| 运行文件或固定项目命令 | `executionProviders[].run` | 命令与 argv 模板，工作目录为工作区根；不提供通用目标选择或构建模型 |
| 运行整个工程、文件所在范围或指定测试 | `executionProviders[].test` | 测试项目标记、文件发现和命令规划；不是完整语义测试发现与结构化结果协议 |
| 官方 Java 项目的运行/测试/调试 | 官方 Java 插件的 API 2 `hostIntegration: jdt` | 宿主已有 JDT 集成，不能复制为社区通用扩展点 |
| 通用 Build 工具窗口、工程模型、任务、依赖与同步 | API 3 `buildSystems`；模型需要程序计算时再加 `buildProviders` | Core 和 macOS 已实现；官方 Gradle、Maven、Cargo 插件已按 API 3 发布；Windows 在安装前拒绝构建插件 |

当需求只是 `lua <当前文件>`、`go test ./...` 这类固定命令，可以使用现有执行提供者。
当需求需要 Maven/Gradle/Cargo 的模块、profile、target、classpath、依赖树或任务动态发现，
应使用 API 3 构建系统和成熟构建后端给出的完整模型；不要把一个 `run` 命令包装成已经完成工程集成。

### 2. 给 Lua 模板加上运行能力

在生命周期教程的最小清单中，替换下面三个顶层字段，其他字段保留：

```json
{
  "permissions": ["languageServer.launch", "process.run"],
  "activationEvents": ["onLanguage:lua", "onRun:lua"],
  "executionProviders": [
    {
      "languageID": "lua",
      "run": {
        "executable": { "command": "lua" },
        "arguments": ["${file}", "${userArguments}"]
      }
    }
  ]
}
```

这也是[Lua 模板](https://github.com/Lithe-IDEA/lithe-plugin-lua)实际采用的方式。
用户需要另行安装 Lua 解释器；随包提供的 LuaLS 不等于解释器。
`process.run` 声明运行权限，`onRun:lua` 声明按运行操作激活，二者都不能省略。
测试声明同理需要 `process.test` 和 `onTest:<languageID>`。

`executable` 只能选一个：裸 `command` 交给宿主工具路径发现，或使用宿主已经支持的
项目 `toolchain` ID，例如 Go 的 `project-go`。插件不能发明一个 toolchain ID 就指望宿主实现。

所有参数都是独立 argv 元素，不经过 shell：

| 模板 | 含义与范围 |
| --- | --- |
| `${file}` | 工作区相对路径；以 `-` 开头时宿主加 `./`，适用于运行、文件测试和用例测试 |
| `${fileDirectory}` | 文件所在目录，根目录为 `.`，其他为 `./…` |
| `${userArguments}` | 仅用于运行，必须独占一个元素，展开为用户参数数组 |
| `${testName}` | 指定测试用例名，去除首尾空白 |
| `${testName:regex}` | 将用例名的正则特殊字符转义，适合测试筛选参数 |

不要在数组里写 `"lua ${file} && echo done"`、额外 shell 引号或依赖变量替换；
那只是一个普通参数。路径中的空格也不需要作者再加引号。执行提供者没有可自行
设置的 `cwd`、`${projectDir}` 或通用构建设置字段，工作目录固定是工作区根。

### 3. 用 Go 说明三个测试范围

下面是一个 **Go 语言插件**的顶层字段片段。它来自现有 Go 提供者的命令结构；
需要与声明 `go` 语言及其服务器的清单合并，不能直接粘贴到 Lua 插件中使用。

```json
{
  "permissions": ["languageServer.launch", "process.run", "process.test"],
  "activationEvents": ["onLanguage:go", "onRun:go", "onTest:go"],
  "executionProviders": [
    {
      "languageID": "go",
      "run": {
        "executable": { "toolchain": "project-go" },
        "arguments": ["run", "${file}", "${userArguments}"]
      },
      "test": {
        "frameworkID": "go",
        "workspaceLabel": "All Go Tests",
        "projectMarkers": ["go.mod", "go.work"],
        "testFilePatterns": ["*_test.go"],
        "executable": { "toolchain": "project-go" },
        "workspaceArguments": ["test", "./..."],
        "fileArguments": ["test", "${fileDirectory}"],
        "testCaseArguments": [
          "test",
          { "value": "${fileDirectory}", "when": "fileSelected" },
          { "value": "./...", "when": "noFileSelected" },
          "-run",
          "^${testName:regex}$"
        ]
      }
    }
  ]
}
```

- 工作区有任一 `projectMarkers` 才被识别为测试项目。`testFilePatterns` 是忽略大小写、
  只支持 `*` 的文件名模式，不是正则或任意 glob。
- 工作区范围运行 `go test ./...`；“文件”范围运行文件所在的包，符合 Go 的测试单位，
  不承诺只执行该文件。
- 用例范围有选中文件时限定包，没有文件时使用 `./...`；引用 `${file}` 或
  `${fileDirectory}` 的用例参数必须用 `when: "fileSelected"` 保护。
- 声明式发现返回工作区和匹配文件，不会仅凭上述字段解析任意框架的测试函数、
  测试结果树或调试入口。要得到这些能力，需要 API 3 构建系统和构建提供者。

这里的 `go run <文件>` 也只运行选中文件，不能替代需要多个源码文件的包运行。
需求超出模板能力时，应如实说明边界，或改用第 5 节的构建系统。

### 4. 将项目事实留给成熟后端

先检查你选用的语言服务器或项目后端是否已经支持工程导入、构建命令和运行目标。
例如 Java 的模块、classpath、测试/调试目标应使用相应 Java 后端给出的事实，
不要由插件另读构建文件再拼一套容易漂移的 classpath。其他语言也应复用其成熟工具链。

一个完整的工程工作流需要确认以下顺序：

```text
保存文档 → 等待后端接收并同步工程变更 → 构建目标 → 检查构建结果
        → 取得本次构建后的运行路径/产物 → 启动所选入口
```

构建失败时不能继续运行旧产物；项目切换后不能用旧模型启动新项目；取消后不能把迟到结果
覆盖到新一次操作。上述职责需要真实后端与宿主配合，声明一条命令并不会自动完成。

构建工具正常解析项目依赖时，可能需要访问网络并写用户的工具缓存；这与启动插件时临时下载
语言服务器不同。语言服务器和随包运行时应在打包阶段准备好。项目依赖、构建产物、
Gradle daemon 状态、Maven 仓库或 Cargo target 等应由对应工具放到其用户缓存或工作区，
不能写入插件包或应用安装目录。每个后台进程都要有明确的停止与清理策略。

### 5. 需要完整工程模型时：API 3 构建系统

`buildSystems` 用数据描述一个项目级构建工具：识别、可执行文件、选项、模型、任务、运行入口和同步。
Core 校验声明、检测实例、解码模型、展开命令行；macOS 宿主执行这些计划，并显示 Build 工具窗口。
模型只能由程序计算时（例如 Gradle Tooling API），在 `buildSystems[].provider` 声明一个
[构建提供者](../reference/domains/build-providers.md)，用 `litheBuild` 协议回答模型、任务、测试和运行准备请求。
字段和版本见[构建系统](../reference/domains/build-systems.md)；
运行原生程序、调试准备和终端见[原生构建工作流](native-build-workflows.md)。

可以直接参考的已发布插件：

| 插件 | 做法 |
| --- | --- |
| [Gradle](https://github.com/Lithe-IDEA/lithe-plugin-gradle) | 纯声明式构建系统，不需要后端 |
| [Maven](https://github.com/Lithe-IDEA/lithe-plugin-maven) | 用 Java SDK 写的构建后端 |
| [Cargo](https://github.com/Lithe-IDEA/lithe-plugin-cargo) | 用 Node.js SDK 写的构建后端和插件设置 |

设计自己的构建系统时，先用下列问题检查。它们是设计检查项，不能替代产品验收：

| 模型部分 | 需要回答的问题 |
| --- | --- |
| 检测 | 如何发现根工程、嵌套模块、wrapper，如何排除生成目录？ |
| 工具与运行时 | wrapper/系统命令如何选择，所需 JDK/其他运行时如何发现和验证？ |
| 选项 | profile、target、模式、平台等如何进入明确的参数与环境，而不是拼接 shell？ |
| 工程模型 | 模块、目标、依赖和运行入口来自哪个成熟后端，如何解码与校验？ |
| 任务 | 支持哪些范围、是否需要用户参数、是否执行项目代码或访问网络？ |
| 同步 | 哪些配置变更触发重导入，如何取消旧请求并拒绝过期结果？ |
| 测试与诊断 | 发现、执行、结果映射由谁提供，路径如何对应用户源码？ |
| 生命周期 | 谁启动进程、监督超时、终止任务并清理状态？ |

Core 的计划接口本身不启动进程、不监视文件、不持久化选择，也不会显示界面；
执行、权限/确认、取消、状态、结果和工具窗口都由宿主负责。fixture 能生成正确命令，
只说明声明和规划正确，仍要在真实宿主里验收。
不要把内部 dispatcher 命令写成社区插件可以调用的宿主脚本 API。

`languageServers[].hostIntegration` 的 JDT 集成（API 2）与构建系统 `model.source: "hostIntegration"`
是不同接口，都只限官方插件：语言服务器侧只有 `jdt`，构建侧目前只有 `maven`。
它们都不是社区作者任意命名即可扩展的机制；不要复制官方 ID 或修改 `vendor` 来绕过限制。
社区构建插件用 `static`、`command` 或 `provider` 模型。

### 6. 验证完整工作流

打包与清单校验完成后，还应在目标平台、实际宿主中完成：

1. 分别准备单文件、多文件、嵌套工程和带空格路径的项目，明确插件支持的范围。
2. 保存修改后运行/测试，确认使用当前文件和目标；将源码改错，确认错误能被看到，
   完整构建集成应阻止继续启动旧产物。
3. 覆盖工作区、文件和指定用例范围，核对实际命令、工作目录、输出与退出码。
4. 缺少解释器、工具链版本不符、wrapper 无执行权限或依赖下载失败时，错误能定位到原因。
5. 取消耗时任务，随后再运行；切换项目与关闭工作区，确认旧任务、锁与后端进程没有残留。
   若宿主或工具尚不能做到，应报告限制，不能只在文档里宣称已经支持。
6. 禁用/卸载语言插件后检查该语言的运行、测试与 LSP 都处于正确状态。
   插件声明的语言由插件负责，不能假定缺失能力会回退到宿主内置实现。
7. 清理验收启动的应用和辅助进程，再按[LSP 生命周期检查表](lsp-plugin-lifecycle.md#7-把停止失败和卸载也当作功能验收)
   验证重启、升级和卸载。

命令规划测试证明参数正确，进程冒烟验证证明工具能够启动，真实项目验收才能证明用户工作流
成立。发布说明应明确已经验证的平台、宿主、语言服务器及构建工具版本。

相关入口：[生命周期教程](lsp-plugin-lifecycle.md) · [发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md) ·
[版本追踪](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md) ·
[Lithe 插件契约](../contract/v1/README.md)。

---

## English

A language server handles language analysis; a build tool handles project import, compilation,
dependency resolution, and artifacts. A useful language plugin verifies each chain on its own,
then verifies that they work together on save, on project switch, and on stop.
Finish the [LSP plugin lifecycle tutorial](lsp-plugin-lifecycle.md) first, then add execution with this guide.
The runnable examples here target the macOS host. The Windows host's capability table does not
advertise `executionProviders`, `buildSystems`, or `buildProviders`, and Windows rejects build
plugins before install; passing Core validation does not mean Windows has the same run and test support.

### 1. Pick an integration you can publish today

API 3 build systems (`buildSystems`) and build providers (`buildProviders`) are implemented in
Core and the macOS host; Windows does not support them. For the versions of each capability
and how the hosts compare, see [build systems](../reference/domains/build-systems.md),
[build providers](../reference/domains/build-providers.md), and the
[version tables](../reference/capabilities-and-versions.md).
Before publishing, still confirm that the target host version supports every capability version you declare.

| Need | Integration | What it does and where it stops |
| --- | --- | --- |
| Completion, navigation, diagnostics | API 1/2 `languageServers` | Features come from LSP; compile and test are not declared automatically |
| Run a file or a fixed project command | `executionProviders[].run` | A command and argv template; the working directory is the workspace root; no general target selection or build model |
| Run the whole project, a file's scope, or a named test | `executionProviders[].test` | Test-project markers, file discovery, and command planning; not a full semantic test-discovery or structured-results protocol |
| Run, test, and debug official Java projects | The official Java plugin's API 2 `hostIntegration: jdt` | The host already has the JDT integration; it cannot be copied as a general community extension point |
| A general Build tool window, project model, tasks, dependencies, and sync | API 3 `buildSystems`, plus `buildProviders` when a program must compute the model | Implemented in Core and macOS; the official Gradle, Maven, and Cargo plugins are published at API 3; Windows rejects build plugins before install |

When all you need is a fixed command such as `lua <current file>` or `go test ./...`, an
execution provider is enough. When you need Maven/Gradle/Cargo modules, profiles, targets,
classpaths, dependency trees, or dynamic task discovery, use an API 3 build system with the
full model a mature build backend reports; do not wrap a single `run` command and call it project integration.

### 2. Add run support to the Lua template

In the lifecycle tutorial's minimal manifest, replace these three top-level fields and keep the rest:

```json
{
  "permissions": ["languageServer.launch", "process.run"],
  "activationEvents": ["onLanguage:lua", "onRun:lua"],
  "executionProviders": [
    {
      "languageID": "lua",
      "run": {
        "executable": { "command": "lua" },
        "arguments": ["${file}", "${userArguments}"]
      }
    }
  ]
}
```

This is exactly what the [Lua template](https://github.com/Lithe-IDEA/lithe-plugin-lua) does.
Users must install a Lua interpreter themselves; the bundled LuaLS is not an interpreter.
`process.run` declares the run permission and `onRun:lua` declares activation on run; neither can be omitted.
A test declaration likewise needs `process.test` and `onTest:<languageID>`.

`executable` is exactly one of two things: a bare `command` that the host finds on its tool path,
or a project `toolchain` ID the host already supports, such as Go's `project-go`.
A plugin cannot invent a toolchain ID and expect the host to implement it.

Every argument is a separate argv element and never goes through a shell:

| Template | Meaning and scope |
| --- | --- |
| `${file}` | Workspace-relative path; the host prefixes `./` when it starts with `-`; valid for run, file tests, and test-case tests |
| `${fileDirectory}` | The file's directory: `.` for the root, `./…` otherwise |
| `${userArguments}` | Run only; must be a whole element on its own and expands to the user's argument array |
| `${testName}` | The selected test-case name, trimmed |
| `${testName:regex}` | The test-case name with regex metacharacters escaped, for test-filter arguments |

Do not write `"lua ${file} && echo done"`, extra shell quotes, or anything that relies on variable
substitution; that is just one ordinary argument. Paths with spaces need no extra quoting either.
Execution providers have no author-settable `cwd`, no `${projectDir}`, and no general build-settings
field; the working directory is always the workspace root.

### 3. Three test scopes, using Go

Below are the top-level fields of a **Go language plugin**. They follow the command structure of
the existing Go provider; merge them into a manifest that declares the `go` language and its server.
They cannot be pasted into the Lua plugin as is.

```json
{
  "permissions": ["languageServer.launch", "process.run", "process.test"],
  "activationEvents": ["onLanguage:go", "onRun:go", "onTest:go"],
  "executionProviders": [
    {
      "languageID": "go",
      "run": {
        "executable": { "toolchain": "project-go" },
        "arguments": ["run", "${file}", "${userArguments}"]
      },
      "test": {
        "frameworkID": "go",
        "workspaceLabel": "All Go Tests",
        "projectMarkers": ["go.mod", "go.work"],
        "testFilePatterns": ["*_test.go"],
        "executable": { "toolchain": "project-go" },
        "workspaceArguments": ["test", "./..."],
        "fileArguments": ["test", "${fileDirectory}"],
        "testCaseArguments": [
          "test",
          { "value": "${fileDirectory}", "when": "fileSelected" },
          { "value": "./...", "when": "noFileSelected" },
          "-run",
          "^${testName:regex}$"
        ]
      }
    }
  ]
}
```

- A workspace is a test project only if it has at least one of the `projectMarkers`.
  `testFilePatterns` are case-insensitive file-name patterns that support only `*`; they are not regexes or arbitrary globs.
- The workspace scope runs `go test ./...`. The file scope runs the file's package, which is Go's
  unit of testing; it does not promise to run only that file.
- The test-case scope narrows to the package when a file is selected and uses `./...` when none is.
  Test-case arguments that reference `${file}` or `${fileDirectory}` must be guarded with `when: "fileSelected"`.
- Declarative discovery returns the workspace and matching files. These fields alone do not parse
  any framework's test functions, result trees, or debug entries; for those you need an API 3 build system and build provider.

`go run <file>` here also runs only the selected file and cannot replace running a package made of several source files.
When a need goes beyond what the templates can express, state the limit honestly, or switch to the build system in section 5.

### 4. Leave project facts to a mature backend

First check whether the language server or project backend you chose already supports project import,
build commands, and run targets. For Java, for example, modules, classpaths, and test/debug targets
should come from what the Java backend reports; do not have the plugin read build files again and
assemble its own classpath that will drift. Other languages should reuse their mature toolchains too.

A complete project workflow must confirm this order:

```text
save document → wait for the backend to receive and sync the project change → build the target
              → check the build result → get the run path/artifact from this build → launch the chosen entry
```

A failed build must not go on to run an old artifact; after a project switch the old model must not
launch the new project; after a cancel, a late result must not overwrite the next operation.
These duties need a real backend working with the host; declaring a command does not do them for you.

When a build tool resolves project dependencies normally, it may access the network and write the
user's tool caches; that is different from downloading a language server when the plugin starts.
Language servers and bundled runtimes must be prepared at packaging time. Project dependencies,
build outputs, Gradle daemon state, the Maven repository, Cargo targets, and so on belong in the
tool's user cache or the workspace, never in the plugin package or the application install directory.
Every background process needs a clear stop and cleanup policy.

### 5. When you need the full project model: API 3 build systems

`buildSystems` describes a project-level build tool as data: detection, executables, options, model,
tasks, run entries, and sync. Core validates the declaration, detects instances, decodes the model,
and expands command lines; the macOS host executes those plans and shows the Build tool window.
When only a program can compute the model (the Gradle Tooling API, for example), declare a
[build provider](../reference/domains/build-providers.md) in `buildSystems[].provider`; it answers
model, task, test, and run-preparation requests over the `litheBuild` protocol.
For fields and versions see [build systems](../reference/domains/build-systems.md);
for running native programs, debug preparation, and terminals see [native build workflows](native-build-workflows.md).

Published plugins you can learn from:

| Plugin | Approach |
| --- | --- |
| [Gradle](https://github.com/Lithe-IDEA/lithe-plugin-gradle) | A purely declarative build system with no backend |
| [Maven](https://github.com/Lithe-IDEA/lithe-plugin-maven) | A build backend written with the Java SDK |
| [Cargo](https://github.com/Lithe-IDEA/lithe-plugin-cargo) | A build backend and plugin settings written with the Node.js SDK |

When designing your own build system, check it against these questions first.
They are a design checklist and do not replace product acceptance:

| Model part | Questions to answer |
| --- | --- |
| Detection | How are root projects, nested modules, and wrappers found, and how are generated directories excluded? |
| Tools and runtimes | How is a wrapper or system command chosen, and how is the required JDK or other runtime found and verified? |
| Options | How do profiles, targets, modes, and platforms become explicit arguments and environment instead of shell concatenation? |
| Project model | Which mature backend supplies modules, targets, dependencies, and run entries, and how are they decoded and validated? |
| Tasks | Which scopes are supported, do they need user arguments, and do they run project code or access the network? |
| Sync | Which configuration changes trigger a re-import, and how are old requests cancelled and stale results rejected? |
| Tests and diagnostics | Who provides discovery, execution, and result mapping, and how do paths map back to the user's sources? |
| Lifecycle | Who starts processes, enforces timeouts, terminates tasks, and cleans up state? |

Core's planning interface does not start processes, watch files, persist selections, or show any UI;
execution, permissions and confirmation, cancellation, status, results, and the tool window are the host's job.
A fixture that produces the right command only shows that the declaration and plan are correct;
you still have to accept it in a real host.
Do not present internal dispatcher commands as a host scripting API that community plugins can call.

The JDT integration in `languageServers[].hostIntegration` (API 2) and the build-system
`model.source: "hostIntegration"` are different interfaces, and both are limited to official plugins:
the language-server side has only `jdt`, and the build side currently has only `maven`.
Neither is a mechanism community authors can extend by picking a name; do not copy an official ID
or change `vendor` to get around the limit. Community build plugins use the `static`, `command`, or `provider` models.

### 6. Verify the complete workflow

After packaging and manifest validation, also do the following on the target platform in a real host:

1. Prepare single-file, multi-file, nested-project, and space-in-path projects, and state which ones the plugin supports.
2. Run or test after saving a change and confirm the current file and target are used; break the source
   and confirm the error is visible. A full build integration must stop an old artifact from launching.
3. Cover the workspace, file, and named-test scopes, and check the actual command, working directory, output, and exit code.
4. When the interpreter is missing, the toolchain version is wrong, the wrapper is not executable,
   or a dependency download fails, the error must point to the cause.
5. Cancel a long task and run again; switch projects and close the workspace, and confirm no old
   tasks, locks, or backend processes remain. If the host or tool cannot do this yet, report the limit
   instead of only claiming support in the docs.
6. After disabling or uninstalling the language plugin, check that run, test, and LSP for that language
   are all in the right state. The plugin owns the languages it declares; do not assume missing
   features fall back to a built-in host implementation.
7. Clean up the apps and helper processes the acceptance run started, then verify restart, upgrade,
   and uninstall with the [LSP lifecycle checklist](lsp-plugin-lifecycle.md#7-treat-stop-failure-and-uninstall-as-acceptance-criteria).

Command-planning tests prove the arguments are right, process smoke tests prove the tool can start,
and only acceptance on real projects proves the user workflow holds. Release notes must state the
platforms, host, language server, and build tool versions you verified.

Related: [lifecycle tutorial](lsp-plugin-lifecycle.md) · [publishing a community plugin](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md) ·
[version tracking](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md) ·
[Lithe plugin contract](../contract/v1/README.md).
