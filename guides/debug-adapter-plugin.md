# 编写调试适配器插件（API 4） / Writing a debug adapter plugin (API 4)

[中文](#中文) · [English](#english)

## 中文

这篇指南面向想让 Lithe 调试某种语言或运行时的插件作者。调试适配器插件只需要一份 `plugin.json`：声明一个遵循
[DAP（Debug Adapter Protocol）](https://microsoft.github.io/debug-adapter-protocol/)的进程怎么启动，Lithe 负责断点、
单步、变量、调用栈、调试控制台这些界面和会话管理。我们以官方的
[LLDB 插件](https://github.com/Lithe-IDEA/lithe-plugin-lldb)为例，它用 Xcode 自带的 `lldb-dap` 调试 C、C++、Objective-C、Rust 和 Swift。

字段的权威定义在[契约](../contract/v1/README.md#api-level-4-debug-adapters)，这里只讲怎么用、为什么这样设计。发布步骤见
[发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)。

> **宿主支持。** 调试适配器是 API 4，需要所用 Lithe 明确支持 API 4。macOS 宿主支持 API 4 和调试适配器；
> Windows 宿主还没有实现调试适配器。各能力版本以按最新 preview 核对的[版本对照](../reference/capabilities-and-versions.md)和[契约](../contract/v1/README.md)为准，稳定版是否已包含看发行说明。

### 1. 插件和宿主各自负责什么

| 对象 | 插件作者负责 | Lithe 负责 |
| --- | --- | --- |
| 调试适配器 | 选型、许可、固定版本、启动入口 | 启动进程、DAP 通信、会话停止 |
| 要调试的程序 | 说明怎么产生它（构建命令、编译参数） | 在设置里显示实际找到的路径，把它交给适配器 |
| 调试界面 | 不需要写 | 断点、变量、调用栈、调试控制台与补全 |
| 权限 | 声明 `debug.launch` | 安装时让用户确认 |
| 激活 | 声明 `onDebug:<适配器 id>` | 调试会话需要时才启动适配器 |

插件不加载进 Lithe 进程。清单是纯数据：没有解析器、没有工作流、没有宿主集成，也不能传 `initializationOptions`，写了会被拒绝。

### 2. 最小清单

```json
{
  "schemaVersion": 2,
  "id": "com.example.demo-debug",
  "displayName": "Demo Debugger",
  "version": "0.1.0",
  "pluginAPIVersion": 4,
  "requires": { "debugAdapters": 1, "languages": 1, "serverProcess": 1 },
  "vendor": { "id": "com.example", "displayName": "Example", "signatureRequirement": "none" },
  "permissions": ["debug.launch"],
  "activationEvents": ["onDebug:demo"],
  "languages": [
    { "id": "demo", "displayName": "Demo", "fileExtensions": ["demo"] }
  ],
  "debugAdapters": [
    {
      "id": "demo",
      "displayName": "Demo",
      "requests": ["launch"],
      "selectors": [{ "targetKinds": ["file"], "languageIDs": ["demo"] }],
      "configurations": {
        "launch": {
          "arguments": { "stopOnEntry": false },
          "targetBindings": { "program": "target.path", "cwd": "target.workingDirectory" }
        }
      },
      "launch": {
        "process": { "launch": { "kind": "systemCommand", "commands": ["demo-dap"] } },
        "transport": "stdio"
      }
    }
  ]
}
```

要点：

- `pluginAPIVersion` 是 `4`，`requires` 必须逐项列出实际用到的能力（`debugAdapters` 为 `1`）。
- 每个适配器都要有一条 `onDebug:<id>` 激活事件，并声明 `debug.launch` 权限。适配器列出 `attach` 时才需要（也只能声明）`debug.attach`。
- `selectors` 决定 Lithe 在哪些目标上自动选这个适配器：`targetKinds` 是目标种类，`languageIDs` 是语言。不写 `selectors` 表示只能被用户显式选择。
- `languages[]` 声明适配器服务的语言。如果这个语言已经由别的插件提供（例如 Rust），你的声明只会给它增加“可以调试”的能力，并并入你声明的文件扩展名；语言名、语言标识和语言服务器不变，装上调试器也不会重启正在运行的语言服务器。
- `targetBindings` 把调试目标的事实填进适配器的启动参数：`program` ← 目标路径，`cwd` ← 工作目录，`processId` ← 进程 ID。只填 `arguments` 里没写的字段，不会覆盖你显式写出的默认值。目标缺少绑定要读的事实时，Lithe 会拒绝这次启动并说明缺什么。
- `configurationTypes` 声明适配器对应的 `launch.json` `type`，用于导入用户已有的 VS Code 风格配置。

### 3. 怎么启动适配器

`launch.process` 有三种常见写法：

| 写法 | 适用场景 |
| --- | --- |
| `{"launch": {"kind": "packaged", "path": …}}` | 适配器许可允许分发、你能为每个平台提供构建；可执行文件随插件包安装 |
| `{"launch": {"kind": "systemCommand", "commands": ["lldb-dap"]}}` | 适配器已经装在用户机器上（例如随 Xcode 命令行工具提供）；Lithe 用自己的工具发现（PATH、项目工具链、用户覆盖）解析命令名，macOS 上也覆盖 Xcode 工具链 |
| `{"launch": {"kind": "runSelectedRuntime", "tool": "python"}}` | 适配器是 Python / Java / Node.js 程序，用用户选定的运行时启动；这是 `serverProcess` 2 唯一的用途 |

`launch.transport` 是 `stdio`，或者 TCP（`tcpAnnounced`：适配器在输出里打印端口；`tcpPort`：固定端口）。TCP 只允许回环地址。

插件包和 Lithe 的安装目录运行时是只读的，适配器不能往里面写日志、索引或状态，否则会破坏代码签名和增量更新。macOS 上宿主把适配器的状态目录放在 Application Support 的 `debug-adapters/<适配器 id>` 下。

### 4. 调试要先编译的语言：`executable`

DAP 适配器大多直接接受“要运行的程序”，但 C、C++、Swift 这类语言的用户打开的是**源文件**，真正要调试的是编译产物。
`configurations.launch.executable` 就是为这种情况设计的：

```json
"configurations": {
  "launch": {
    "targetBindings": { "program": "target.executable", "cwd": "target.workingDirectory" },
    "executable": {
      "setting": "program",
      "candidates": [
        "${fileDirectory}/${fileBasenameNoExtension}",
        "${workspaceFolder}/${fileBasenameNoExtension}",
        "${workspaceFolder}/build/${fileBasenameNoExtension}"
      ]
    }
  }
}
```

- `setting` 是你在 `settings[]` 里声明的一个 `file` 类型设置的 id。用户在设置页里选了路径，就用它。
- 用户没选时，Lithe 依次查找 `candidates`（最多 8 个，每个必须以 `${workspaceFolder}/` 或 `${fileDirectory}/` 开头，可以用 `${fileBasenameNoExtension}`，不能含 `..`），用找到的第一个可运行文件。
- 找到的路径会**回显在这个设置里**，用户能看到实际会调试哪个文件。
- 用户选的路径如果不是可运行文件，是错误，**不会**悄悄回落到候选路径。
- `executable` 和 `targetBindings.program: "target.executable"` 必须一起声明，缺一不可。

设置的 `description` 是教用户怎么产生这个程序的地方，写清楚构建命令（例如 `clang -g main.c -o main`）和适配器本身的安装方式。LLDB 插件的 `program` 设置就是这样写的。设置的写法见契约的 `settings` 一节，`scope: "project"`、`effect: "nextRun"` 适合这类“下次运行生效”的路径。

### 5. 调试控制台：提示符、补全和 `console`

Lithe 的调试控制台有输入行和补全弹窗。补全来自适配器的 DAP `completions` 请求；`console` 补充两件事：

- `commandPrefix`：1 到 8 个字符、不含空格。有些适配器把“光秃秃的单词”当表达式（lldb-dap 就是这样），补全第一个词时会混进 `long` 之类的语言关键字。声明前缀（lldb-dap 用反引号 `` ` ``）后，Lithe 只在向适配器请求补全时把它加在输入前面，并把适配器的答案移回去，用户看不到它。
- `commands[]`：适配器不支持 `completions`，或者它的实时答案缺一些命令时，列出调试器命令（`name`，可选的一行 `description`）。名字按原样插入，不能重复，不要在名字里补空格；说明按原样显示，不翻译。

两者至少声明一个。适配器能实时补全时，不要把整个命令集再抄一遍。

### 6. 本地验证

1. 对照契约示例 [`contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json`](../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json)，并用[编辑器 Schema](../README.md#在编辑器里校验-pluginjson) 检查字段。
2. [`contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json`](../contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json) 列出每种拒绝的稳定错误码、出错字段和报告它的层（`schema`、`coreManifest`、`corePlan`、`coreInstall`），写测试时可以直接对照。
3. 在支持 API 4 的 Lithe 里安装插件，打开一个目标文件，设置断点并启动调试。适配器被拒绝请求时，Lithe 会显示适配器自己的解释，所以适配器返回的错误文字要写得让用户能看懂。
4. 在设置页确认 `executable` 找到的路径被回显，再把它改成一个不存在的文件，确认得到的是错误而不是悄悄换文件。

### 7. 安装后是否需要重启

只贡献调试适配器（以及它声明的语言和设置）的插件，在支持热激活的 Lithe 里安装、更新、回退、卸载、启用、停用后**不需要重启**。是否免重启由宿主按它实际能在运行期登记和撤回的贡献类型决定，不是 `plugin.json` 里的字段，所以插件作者不需要也不能声明。带有其他贡献（语言服务器、构建系统、界面等）的插件仍然在重启后生效。

### 8. 发布

调试适配器插件需要 API 4，只有支持 API 4 的 Lithe 才能安装。旧版 Lithe 在插件市场里会提示升级，而不是安装。

- 插件声明的能力版本，或所用 SDK 要求的协议（例如 0.3.0 起的 SDK 要求宿主提供 `litheLogging` 1），还没进入 Lithe 稳定版时，把 GitHub Release 标为**预发布**。Lithe Store 的自动追踪只收正式 Release，预发布需要提交一个手工的索引条目，写法参考 Maven、Gradle 和 LLDB 插件的 0.1.0。
- 稳定版支持插件需要的全部能力和协议后，把 Release 转为正式版，追踪器会自动提交后续版本的索引更新。
- 发布后不要替换或删除资产；索引固定了它的 SHA-256。

### 9. 当前边界

- 只有 macOS 宿主实现了调试适配器。
- 宿主的调试入口对插件适配器只发起 `launch` 请求。`attach` 可以在清单里声明，目前只在构建提供者服务运行的连接调试（`buildProviders` 11）中由宿主选用；用户手动填写主机和端口的远程 attach 还没有宿主入口，不要依赖。
- 适配器声明是纯数据，没有自定义界面、没有宿主集成点；需要调试之外的能力（构建、测试、运行配置）请另用对应的贡献类型。

---

## English

This guide is for plugin authors who want Lithe to debug a language or runtime. A debug adapter plugin needs only a `plugin.json`: it declares how to start a process that speaks the
[Debug Adapter Protocol (DAP)](https://microsoft.github.io/debug-adapter-protocol/), and Lithe provides the breakpoint, stepping, variable, call stack, and debug console UI and manages the session. The running example is the official
[LLDB plugin](https://github.com/Lithe-IDEA/lithe-plugin-lldb), which debugs C, C++, Objective-C, Rust, and Swift with the `lldb-dap` that ships with Xcode.

The normative field definitions are in the [contract](../contract/v1/README.md#api-level-4-debug-adapters); this guide covers only how to use them and why they are designed this way. For publishing steps, see
[Publishing community plugins](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md).

> **Host support.** Debug adapters are API 4 and need a Lithe that explicitly supports API 4. The macOS host supports API 4 and debug adapters;
> the Windows host does not implement debug adapters yet. For each capability version, go by the [version tables](../reference/capabilities-and-versions.md), which track the latest preview, and the [contract](../contract/v1/README.md); the release notes say whether a stable release includes it.

### 1. What the plugin and the host each own

| Item | The plugin author owns | Lithe owns |
| --- | --- | --- |
| Debug adapter | Choosing it, its license, the pinned version, the launch entry | Starting the process, DAP communication, stopping the session |
| Program to debug | Explaining how it is produced (build command, compiler flags) | Showing the path actually found in settings and passing it to the adapter |
| Debugging UI | Nothing to write | Breakpoints, variables, call stack, debug console and completion |
| Permissions | Declaring `debug.launch` | Asking the user to confirm at install time |
| Activation | Declaring `onDebug:<adapter id>` | Starting the adapter only when a debug session needs it |

Plugins are not loaded into the Lithe process. The manifest is pure data: no resolvers, no workflows, no host integrations, and no `initializationOptions`; writing one is rejected.

### 2. A minimal manifest

```json
{
  "schemaVersion": 2,
  "id": "com.example.demo-debug",
  "displayName": "Demo Debugger",
  "version": "0.1.0",
  "pluginAPIVersion": 4,
  "requires": { "debugAdapters": 1, "languages": 1, "serverProcess": 1 },
  "vendor": { "id": "com.example", "displayName": "Example", "signatureRequirement": "none" },
  "permissions": ["debug.launch"],
  "activationEvents": ["onDebug:demo"],
  "languages": [
    { "id": "demo", "displayName": "Demo", "fileExtensions": ["demo"] }
  ],
  "debugAdapters": [
    {
      "id": "demo",
      "displayName": "Demo",
      "requests": ["launch"],
      "selectors": [{ "targetKinds": ["file"], "languageIDs": ["demo"] }],
      "configurations": {
        "launch": {
          "arguments": { "stopOnEntry": false },
          "targetBindings": { "program": "target.path", "cwd": "target.workingDirectory" }
        }
      },
      "launch": {
        "process": { "launch": { "kind": "systemCommand", "commands": ["demo-dap"] } },
        "transport": "stdio"
      }
    }
  ]
}
```

Key points:

- `pluginAPIVersion` is `4`, and `requires` must list exactly the capabilities you use (`debugAdapters` is `1`).
- Every adapter needs an `onDebug:<id>` activation event and the `debug.launch` permission. `debug.attach` is needed (and allowed) only when the adapter lists `attach`.
- `selectors` decide on which targets Lithe picks this adapter automatically: `targetKinds` are target kinds and `languageIDs` are languages. Without `selectors`, the adapter can only be chosen explicitly by the user.
- `languages[]` declares the languages the adapter serves. If another plugin already provides the language (for example Rust), your declaration only adds the ability to debug it and merges in the file extensions you declare; the language name, language ID, and language server stay the same, and installing the debugger does not restart a running language server.
- `targetBindings` fills facts about the debug target into the adapter's launch arguments: `program` ← the target path, `cwd` ← the working directory, `processId` ← the process ID. Only fields missing from `arguments` are filled; defaults you wrote explicitly are never overwritten. When the target lacks a fact a binding reads, Lithe refuses the launch and says what is missing.
- `configurationTypes` declares the `launch.json` `type` values the adapter matches, used to import the user's existing VS Code-style configurations.

### 3. Launching the adapter

`launch.process` has three common forms:

| Form | When to use it |
| --- | --- |
| `{"launch": {"kind": "packaged", "path": …}}` | The adapter's license allows distribution and you can provide a build for each platform; the executable installs with the plugin package |
| `{"launch": {"kind": "systemCommand", "commands": ["lldb-dap"]}}` | The adapter is already installed on the user's machine (for example with the Xcode command line tools); Lithe resolves the command name with its own tool discovery (PATH, project toolchains, user overrides), which on macOS also covers the Xcode toolchain |
| `{"launch": {"kind": "runSelectedRuntime", "tool": "python"}}` | The adapter is a Python, Java, or Node.js program started with the runtime the user selected; this is the only use of `serverProcess` 2 |

`launch.transport` is `stdio` or TCP (`tcpAnnounced`: the adapter prints its port in its output; `tcpPort`: a fixed port). TCP allows only loopback addresses.

The plugin package and the Lithe install directory are read-only at run time. An adapter must not write logs, indexes, or state there, or it breaks code signing and delta updates. On macOS the host puts the adapter's state directory under `debug-adapters/<adapter id>` in Application Support.

### 4. Languages that compile before debugging: `executable`

Most DAP adapters take "the program to run" directly, but in languages such as C, C++, and Swift the user opens a **source file** while the thing to debug is the compiled output.
`configurations.launch.executable` is designed for this case:

```json
"configurations": {
  "launch": {
    "targetBindings": { "program": "target.executable", "cwd": "target.workingDirectory" },
    "executable": {
      "setting": "program",
      "candidates": [
        "${fileDirectory}/${fileBasenameNoExtension}",
        "${workspaceFolder}/${fileBasenameNoExtension}",
        "${workspaceFolder}/build/${fileBasenameNoExtension}"
      ]
    }
  }
}
```

- `setting` is the ID of a `file` setting you declare in `settings[]`. When the user picks a path on the settings page, that path is used.
- When the user has picked nothing, Lithe tries `candidates` in order (at most 8; each must start with `${workspaceFolder}/` or `${fileDirectory}/`, may use `${fileBasenameNoExtension}`, and must not contain `..`) and uses the first runnable file it finds.
- The path it finds is **shown back in that setting**, so the user can see which file will actually be debugged.
- A user-picked path that is not a runnable file is an error; Lithe does **not** quietly fall back to the candidates.
- `executable` and `targetBindings.program: "target.executable"` must be declared together.

The setting's `description` is where you teach the user how to produce the program: give the build command (for example `clang -g main.c -o main`) and how to install the adapter itself. The LLDB plugin's `program` setting is written this way. For writing settings, see the contract's `settings` section; `scope: "project"` and `effect: "nextRun"` suit a path like this that takes effect on the next run.

### 5. The debug console: prompt, completion, and `console`

Lithe's debug console has an input line and a completion popup. Completions come from the adapter's DAP `completions` request; `console` adds two things:

- `commandPrefix`: 1 to 8 characters with no spaces. Some adapters treat a bare word as an expression (lldb-dap does), so completing the first word mixes in language keywords such as `long`. With a declared prefix (lldb-dap uses a backtick `` ` ``), Lithe adds it in front of the input only when asking the adapter for completions and moves the adapter's answers back, so the user never sees it.
- `commands[]`: when the adapter does not support `completions`, or its live answers miss some commands, list the debugger commands (`name`, with an optional one-line `description`). Names are inserted as written, must be unique, and must not have padding spaces; descriptions are shown as written and are not translated.

Declare at least one of the two. When the adapter completes live, do not copy its whole command set as well.

### 6. Local verification

1. Compare with the contract example [`contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json`](../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) and check fields with the [editor schema](../README.md#validate-pluginjson-in-your-editor).
2. [`contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json`](../contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json) lists, for each rejection, the stable error code, the offending field, and the layer that reports it (`schema`, `coreManifest`, `corePlan`, `coreInstall`); compare against it directly when writing tests.
3. Install the plugin in a Lithe that supports API 4, open a target file, set a breakpoint, and start debugging. When the adapter rejects a request, Lithe shows the adapter's own explanation, so write the adapter's error text for users to understand.
4. On the settings page, confirm that the path `executable` found is shown, then change it to a file that does not exist and confirm you get an error rather than a quietly swapped file.

### 7. Whether installing needs a restart

A plugin that contributes only debug adapters (and the languages and settings it declares) needs **no restart** after install, update, rollback, uninstall, enable, or disable in a Lithe that supports hot activation. The host decides this from the contribution types it can actually register and withdraw at run time; it is not a `plugin.json` field, so plugin authors neither need to nor can declare it. Plugins with other contributions (language servers, build systems, UI, and so on) still take effect after a restart.

### 8. Publishing

A debug adapter plugin needs API 4 and installs only on a Lithe that supports API 4. Older Lithe versions show an update prompt in the marketplace instead of installing.

- Mark your GitHub Release as a **prerelease** while a capability version the plugin declares, or a protocol its SDK requires (for example, SDKs from 0.3.0 require the host to offer `litheLogging` 1), is not yet in a stable Lithe release. Lithe Store's automatic tracking only picks up full Releases, so a prerelease needs a hand-written index entry; follow the 0.1.0 entries of the Maven, Gradle, and LLDB plugins.
- Once a stable release supports every capability and protocol the plugin needs, convert the Release to a full release, and the tracker submits index updates for later versions automatically.
- Do not replace or delete assets after publishing; the index pins their SHA-256.

### 9. Current limits

- Only the macOS host implements debug adapters.
- The host's debug entry issues only `launch` requests to plugin adapters. `attach` can be declared in the manifest; today the host chooses it only for connection debugging of build-provider service runs (`buildProviders` 11). Remote attach with a user-entered host and port has no host entry yet, so do not rely on it.
- An adapter declaration is pure data, with no custom UI and no host integration points; for capabilities beyond debugging (build, test, run configurations), use the matching contribution types.
