# 编写调试适配器插件（API 4）

这篇指南面向想让 Lithe 调试某种语言或运行时的插件作者。调试适配器插件只需要一份 `plugin.json`：声明一个遵循
[DAP（Debug Adapter Protocol）](https://microsoft.github.io/debug-adapter-protocol/)的进程怎么启动，Lithe 负责断点、
单步、变量、调用栈、调试控制台这些界面和会话管理。我们以官方的
[LLDB 插件](https://github.com/Lithe-IDEA/lithe-plugin-lldb)为例，它用 Xcode 自带的 `lldb-dap` 调试 C、C++、Objective-C、Rust 和 Swift。

字段的权威定义在[契约](../contract/v1/README.md#api-level-4-debug-adapters)，这里只讲怎么用、为什么这样设计。发布步骤见
[发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)。

> **宿主支持。** 调试适配器是 API 4，需要所用 Lithe 明确支持 API 4。目前只有 macOS 宿主实现了调试适配器；
> Windows 宿主还没有。不要只凭应用版本号推断支持情况，以[契约](../contract/v1/README.md)和发行说明为准。

## 1. 插件和宿主各自负责什么

| 对象 | 插件作者负责 | Lithe 负责 |
| --- | --- | --- |
| 调试适配器 | 选型、许可、固定版本、启动入口 | 启动进程、DAP 通信、会话停止 |
| 要调试的程序 | 说明怎么产生它（构建命令、编译参数） | 在设置里显示实际找到的路径，把它交给适配器 |
| 调试界面 | 不需要写 | 断点、变量、调用栈、调试控制台与补全 |
| 权限 | 声明 `debug.launch` | 安装时让用户确认 |
| 激活 | 声明 `onDebug:<适配器 id>` | 调试会话需要时才启动适配器 |

插件不加载进 Lithe 进程。清单是纯数据：没有解析器、没有工作流、没有宿主集成，也不能传 `initializationOptions`，写了会被拒绝。

## 2. 最小清单

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

## 3. 怎么启动适配器

`launch.process` 有三种常见写法：

| 写法 | 适用场景 |
| --- | --- |
| `{"launch": {"kind": "packaged", "path": …}}` | 适配器许可允许分发、你能为每个平台提供构建；可执行文件随插件包安装 |
| `{"launch": {"kind": "systemCommand", "commands": ["lldb-dap"]}}` | 适配器已经装在用户机器上（例如随 Xcode 命令行工具提供）；Lithe 用自己的工具发现（PATH、项目工具链、用户覆盖）解析命令名，macOS 上也覆盖 Xcode 工具链 |
| `{"launch": {"kind": "runSelectedRuntime", "tool": "python"}}` | 适配器是 Python / Java / Node.js 程序，用用户选定的运行时启动；这是 `serverProcess` 2 唯一的用途 |

`launch.transport` 是 `stdio`，或者 TCP（`tcpAnnounced`：适配器在输出里打印端口；`tcpPort`：固定端口）。TCP 只允许回环地址。

插件包和 Lithe 的安装目录运行时是只读的，适配器不能往里面写日志、索引或状态，否则会破坏代码签名和增量更新。macOS 上宿主把适配器的状态目录放在 Application Support 的 `debug-adapters/<适配器 id>` 下。

## 4. 调试要先编译的语言：`executable`

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

## 5. 调试控制台：提示符、补全和 `console`

Lithe 的调试控制台有输入行和补全弹窗。补全来自适配器的 DAP `completions` 请求；`console` 补充两件事：

- `commandPrefix`：1 到 8 个字符、不含空格。有些适配器把“光秃秃的单词”当表达式（lldb-dap 就是这样），补全第一个词时会混进 `long` 之类的语言关键字。声明前缀（lldb-dap 用反引号 `` ` ``）后，Lithe 只在向适配器请求补全时把它加在输入前面，并把适配器的答案移回去，用户看不到它。
- `commands[]`：适配器不支持 `completions`，或者它的实时答案缺一些命令时，列出调试器命令（`name`，可选的一行 `description`）。名字按原样插入，不能重复，不要在名字里补空格；说明按原样显示，不翻译。

两者至少声明一个。适配器能实时补全时，不要把整个命令集再抄一遍。

## 6. 本地验证

1. 对照契约示例 `contract/v1/fixtures/debug-adapter.plugin.json`，并用[编辑器 Schema](../README.md#在编辑器里校验-pluginjson) 检查字段。
2. `contract/v1/fixtures/debug-adapter-rejections.json` 列出每种拒绝的稳定错误码、出错字段和报告它的层（`schema`、`coreManifest`、`corePlan`、`coreInstall`），写测试时可以直接对照。
3. 在支持 API 4 的 Lithe 里安装插件，打开一个目标文件，设置断点并启动调试。适配器被拒绝请求时，Lithe 会显示适配器自己的解释，所以适配器返回的错误文字要写得让用户能看懂。
4. 在设置页确认 `executable` 找到的路径被回显，再把它改成一个不存在的文件，确认得到的是错误而不是悄悄换文件。

## 7. 安装后是否需要重启

只贡献调试适配器（以及它声明的语言和设置）的插件，在支持热激活的 Lithe 里安装、更新、回退、卸载、启用、停用后**不需要重启**。是否免重启由宿主按它实际能在运行期登记和撤回的贡献类型决定，不是 `plugin.json` 里的字段，所以插件作者不需要也不能声明。带有其他贡献（语言服务器、构建系统、界面等）的插件仍然在重启后生效。

## 8. 发布

调试适配器插件需要 API 4，只有支持 API 4 的 Lithe 才能安装。旧版 Lithe 在插件市场里会提示升级，而不是安装。

- 在 Lithe 稳定版支持 API 4 之前，把你的 GitHub Release 标为**预发布**。Lithe Store 的自动追踪只收正式 Release，预发布需要提交一个手工的索引条目，写法参考 Maven、Gradle 和 LLDB 插件的 0.1.0。
- 稳定版支持 API 4 后，把 Release 转为正式版，追踪器会自动提交后续版本的索引更新。
- 发布后不要替换或删除资产；索引固定了它的 SHA-256。

## 9. 当前边界

- 只有 macOS 宿主实现了调试适配器。
- 宿主调试入口目前只发起 `launch` 请求；`attach` 可以在清单里声明，但宿主入口还没有完成，不要依赖。
- 适配器声明是纯数据，没有自定义界面、没有宿主集成点；需要调试之外的能力（构建、测试、运行配置）请另用对应的贡献类型。
