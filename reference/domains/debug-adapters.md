# 调试适配器 / Debug adapters

[中文](#中文) · [English](#english)

## 中文

插件在 `debugAdapters[]` 里声明自己实现的调试适配器。每个适配器是一个 DAP 进程，宿主启动它，并通过 stdio 或本机回环 TCP 端口和它通信；也可以是插件语言服务器里内置的调试服务（例如运行在 JDT LS 里的 Java Debug Server）。声明只是数据：没有解析器、工作流、宿主集成或 `initializationOptions`，出现这些字段会被拒绝。调试界面、断点、变量、控制台都归宿主。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `debugAdapters[]` |
| 能力版本 | `debugAdapters` 1–2，需要级别 4；`launch.process` 用到的 `serverProcess` 1，`runSelectedRuntime` 需要 `serverProcess` 2。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | DAP（Debug Adapter Protocol），没有 Lithe 专用协议 |
| API 编号 | [API-30](../api-catalog.md#现在能用的-api)：调试适配器 |
| 权限 | `debug.launch`；任何适配器支持 `attach` 时加 `debug.attach`（远程 attach 不另设权限）；`tcpLanguageServer` 依赖同一插件的语言服务器，所以还要有 `languageServers` 和 `languageServer.launch` |
| 激活事件 | 每个适配器一个 `onDebug:<id>` |
| Windows | 不支持；Windows 宿主不声明 `debugAdapters` |

### 版本对照

| 版本 | 最低级别 | 新增 | macOS | Windows |
| --- | --- | --- | --- | --- |
| `debugAdapters` 1 | 4 | `debugAdapters[]` 的全部字段：`launch`/`attach` 请求、自动匹配 `selectors`、`configurationTypes`、默认参数与 `targetBindings`（`program`、`processId`、`cwd`、`connection`）、`executable` 候选、`capabilities.disabled`、`console`、四种传输 | 支持 | 不支持 |
| `debugAdapters` 2 | 4 | `targetBindings` 新增有界的 `args` ← `target.arguments`、`env` ← `target.environment` | 支持 | 不支持 |
| `serverProcess` 2 | 4 | `launch.process` 可以写成 `{"launch": {"kind": "runSelectedRuntime", "tool": "python" \| "java" \| "node"}}`，由宿主用用户选中的运行时启动适配器；这是 `serverProcess` 2 唯一的用途 | 只有 Core 支持，macOS 宿主只声明 `serverProcess` 1 | 不支持 |
| `buildProviders` 5 | 4 | 构建提供者为原生调试准备目标，运行声明 `debug: {"adapter": "native"}`；同时需要 `debugAdapters` 2，见[构建提供者](build-providers.md) | 支持 | 不支持 |
| `buildProviders` 11 | 3 | 服务运行声明 `debug: {"configurationType": "…"}`，宿主选一个已启用、支持 `attach` 和该类型的适配器连接 | 支持 | 不支持 |

低于级别 4 的清单不能声明 `debugAdapters`；级别 3 的清单不能要求 `debugAdapters` 或 `serverProcess` 2。

### 传输方式

| `launch.transport` | 端点 | 需要 `launch.process` |
| --- | --- | --- |
| `stdio` | 适配器的标准输入输出 | 是 |
| `tcpAnnounced` | 适配器在输出流里宣布端口：`endpoint` 带 `stream`、单行 `prefix`、`value`（`port` 或 `hostPort`） | 是 |
| `tcpPort` | 固定端口 `endpoint.port` | 是 |
| `tcpLanguageServer` | 同一插件的语言服务器：`endpoint` 带 `languageID` 和 `command`；宿主每次会话发 `workspace/executeCommand`，服务器回答回环端口 | 否，且不能有 |

所有 TCP 端点只能是本机回环地址。`tcpLanguageServer` 的命令是安装和更新审查的一部分，命令改变时会重新询问用户。

### 远程 attach

连接另一台机器上的被调试程序（带 `-agentlib:jdwp` 的 JVM、Node inspector、gdbserver）时，适配器仍在本机运行，只有被调试程序在远端。`attach` 配置的 `targetBindings.connection` 把用户确认的主机、端口、路径映射写进适配器的 `arguments`：

| 字段 | 说明 |
| --- | --- |
| `host` + `port` | 一起声明，值是 `arguments` 里的路径（1–4 段，如 `connect.host`） |
| `address` | 单个 `host:port` 字符串，IPv6 用方括号；可以代替 `host` + `port`，也可以同时写 |
| `pathMappings.shape: "objects"` | 对象列表，`localKey`、`remoteKey` 命名每个对象的键 |
| `pathMappings.shape: "first"` | 只有一对根目录的适配器；目标多于一对时拒绝，不截断 |
| `pathMappings.shape: "dictionary"` | 一个对象，`keyedBy` 决定哪一侧做键 |
| `pathMappings.shape: "pairs"` | 二元数组列表，`order` 为 `remoteLocal` 或 `localRemote` |

Core 只把 `localhost`、`127.0.0.0/8`、`::1` 判为本机；其他主机都是 `remote: true`，宿主每次都请用户确认，默认不记住。`connection` 不声明外部 DAP 端点或远程执行适配器（例如 `ssh host gdb -i=dap`），也不提供代理、隧道、端口转发。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [API level 4: debug adapters](../../contract/v1/README.md#api-level-4-debug-adapters) | 全部字段、绑定、`executable` 候选、控制台、传输 |
| [Remote attach](../../contract/v1/README.md#remote-attach) | `connection` 绑定、四种路径映射、目标格式、本机判定、拒绝层次 |
| [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | 构建提供者准备原生调试目标 |
| [Registry connection debugging (buildProviders 11)](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | 服务运行按配置类型选择 attach 适配器 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`debug-adapters/debug-adapter.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) | 合法示例：两个适配器（`example`、`native`），`settings` 2 的 `file` 设置选择要调试的程序 |
| [`debug-adapters/debug-adapter-language-server.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-language-server.plugin.json) | `tcpLanguageServer`：调试服务在插件的语言服务器里 |
| [`debug-adapters/debug-adapter-remote-attach.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-remote-attach.plugin.json) | 六个适配器，分别演示 `connection` 的不同写法：`flat`（平铺的 `host`、`port`）、`nested`（嵌套路径加 `objects`）、`scalar`（`first`）、`joined`（`address`）、`keyed`（`dictionary`）、`paired`（`pairs`） |
| [`debug-adapters/debug-adapter-rejections.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json) | 76 个拒绝用例，每个写明稳定错误码、字段和报告层次（`schema`、`coreManifest`、`corePlan`、`coreInstall`） |
| [`build-providers/build-provider-debug-preparation.plugin.json`](../../contract/v1/fixtures/build-providers/build-provider-debug-preparation.plugin.json) | `buildProviders` 5 加 `debugAdapters` 2 |
| [`build-providers/build-debug-preparation.json`](../../contract/v1/fixtures/build-providers/build-debug-preparation.json) | `lithe/build/prepareDebug` 的结果和拒绝用例 |
| [`build-providers/build-connection-debug-options.json`](../../contract/v1/fixtures/build-providers/build-connection-debug-options.json) | `buildProviders` 11 的服务运行调试选项 |
| [Lithe-IDEA/lithe-plugin-lldb](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | 官方 lldb-dap 调试插件（预发布），使用 `executable` 候选和 `console.commandPrefix` |

### 相关指南

- [编写调试适配器插件（API 4）](../../guides/debug-adapter-plugin.md)
- [原生构建、测试与运行准备（开发中）](../../guides/native-build-workflows.md)

---

## English

A plugin declares the debug adapters it implements in `debugAdapters[]`. Each adapter is a DAP process the host starts and talks to over stdio or a loopback TCP port, or a debug server built into the plugin's language server (such as the Java Debug Server inside JDT LS). The declaration is data only: there is no resolver, workflow, host integration, or `initializationOptions`, and any such field is refused. The debug UI, breakpoints, variables, and console belong to the host.

| Item | Value |
| --- | --- |
| Manifest field | `debugAdapters[]` |
| Capability versions | `debugAdapters` 1–2, at level 4; `serverProcess` 1 for `launch.process`, and `serverProcess` 2 for `runSelectedRuntime`. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | DAP (Debug Adapter Protocol); no Lithe-specific protocol |
| API IDs | [API-30](../api-catalog.md#apis-usable-now): debug adapters |
| Permissions | `debug.launch`; `debug.attach` when any adapter serves `attach` (remote attach has no separate permission); `tcpLanguageServer` relies on a language server of the same plugin, so it also needs `languageServers` and `languageServer.launch` |
| Activation events | One `onDebug:<id>` per adapter |
| Windows | Not supported; the Windows host does not advertise `debugAdapters` |

### Versions compared

| Version | Lowest level | Adds | macOS | Windows |
| --- | --- | --- | --- | --- |
| `debugAdapters` 1 | 4 | Every `debugAdapters[]` field: `launch`/`attach` requests, automatic `selectors`, `configurationTypes`, default arguments and `targetBindings` (`program`, `processId`, `cwd`, `connection`), `executable` candidates, `capabilities.disabled`, `console`, the four transports | Yes | No |
| `debugAdapters` 2 | 4 | Bounded `args` ← `target.arguments` and `env` ← `target.environment` in `targetBindings` | Yes | No |
| `serverProcess` 2 | 4 | `launch.process` may be `{"launch": {"kind": "runSelectedRuntime", "tool": "python" \| "java" \| "node"}}`, so the host starts the adapter with the runtime the user selected; this is the only use of `serverProcess` 2 | Core only; the macOS host advertises `serverProcess` 1 | No |
| `buildProviders` 5 | 4 | A build provider prepares a native debug target, and a run declares `debug: {"adapter": "native"}`; also needs `debugAdapters` 2; see [build providers](build-providers.md) | Yes | No |
| `buildProviders` 11 | 3 | A service run declares `debug: {"configurationType": "…"}` and the host connects an enabled adapter that serves `attach` and that type | Yes | No |

A manifest below level 4 may not declare `debugAdapters`, and a level 3 manifest may not require `debugAdapters` or `serverProcess` 2.

### Transports

| `launch.transport` | Endpoint | Needs `launch.process` |
| --- | --- | --- |
| `stdio` | The adapter's standard input and output | Yes |
| `tcpAnnounced` | The adapter announces its port on an output stream: `endpoint` has `stream`, a one-line `prefix`, and `value` (`port` or `hostPort`) | Yes |
| `tcpPort` | A fixed `endpoint.port` | Yes |
| `tcpLanguageServer` | A language server of the same plugin: `endpoint` has `languageID` and `command`; for each session the host sends `workspace/executeCommand` and the server answers a loopback port | No, and it must not have one |

Every TCP endpoint is loopback only. The `tcpLanguageServer` command is part of the install and update review, and changing it asks the user again.

### Remote attach

When the debuggee runs on another machine (a JVM with `-agentlib:jdwp`, a Node inspector, gdbserver), the adapter still runs locally and only the debuggee is remote. The `targetBindings.connection` of an `attach` configuration writes the host, port, and path mappings the user confirmed into the adapter's `arguments`:

| Field | Notes |
| --- | --- |
| `host` + `port` | Declared together; values are paths into `arguments` (1–4 segments, such as `connect.host`) |
| `address` | One `host:port` string, IPv6 in brackets; replaces `host` + `port` or sits beside them |
| `pathMappings.shape: "objects"` | A list of objects; `localKey` and `remoteKey` name each object's keys |
| `pathMappings.shape: "first"` | For an adapter with a single pair of roots; a target with more is refused, not truncated |
| `pathMappings.shape: "dictionary"` | One object; `keyedBy` chooses which side is the key |
| `pathMappings.shape: "pairs"` | A list of two-element arrays; `order` is `remoteLocal` or `localRemote` |

Core treats only `localhost`, `127.0.0.0/8`, and `::1` as local; every other host is `remote: true`, and the host asks the user to confirm it each time without remembering the answer by default. `connection` declares no external DAP endpoint or remote adapter execution (such as `ssh host gdb -i=dap`), and no proxy, tunnel, or port forwarding.

### Contract sections

| Section | Content |
| --- | --- |
| [API level 4: debug adapters](../../contract/v1/README.md#api-level-4-debug-adapters) | Every field, bindings, `executable` candidates, console, transports |
| [Remote attach](../../contract/v1/README.md#remote-attach) | The `connection` binding, the four mapping shapes, the target format, local detection, refusal layers |
| [Native debug preparation (version 5)](../../contract/v1/README.md#native-debug-preparation-version-5) | Build providers preparing native debug targets |
| [Registry connection debugging (buildProviders 11)](../../contract/v1/README.md#registry-connection-debugging-buildproviders-11) | Service runs choosing an attach adapter by configuration type |

### Examples and cases

| File | Content |
| --- | --- |
| [`debug-adapters/debug-adapter.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) | Valid example: two adapters (`example`, `native`), and a `settings` 2 `file` setting picks the program to debug |
| [`debug-adapters/debug-adapter-language-server.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-language-server.plugin.json) | `tcpLanguageServer`: the debug server lives in the plugin's language server |
| [`debug-adapters/debug-adapter-remote-attach.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-remote-attach.plugin.json) | Six adapters, one per way of taking a `connection`: `flat` (top-level `host` and `port`), `nested` (nested paths with `objects`), `scalar` (`first`), `joined` (`address`), `keyed` (`dictionary`), `paired` (`pairs`) |
| [`debug-adapters/debug-adapter-rejections.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter-rejections.json) | 76 refusal cases, each with its stable code, field, and reporting layer (`schema`, `coreManifest`, `corePlan`, `coreInstall`) |
| [`build-providers/build-provider-debug-preparation.plugin.json`](../../contract/v1/fixtures/build-providers/build-provider-debug-preparation.plugin.json) | `buildProviders` 5 with `debugAdapters` 2 |
| [`build-providers/build-debug-preparation.json`](../../contract/v1/fixtures/build-providers/build-debug-preparation.json) | `lithe/build/prepareDebug` results and refusal cases |
| [`build-providers/build-connection-debug-options.json`](../../contract/v1/fixtures/build-providers/build-connection-debug-options.json) | `buildProviders` 11 service-run debug options |
| [Lithe-IDEA/lithe-plugin-lldb](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | The official lldb-dap debug plugin (prerelease), using `executable` candidates and `console.commandPrefix` |

### Related guides

- [Writing a debug adapter plugin (API 4)](../../guides/debug-adapter-plugin.md)
- [Native build, test, and run preparation](../../guides/native-build-workflows.md)
