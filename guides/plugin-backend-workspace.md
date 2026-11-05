# 插件后端：工作区上下文与宿主服务（backend 2、协议 3） / Plugin backend: workspace contexts and host services (backend 2, protocol 3)

[中文](#中文) · [English](#english)

## 中文

这篇指南面向写插件后端（`backend`）的作者。目标读者已经能让自己的网页调用后端函数（`backend` 1），
现在想让后端在没有页面时也能启动、知道用户打开了哪些工作区、响应命令，并按需调用宿主能力。

字段与错误码的权威定义在[契约](../contract/v1/README.md#backend-2-workspace-contexts)，这里只讲怎么用、为什么这样设计。

> **宿主支持。** `requires.backend: 2` 和插件函数协议 3 由 macOS 宿主实现；Windows 宿主不提供 `backend`。
> 只提供 `backend: [1]` 的旧宿主会以 `plugin.api.unsupported` 拒绝 `backend` 2 的插件，所以老版本 Lithe
> 不会启动一个期待工作区上下文的进程。三个官方 SDK 都实现了协议 3：Node.js SDK 0.2.0、Java SDK 0.2.0 和 Rust SDK 0.1.0 起
> （见各自 README 的 Host services：[Node.js](../sdk/node/README.md#host-services)、[Java](../sdk/java/README.md#host-services)、
> [Rust](../sdk/rust/README.md#host-services)）。宿主还支持函数协议 4（跨插件调用，见第 10 节），三个官方 SDK 目前只实现到协议 3。
> 用其他语言或更早的 SDK 需要自己处理下文的请求和通知。
> 各宿主支持到哪一版，见按最新 preview 核对的[版本对照](../reference/capabilities-and-versions.md)。

### 1. 三个版本号各管什么

| 写在哪里 | 值 | 决定什么 |
| --- | --- | --- |
| `requires.backend` | `1` 或 `2` | 后端进程**什么时候启动**、能不能声明 `workspace.metadata` 和 `onWorkspaceOpen` |
| `initialize` 应答的 `litheFunctions.version` | `1`、`2`、`3` 或 `4` | 进程启动后**线路协议**的规则：`2` 起有宿主服务（凭据），`3` 起有能力发现、工作区上下文、宿主通知和带类型的失败码，`4` 起有跨插件调用（第 10 节） |
| `getCapabilities.services` | 运行时查询结果 | 本次会话**实际能调用**哪些宿主方法、会收到哪些通知 |

清单只做声明，宿主读清单决定装不装、什么时候启动；进程起来以后能做什么，靠协商和 `getCapabilities` 判断。

### 2. 最小清单

```json
{
  "schemaVersion": 2,
  "id": "io.github.<你的用户名>.orders",
  "displayName": "Orders",
  "version": "1.0.0",
  "pluginAPIVersion": 3,
  "requires": { "backend": 2, "serverProcess": 1 },
  "vendor": { "id": "io.github.<你的用户名>", "displayName": "<你的名字>", "signatureRequirement": "none" },
  "permissions": ["backend.launch", "workspace.metadata"],
  "activationEvents": ["onWorkspaceOpen"],
  "languages": [],
  "backend": {
    "launch": { "kind": "packaged", "path": "bin/backend" },
    "arguments": ["--state", "${stateDir}"]
  }
}
```

- `backend` 2 至少要有一种启动方式：网页工具窗口（首次页面调用，同 `backend` 1）、`ui.commands[]` 里的命令
  （`onCommand:<id>` 也会启动后端），或 `onWorkspaceOpen`。一种都没有是 `plugin.manifest.invalid`。
- 只想要命令、不要工具窗口时，声明 `requires.ui: 4`：命令可以不写 `toolWindowID`，`toolWindows` 可以为空或省略，
  见第 7 节。`ui` 1～3 的命令仍必须挂在工具窗口上。
- `workspace.metadata` 只让后端知道打开的工作区名字、根目录、ID、来源（本地或远程）和每个工作区的私有状态目录，
  不读任何文件内容。不需要这些信息就不要声明，安装审查会把它列给用户。
- 仍然拒绝 `${workspaceStateDir}`：一个插件只有一个后端进程，被所有窗口共享，工作区不是进程状态。
  每个工作区的状态目录从 `getContext` 的 `stateDirectory` 取。

### 3. 启动时机

- `onWorkspaceOpen` 不会拖慢打开工作区。宿主在窗口显示之后逐个启动这些后端；`initialize` 超时的后端会被停止，
  并像其他启动失败一样报告，不会挡住工作区。
- 后端在 Lithe 所在的机器上运行，远程工作区也一样。
- 激活只看插件是否已安装、启用、通过审查。Lithe 目前没有工作区信任状态；会运行或修改东西的操作，由宿主逐次询问用户。

### 4. 协商协议 3

宿主在 `initialize` 里提供 `capabilities.experimental.litheFunctions: { "versions": [1, 2, 3, 4] }`。后端从中选自己实现的最高版本作答（本篇讲协议 3，协议 4 见第 10 节）：

```json
"experimental": { "litheFunctions": { "version": 3, "functions": [{ "id": "orders.refresh" }] } }
```

只回答你真正实现了的版本。回答 `2` 的后端继续按协议 2 工作，永远看不到协议 3 新增的通知、上下文字段和失败码。

### 5. 先问宿主能做什么

协商到 3 以后，第一件事是调 `lithe/host/getCapabilities`（不带参数或 `{}`）：

```json
{
  "protocolVersion": 3,
  "backendAuthority": "local",
  "services": {
    "commands": { "methods": [], "notifications": ["lithe/host/commands/didRun"] },
    "host": { "methods": ["lithe/host/getCapabilities"], "notifications": [] },
    "subscriptions": {
      "methods": ["lithe/host/subscriptions/dispose"],
      "notifications": ["lithe/host/subscriptions/notify"]
    },
    "workspace": {
      "methods": ["lithe/host/workspace/listContexts", "lithe/host/workspace/getContext", "lithe/host/workspace/getFolders"],
      "notifications": ["lithe/host/workspace/didChangeContexts"]
    }
  },
  "limits": {
    "pendingHostRequests": 16,
    "functionResultBytes": 1048576,
    "hostRequestBytes": 1048576,
    "hostResultBytes": 1048576,
    "errorMessageBytes": 1024,
    "errorDetailsBytes": 4096,
    "notificationBytes": 1048576
  }
}
```

上例是声明了 `workspace.metadata` 的会话。`host`、`subscriptions`、`commands` 每个插件会话都有；`services` 只列本次会话可用的方法，
没声明的权限、宿主没有实现的服务都不会出现。数值以实际返回为准。**按 `services` 判断能不能调用，不要按协议版本号推断**，见第 10 节。

### 6. 工作区上下文

一个上下文就是“某个窗口里打开的某个工作区”：

```json
{
  "contextID": "w-3f2a9c",
  "generation": 2,
  "name": "orders",
  "authority": { "kind": "local" },
  "folders": [{ "rootID": "r0c1d2e3f4a5b6c7d", "name": "orders", "path": "/Users/me/orders" }],
  "stateDirectory": "/Users/me/Library/Application Support/Lithe/PluginState/<plugin>/backend/workspaces/<digest>"
}
```

| 字段 | 怎么用 |
| --- | --- |
| `contextID` | 窗口显示这个工作区期间不变。窗口切到别的工作区时，旧上下文关闭、新上下文换新 ID，所以按 ID 缓存的状态不会串到别的工作区 |
| `generation` | 同一工作区的根目录或来源变化时加一。请求里带旧的 `generation`，或上下文已关闭，会得到 `plugin.host.contextExpired`，重新 `listContexts` 即可 |
| `rootID` | 同一位置跨重启稳定，不暴露路径；按插件加密派生，两个插件对同一目录看到的 ID 不同 |
| `path` | 只有本地工作区有。远程工作区（`authority.kind` 为 `remote`）没有 `path`，宿主在本地为它保留的目录只是空占位 |
| `stateDirectory` | 插件与工作区私有，在插件包和应用包之外。只有 `getContext` 返回它并在那时创建；`listContexts` 不创建目录 |

常用请求：

| 请求 | 参数 | 结果 |
| --- | --- | --- |
| `lithe/host/workspace/listContexts` | 无 | `{ revision, contexts }`，条目不含 `stateDirectory` |
| `lithe/host/workspace/getContext` | `{ context: { contextID, generation } }` | 完整上下文 |
| `lithe/host/workspace/getFolders` | `{ context: { contextID, generation } }` | `{ folders }` |

页面调用后端函数时，宿主会按页面所在窗口填好上下文：协议 3 的后端在 `lithe/function/invoke` 的
`params.context` 里收到 `{ contextID, generation }`，直接拿它调 `getContext`。

### 7. 宿主通知

宿主用 JSON-RPC 通知告诉后端发生了变化，后端不应答：

| 通知 | 参数 | 什么时候来 |
| --- | --- | --- |
| `lithe/host/workspace/didChangeContexts` | `{ revision }` | 有上下文打开、关闭或变化；收到后重新 `listContexts` |
| `lithe/host/commands/didRun` | `{ commandID, context? }` | 用户运行了插件自己的命令。后端没在运行且声明了 `onCommand:<commandID>` 时，宿主先启动它，就绪后再送达 |
| `lithe/host/subscriptions/notify` | `{ subscriptionID, sequence, event }` | 某个订阅的事件。`sequence` 每个订阅从 1 起逐一递增，可以据此发现丢失；用 `lithe/host/subscriptions/dispose` 取消订阅 |

后端晚启动时不会补发之前的 `didChangeContexts`，启动后自己调一次 `listContexts`。

`didRun` 只是事件，不代表命令要做的事已经成功；宿主也不会为了送达它而打开一个空工具窗口。

#### 纯后端命令（`ui` 4）

```json
"requires": { "backend": 2, "serverProcess": 1, "ui": 4 },
"permissions": ["backend.launch", "ui.contribute"],
"activationEvents": ["onCommand:io.github.<你的用户名>.orders.refresh"],
"ui": {
  "commands": [
    { "id": "io.github.<你的用户名>.orders.refresh", "title": "Refresh Orders", "placements": ["commandPalette"] }
  ]
}
```

这样的命令要求 `backend` 2、`ui.contribute` 与 `backend.launch` 权限，以及对应的 `onCommand:<id>`。
不认识 `ui` 4 的旧宿主会在安装时拒绝插件，不会装上一个执行不了的命令。完整示例见
[`fixtures/backend/command-backend.plugin.json`](../contract/v1/fixtures/backend/command-backend.plugin.json)。

### 8. 文档、文件与窗口方法

协议 3 还开放了一组公共宿主方法，每组对应一个新权限。目前只有 macOS 宿主实现；Windows 宿主不提供 `backend`。
`getCapabilities.services` 只列出“清单权限推出的授权”与“本宿主已实现的方法”的交集，所以照第 10 节第 1 条先查再调。

| 方法（前缀 `lithe/host/`） | 权限 | 用途 |
| --- | --- | --- |
| `documents/list`、`documents/getSnapshot`、`editor/getActive`、`editor/getSelections` | `documents.read` | 列出打开的文档，读编辑器里未保存的内容、当前标签页和选区 |
| `editor/applyEdits`、`editor/replaceSelections`、`editor/insertText` | `editor.edit` | 带 `expectedVersion` 的编辑；一批编辑是一次原子操作、一步撤销 |
| `workspace/readDirectory`、`workspace/stat`、`workspace/readFile` | `workspace.files.read` | 读磁盘上的目录和文件 |
| `workspace/watch` | `workspace.files.watch` | 订阅文件变化，事件经 `subscriptions/notify` 送达 |
| `workspace/findFiles`、`workspace/findText` | `workspace.search` | 按文件名或内容搜索，分页返回 |
| `editor/open`、`editor/reveal` | `editor.open` | 打开文件并定位到某处 |
| `preview/openHTML` | `preview.open` | 用内置预览打开 HTML 文件 |
| `browser/openExternal` | `browser.open` | 用系统浏览器打开 HTTP(S) 链接或 HTML 文件 |
| `window/showInformation`、`window/showWarning`、`window/showError` | `window.messages` | 显示消息，可带最多 3 个按钮，返回用户选了哪个 |

参数、结果和上限的权威定义在[契约](../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3)。写后端时要注意：

- 每个方法都要带 `context: { contextID, generation }`。文件用 `{ rootID, path }` 指明，`path` 用 `/`、相对于
  `getFolders` 返回的根目录。这组方法明确拒绝远程工作区（`authorityUnsupported`）。
- `workspace.metadata` 只给工作区信息，**不授权读文件**；读文件要另外声明 `workspace.files.read`。
- 位置 `{ line, column }` 从 1 开始、按 UTF-16 计；范围是半开区间。
- 单次读取最多返回 128 KiB：快照超出时得到 `limitExceeded`，读文件则用 `isTruncated` 标明截断。一批编辑或选区最多 1,000 项。
- 取消或超时**不代表已回滚**：编辑或打开可能已经生效，不要自动重放修改。遇到 `versionConflict` 先重新读取再计算；
  遇到 `contextExpired` 要重新找到原来的目标，不要改用当前聚焦的窗口；`busy`、`rateLimited` 可以有限次重试；
  `permissionDenied`、`readOnly` 需要用户处理。
- 后台没有挂载编辑器的文档可以列出和读取，但编辑与选区可能返回 `busy`。
- 消息有频率和排队上限（每插件 10 秒 5 条、最多 3 条待答），用户回答前请求一直挂起。

协议 3 还包括插件自有的编辑器动作（`editor.actions`）、编辑器语言功能 Provider（`editor.providers`）和原生终端（`terminal.create`），分别见[编辑器、文档与预览](../reference/domains/editor.md)和[从插件打开 Lithe 原生终端](native-terminals.md)。

协议 3 里还有三组方法，同样只有 macOS 宿主实现，调用前同样先查 `getCapabilities.services`。它们的权限都要求 `backend` 2，各自的清单条件见对应的契约章节：

- 运行与调试会话（权限 `run.start`、`debug.start`）：`run/start`、`run/stop` 启动并停止由宿主拥有的 Run 会话（Run 窗口条目、控制台、停止按钮、退出状态），`debug/start` 在本插件自己声明的调试适配器上启动调试会话，`run/didExit` 通知运行结束；见[插件后端与插件函数](../reference/domains/backend.md)、[调试适配器](../reference/domains/debug-adapters.md)和契约的[运行与调试会话](../contract/v1/README.md#run-and-debug-sessions-protocol-version-3)。
- 自有语言服务器（权限 `languageServer.request`）：`languageServer/request`、`languageServer/notify` 向本插件自己声明的语言服务器发送请求和通知，`languageServer/setStatus` 在宿主状态界面显示它的准备状态，服务器状态变化和转发的通知经 `languageServer/didChangeState`、`languageServer/didReceiveNotification` 送达；见[语言、语言服务器与运行/测试](../reference/domains/languages.md)和契约的[语言服务器宿主方法](../contract/v1/README.md#language-server-host-methods)。
- 内容 Provider（权限 `documents.contentProvider`）：`documents/registerContentProvider` 把一个 URI scheme 绑定到后端函数，用户打开该 scheme 的文档时宿主调用这个函数取只读内容；见[编辑器、文档与预览](../reference/domains/editor.md)和契约的[内容 Provider](../contract/v1/README.md#content-providers)。

### 9. 宿主失败与错误码

协议 3 的宿主失败带类型：JSON-RPC 号码与 `error.data.code` 一一对应，号码永不改义，部分失败还带 `error.data.details`。

| `error.data.code` | 号码 | 后端通常怎么做 |
| --- | --- | --- |
| `plugin.host.unavailable` | `-32601` | 宿主没有这个方法；先查 `getCapabilities` |
| `plugin.host.invalidParams` | `-32602` | 参数格式错误，修代码 |
| `plugin.host.failed` | `-32603` | 其他失败，报告给用户 |
| `plugin.host.cancelled` | `-32800` | 你自己用 `$/cancelRequest` 取消了它 |
| `plugin.host.permissionDenied` | `-32010` | 没声明权限，或用户拒绝 |
| `plugin.host.contextExpired` | `-32011` | 重新 `listContexts` 后再试 |
| `plugin.host.authorityUnsupported` | `-32012` | 该服务不支持远程工作区，宿主不会改在本地执行 |
| `plugin.host.versionConflict` | `-32013` | 资源版本已变；`details.currentVersion` 给出当前版本 |
| `plugin.host.limitExceeded` | `-32014` | 超出限制，例如同时挂起的请求超过 `limits.pendingHostRequests` |
| `plugin.host.timeout` | `-32015` | 宿主超时 |
| `plugin.host.notFound` | `-32016` | 资源不存在 |
| `plugin.host.readOnly` | `-32017` | 资源或工作区不可写 |
| `plugin.host.unsupportedOption` | `-32018` | 宿主不认识请求里的某个字段；`details.option` 指明是哪个，去掉它重试 |
| `plugin.host.preconditionFailed` | `-32019` | 所需状态不成立，例如用户没有确认 |
| `plugin.host.busy` | `-32020` | 资源正被占用，稍后重试 |
| `plugin.host.rateLimited` | `-32021` | 请求太频繁；有 `details.retryAfterMilliseconds` 时按它等待 |

`-32000` 到 `-32009` 留给后端自己的失败，宿主号码从 `-32010` 起。

### 10. 协议 3 只增不改：后端要遵守的四条

从协议 3 起，宿主再开放新能力（新服务及其方法和通知、可选参数、结果字段、错误码）都在协议 3 内新增，
不再升协议版本；只有会让已有后端读错的改动（删除或改名、改字段含义或类型、把可选参数改成必填、改错误码号码）
才会出新的协议版本，与旧版本并行提供。这样插件只要实现一次协议 3，就能在今后所有协议 3 宿主上工作，前提是遵守下面四条：

1. **只调用 `getCapabilities.services` 列出的方法。** 其他方法会得到 `-32601` `plugin.host.unavailable`。
2. **忽略不认识的字段和通知。** 结果、通知、`details`、`limits` 里多出来的字段，以及不认识的 `lithe/host/*` 通知，都直接忽略。
3. **新选项不被支持时会得到明确回答。** 请求里带了宿主不认识的字段，Core 在宿主看到请求之前回
   `-32018` `plugin.host.unsupportedOption`，`details: { "option": "<字段名>" }`。宿主不会悄悄忽略它，
   你可以去掉这个选项重试，或告诉用户需要更新 Lithe。
4. **不认识的号码按宿主失败处理。** 新宿主可能带来你没见过的错误码；认识 `error.data.code` 时按它分支，否则当作一般失败。

新服务总是配一个新的清单权限。旧宿主不认识这个权限时，会在安装时以 `plugin.permission.unknown` 拒绝整个插件包，
权限不会被悄悄授予或忽略。

#### 协议 4：跨插件调用

协议 4 只为[跨插件调用](../contract/v1/README.md#cross-plugin-calls-protocol-version-4)增加两样东西，协议 3 的内容全部保留，上面四条规则照旧：

- 调用方用的两个中转方法：先用 `functions/providers` 找到导出某个函数的插件，再用 `functions/invoke` 调用其中一个。
  调用由 Core 中转，调用方接触不到被调方的进程。只有协商到协议 4、且清单至少声明了一个调用（`pluginFunctions.calls`）的后端能发这两个请求：
  协商到 3 或更早得到 `-32601` `plugin.host.unavailable`，没有声明调用得到 `-32010` `plugin.host.permissionDenied`。
- 被中转调用里的 `caller`：被调方收到的 `lithe/function/invoke` 顶层多一个 `caller`，是 Core 按调用方会话写入的插件 ID；
  没有这个字段就表示宿主发起的调用。要按调用方判断的被调方只读这个顶层字段，不读 `params` 里自称的插件 ID。
  被调方在处理中转调用时再发起嵌套调用，要把这次调用顶层的 `operationID` 填进 `parentOperationID`，Core 才知道嵌套调用属于哪条调用链。

使用时要记住：

- 只协商到 3 的后端照常工作。它不能调用 `functions/*`；它导出的函数仍可被中转调用，但收不到 `caller`，要按调用方判断就必须协商 4。
- 导出或调用跨插件函数，清单要声明 `pluginFunctions`：`pluginAPIVersion` 不低于 3，`requires` 里写 `pluginFunctions: 1`、`backend: 2`、`serverProcess: 1`。
  它不授予任何清单权限；清单声明了导出，后端仍要用 `lithe/function/register` 注册该函数。
- 目前只有 macOS 宿主实现；三个官方 SDK 还只协商到协议 3，没有 `functions/*` 的封装，也读不到 `caller`。

### 11. 验证

- 用契约里的 [`fixtures/backend/function-protocol.json`](../contract/v1/fixtures/backend/function-protocol.json) 核对你的实现：
  `hostRequests` 有不认识的选项等请求用例，`invocationContexts` 有调用上下文，`hostFailures` 有失败码映射，
  `hostNotifications` 有通知用例。
- 清单示例是 [`fixtures/backend/backend-workspace.plugin.json`](../contract/v1/fixtures/backend/backend-workspace.plugin.json)；
  纯后端命令的示例是 [`fixtures/backend/command-backend.plugin.json`](../contract/v1/fixtures/backend/command-backend.plugin.json)；
  [`fixtures/manifest/invalid-manifests.json`](../contract/v1/fixtures/manifest/invalid-manifests.json) 里的 backend 用例是会被拒绝的写法。

---

## English

This guide is for authors writing a plugin backend (`backend`). It assumes your web page can already call backend functions (`backend` 1),
and you now want the backend to start without a page, know which workspaces the user has open, respond to commands, and call host services when needed.

The [contract](../contract/v1/README.md#backend-2-workspace-contexts) is normative for fields and error codes; this guide covers how to use them and why they are designed this way.

> **Host support.** The macOS host implements `requires.backend: 2` and plugin-function protocol 3; the Windows host has no `backend`.
> An older host that offers only `backend: [1]` refuses a `backend` 2 plugin with `plugin.api.unsupported`, so an older Lithe
> never starts a process that expects workspace contexts. All three official SDKs implement protocol 3 from Node.js SDK 0.2.0, Java SDK 0.2.0, and Rust SDK 0.1.0
> (see Host services in each README: [Node.js](../sdk/node/README.md#host-services), [Java](../sdk/java/README.md#host-services),
> [Rust](../sdk/rust/README.md#host-services)). The host also supports function protocol 4 (cross-plugin calls, see section 10); the three official SDKs implement only up to protocol 3 so far.
> With another language or an earlier SDK, handle the requests and notifications below yourself.
> For how far each host supports each version, see the [version tables](../reference/capabilities-and-versions.md), which track the latest preview.

### 1. What each of the three version numbers controls

| Where | Value | Controls |
| --- | --- | --- |
| `requires.backend` | `1` or `2` | **When** the backend process starts, and whether it may declare `workspace.metadata` and `onWorkspaceOpen` |
| `litheFunctions.version` in the `initialize` answer | `1`, `2`, `3`, or `4` | The **wire protocol** rules once the process runs: `2` adds host services (credentials); `3` adds capability discovery, workspace contexts, host notifications, and typed failure codes; `4` adds cross-plugin calls (section 10) |
| `getCapabilities.services` | Runtime query result | Which host methods this session **can actually call**, and which notifications it receives |

The manifest only declares. The host reads it to decide whether to install and when to start; what the running process can do is decided by negotiation and `getCapabilities`.

### 2. Minimal manifest

```json
{
  "schemaVersion": 2,
  "id": "io.github.<your-username>.orders",
  "displayName": "Orders",
  "version": "1.0.0",
  "pluginAPIVersion": 3,
  "requires": { "backend": 2, "serverProcess": 1 },
  "vendor": { "id": "io.github.<your-username>", "displayName": "<your name>", "signatureRequirement": "none" },
  "permissions": ["backend.launch", "workspace.metadata"],
  "activationEvents": ["onWorkspaceOpen"],
  "languages": [],
  "backend": {
    "launch": { "kind": "packaged", "path": "bin/backend" },
    "arguments": ["--state", "${stateDir}"]
  }
}
```

- `backend` 2 needs at least one way to start: a web tool window (the first page call, as in `backend` 1), a command in `ui.commands[]`
  (`onCommand:<id>` also starts the backend), or `onWorkspaceOpen`. Having none is `plugin.manifest.invalid`.
- For commands without a tool window, declare `requires.ui: 4`: commands may omit `toolWindowID`, and `toolWindows` may be empty or omitted;
  see section 7. Commands under `ui` 1–3 must still belong to a tool window.
- `workspace.metadata` only tells the backend the open workspaces' names, roots, IDs, authority (local or remote), and each workspace's private state directory;
  it reads no file contents. Do not declare it unless you need this information; installation review lists it to the user.
- `${workspaceStateDir}` is still refused: a plugin has one backend process shared by every window, so a workspace is not process state.
  Take each workspace's state directory from `stateDirectory` in `getContext`.

### 3. When the backend starts

- `onWorkspaceOpen` does not slow down opening a workspace. The host starts these backends one by one after the window appears; a backend whose `initialize` times out is stopped
  and reported like any other start failure, without blocking the workspace.
- The backend runs on the machine that runs Lithe, for remote workspaces too.
- Activation only checks that the plugin is installed, enabled, and approved in review. Lithe has no workspace trust state yet; the host asks the user each time before an operation runs or changes something.

### 4. Negotiating protocol 3

The host offers `capabilities.experimental.litheFunctions: { "versions": [1, 2, 3, 4] }` in `initialize`. The backend answers with the highest version it implements (this guide covers protocol 3; for protocol 4 see section 10):

```json
"experimental": { "litheFunctions": { "version": 3, "functions": [{ "id": "orders.refresh" }] } }
```

Answer only versions you actually implement. A backend that answers `2` keeps working under protocol 2 and never sees the notifications, context fields, and failure codes added in protocol 3.

### 5. Ask the host what it can do first

Once protocol 3 is negotiated, first call `lithe/host/getCapabilities` (no params, or `{}`):

```json
{
  "protocolVersion": 3,
  "backendAuthority": "local",
  "services": {
    "commands": { "methods": [], "notifications": ["lithe/host/commands/didRun"] },
    "host": { "methods": ["lithe/host/getCapabilities"], "notifications": [] },
    "subscriptions": {
      "methods": ["lithe/host/subscriptions/dispose"],
      "notifications": ["lithe/host/subscriptions/notify"]
    },
    "workspace": {
      "methods": ["lithe/host/workspace/listContexts", "lithe/host/workspace/getContext", "lithe/host/workspace/getFolders"],
      "notifications": ["lithe/host/workspace/didChangeContexts"]
    }
  },
  "limits": {
    "pendingHostRequests": 16,
    "functionResultBytes": 1048576,
    "hostRequestBytes": 1048576,
    "hostResultBytes": 1048576,
    "errorMessageBytes": 1024,
    "errorDetailsBytes": 4096,
    "notificationBytes": 1048576
  }
}
```

The example is a session that declared `workspace.metadata`. Every plugin session has `host`, `subscriptions`, and `commands`; `services` lists only the methods this session may use,
so undeclared permissions and services the host does not implement never appear. Treat the numbers as whatever the host returns. **Decide what you can call from `services`, not from the protocol version**; see section 10.

### 6. Workspace contexts

A context is "one workspace open in one window":

```json
{
  "contextID": "w-3f2a9c",
  "generation": 2,
  "name": "orders",
  "authority": { "kind": "local" },
  "folders": [{ "rootID": "r0c1d2e3f4a5b6c7d", "name": "orders", "path": "/Users/me/orders" }],
  "stateDirectory": "/Users/me/Library/Application Support/Lithe/PluginState/<plugin>/backend/workspaces/<digest>"
}
```

| Field | How to use it |
| --- | --- |
| `contextID` | Stable while the window shows this workspace. When the window switches to another workspace, the old context closes and the new one gets a new ID, so state cached by ID never leaks into another workspace |
| `generation` | Increments when the same workspace's roots or authority change. A request with a stale `generation`, or for a closed context, gets `plugin.host.contextExpired`; call `listContexts` again |
| `rootID` | Stable for the same location across restarts and does not reveal the path; derived cryptographically per plugin, so two plugins see different IDs for the same folder |
| `path` | Present only for local workspaces. A remote workspace (`authority.kind` is `remote`) has no `path`, and the local directory the host keeps for it is only an empty placeholder |
| `stateDirectory` | Private to the plugin and the workspace, outside the plugin package and the app bundle. Only `getContext` returns it, creating it at that moment; `listContexts` creates no directory |

Common requests:

| Request | Params | Result |
| --- | --- | --- |
| `lithe/host/workspace/listContexts` | none | `{ revision, contexts }`; entries have no `stateDirectory` |
| `lithe/host/workspace/getContext` | `{ context: { contextID, generation } }` | The full context |
| `lithe/host/workspace/getFolders` | `{ context: { contextID, generation } }` | `{ folders }` |

When a page calls a backend function, the host fills in the context of the page's window: a protocol 3 backend receives `{ contextID, generation }`
in `params.context` of `lithe/function/invoke` and can pass it straight to `getContext`.

### 7. Host notifications

The host tells the backend about changes with JSON-RPC notifications; the backend does not answer them:

| Notification | Params | When it arrives |
| --- | --- | --- |
| `lithe/host/workspace/didChangeContexts` | `{ revision }` | A context opened, closed, or changed; call `listContexts` again |
| `lithe/host/commands/didRun` | `{ commandID, context? }` | The user ran one of the plugin's own commands. If the backend is not running and declares `onCommand:<commandID>`, the host starts it first and delivers the notification once it is ready |
| `lithe/host/subscriptions/notify` | `{ subscriptionID, sequence, event }` | An event of a subscription. `sequence` starts at 1 per subscription and increments by one, so gaps reveal losses; cancel with `lithe/host/subscriptions/dispose` |

A backend that starts late gets no replay of earlier `didChangeContexts`; call `listContexts` once after starting.

`didRun` is only an event and does not mean the command's work succeeded; the host also never opens an empty tool window just to deliver it.

#### Backend-only commands (`ui` 4)

```json
"requires": { "backend": 2, "serverProcess": 1, "ui": 4 },
"permissions": ["backend.launch", "ui.contribute"],
"activationEvents": ["onCommand:io.github.<your-username>.orders.refresh"],
"ui": {
  "commands": [
    { "id": "io.github.<your-username>.orders.refresh", "title": "Refresh Orders", "placements": ["commandPalette"] }
  ]
}
```

Such a command requires `backend` 2, the `ui.contribute` and `backend.launch` permissions, and the matching `onCommand:<id>`.
An older host that does not know `ui` 4 refuses the plugin at installation, so it never installs a command it cannot run. For a complete example, see
[`fixtures/backend/command-backend.plugin.json`](../contract/v1/fixtures/backend/command-backend.plugin.json).

### 8. Document, file, and window methods

Protocol 3 also opens a set of public host methods, each group behind a new permission. Only the macOS host implements them; the Windows host has no `backend`.
`getCapabilities.services` lists only the intersection of "grants derived from the manifest's permissions" and "methods this host implements", so check before calling, as rule 1 of section 10 says.

| Method (prefix `lithe/host/`) | Permission | Purpose |
| --- | --- | --- |
| `documents/list`, `documents/getSnapshot`, `editor/getActive`, `editor/getSelections` | `documents.read` | List open documents; read unsaved editor content, the active tab, and selections |
| `editor/applyEdits`, `editor/replaceSelections`, `editor/insertText` | `editor.edit` | Edits with `expectedVersion`; a batch is one atomic operation and one undo step |
| `workspace/readDirectory`, `workspace/stat`, `workspace/readFile` | `workspace.files.read` | Read directories and files on disk |
| `workspace/watch` | `workspace.files.watch` | Subscribe to file changes; events arrive through `subscriptions/notify` |
| `workspace/findFiles`, `workspace/findText` | `workspace.search` | Search by file name or content, paged |
| `editor/open`, `editor/reveal` | `editor.open` | Open a file and go to a location |
| `preview/openHTML` | `preview.open` | Open an HTML file in the built-in preview |
| `browser/openExternal` | `browser.open` | Open an HTTP(S) link or HTML file in the system browser |
| `window/showInformation`, `window/showWarning`, `window/showError` | `window.messages` | Show a message with up to 3 buttons and return which one the user chose |

The [contract](../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) is normative for params, results, and limits. When writing a backend:

- Every method takes `context: { contextID, generation }`. Files are named by `{ rootID, path }`, where `path` uses `/` and is relative to
  a root returned by `getFolders`. These methods refuse remote workspaces explicitly (`authorityUnsupported`).
- `workspace.metadata` only gives workspace information and **does not grant file reads**; declare `workspace.files.read` separately to read files.
- Positions `{ line, column }` start at 1 and count UTF-16 units; ranges are half-open.
- One read returns at most 128 KiB: an oversized snapshot gets `limitExceeded`, and a file read marks truncation with `isTruncated`. A batch of edits or selections has at most 1,000 items.
- Cancellation or timeout **does not mean rollback**: an edit or open may already have taken effect, so do not replay changes automatically. On `versionConflict`, reread and recompute;
  on `contextExpired`, find the original target again instead of switching to the focused window; `busy` and `rateLimited` may be retried a bounded number of times;
  `permissionDenied` and `readOnly` need the user.
- Documents open in the background without a mounted editor can be listed and read, but edits and selections may return `busy`.
- Messages have rate and queue limits (5 per 10 seconds per plugin, at most 3 awaiting an answer), and the request stays pending until the user answers.

Protocol 3 also includes the plugin's own editor actions (`editor.actions`), editor language-feature providers (`editor.providers`), and native terminals (`terminal.create`); see [Editor, documents, and previews](../reference/domains/editor.md) and [Open a native Lithe terminal from a plugin](native-terminals.md).

Protocol 3 has three more groups of methods. Only the macOS host implements them, and you should likewise check `getCapabilities.services` before calling. Every one of their permissions requires `backend` 2; the manifest conditions of each are in its contract section:

- Run and debug sessions (permissions `run.start`, `debug.start`): `run/start` and `run/stop` start and stop a host-owned Run session (Run window entry, console, Stop button, exit status), `debug/start` starts a debug session on a debug adapter the plugin itself declares, and the `run/didExit` notification reports that a run ended; see [Plugin backend and plugin functions](../reference/domains/backend.md), [Debug adapters](../reference/domains/debug-adapters.md), and the contract's [Run and debug sessions](../contract/v1/README.md#run-and-debug-sessions-protocol-version-3).
- Own language servers (permission `languageServer.request`): `languageServer/request` and `languageServer/notify` send requests and notifications to a language server the plugin itself declares, `languageServer/setStatus` shows its preparation state in the host's status UI, and server state changes and forwarded notifications arrive through `languageServer/didChangeState` and `languageServer/didReceiveNotification`; see [Languages, language servers, run and test](../reference/domains/languages.md) and the contract's [Language server host methods](../contract/v1/README.md#language-server-host-methods).
- Content providers (permission `documents.contentProvider`): `documents/registerContentProvider` binds a URI scheme to a backend function, and the host calls that function for read-only content when the user opens a document of that scheme; see [Editor, documents, and previews](../reference/domains/editor.md) and the contract's [Content providers](../contract/v1/README.md#content-providers).

### 9. Host failures and error codes

Protocol 3 host failures are typed: each JSON-RPC number maps one-to-one to an `error.data.code`, a number never changes meaning, and some failures carry `error.data.details`.

| `error.data.code` | Number | What a backend usually does |
| --- | --- | --- |
| `plugin.host.unavailable` | `-32601` | The host has no such method; check `getCapabilities` first |
| `plugin.host.invalidParams` | `-32602` | Malformed params; fix the code |
| `plugin.host.failed` | `-32603` | Any other failure; report it to the user |
| `plugin.host.cancelled` | `-32800` | You cancelled it yourself with `$/cancelRequest` |
| `plugin.host.permissionDenied` | `-32010` | The permission is not declared, or the user refused |
| `plugin.host.contextExpired` | `-32011` | Call `listContexts` again, then retry |
| `plugin.host.authorityUnsupported` | `-32012` | The service does not support remote workspaces, and the host will not run it locally instead |
| `plugin.host.versionConflict` | `-32013` | The resource version changed; `details.currentVersion` gives the current version |
| `plugin.host.limitExceeded` | `-32014` | A limit was exceeded, for example more pending requests than `limits.pendingHostRequests` |
| `plugin.host.timeout` | `-32015` | The host timed out |
| `plugin.host.notFound` | `-32016` | The resource does not exist |
| `plugin.host.readOnly` | `-32017` | The resource or workspace is not writable |
| `plugin.host.unsupportedOption` | `-32018` | The host does not know a field in the request; `details.option` names it, so drop it and retry |
| `plugin.host.preconditionFailed` | `-32019` | A required state does not hold, for example the user did not confirm |
| `plugin.host.busy` | `-32020` | The resource is in use; retry later |
| `plugin.host.rateLimited` | `-32021` | Too many requests; wait `details.retryAfterMilliseconds` when given |

`-32000` through `-32009` are reserved for the backend's own failures; host numbers start at `-32010`.

### 10. Protocol 3 only grows: four rules for backends

From protocol 3 on, new host capabilities (new services with their methods and notifications, optional params, result fields, error codes) are added within protocol 3
without raising the protocol version; only changes that would make an existing backend misread something (removing or renaming, changing a field's meaning or type, making an optional param required, changing an error number)
lead to a new protocol version, offered next to the old one. So a plugin that implements protocol 3 once works on every future protocol 3 host, provided it follows these four rules:

1. **Call only methods listed in `getCapabilities.services`.** Any other method gets `-32601` `plugin.host.unavailable`.
2. **Ignore unknown fields and notifications.** Ignore extra fields in results, notifications, `details`, and `limits`, and any unknown `lithe/host/*` notification.
3. **An unsupported new option gets an explicit answer.** If a request carries a field the host does not know, Core answers
   `-32018` `plugin.host.unsupportedOption` with `details: { "option": "<field name>" }` before the host sees the request. The host never silently ignores it;
   drop the option and retry, or tell the user to update Lithe.
4. **Treat unknown numbers as host failures.** A newer host may bring error codes you have not seen; branch on `error.data.code` when you know it, and otherwise treat it as a general failure.

A new service always comes with a new manifest permission. An older host that does not know the permission refuses the whole plugin package at installation with `plugin.permission.unknown`,
so the permission is never silently granted or ignored.

#### Protocol 4: cross-plugin calls

Protocol 4 adds only two things, both for [cross-plugin calls](../contract/v1/README.md#cross-plugin-calls-protocol-version-4); it keeps everything in protocol 3, and the four rules above still apply:

- Two relay methods for the caller: it finds the plugins that export a function with `functions/providers`, then calls one of them with `functions/invoke`.
  Core relays the call, and the caller never reaches the provider's process. Only a backend that negotiated protocol 4 and whose manifest declares at least one call (`pluginFunctions.calls`) may send these requests:
  one on version 3 or earlier gets `-32601` `plugin.host.unavailable`, and one that declares no call gets `-32010` `plugin.host.permissionDenied`.
- `caller` in a relayed call: the `lithe/function/invoke` a provider receives has one more top-level field, `caller`, the calling plugin's ID that Core writes from the calling session;
  its absence means the host started the call. A provider that decides by who is asking reads only this top-level field, never a plugin ID that `params` claims.
  When a provider makes a nested call while serving a relayed call, it puts that call's top-level `operationID` in `parentOperationID`, so Core knows which call chain the nested call belongs to.

What to remember:

- A backend that negotiated only 3 keeps working as before. It cannot call `functions/*`; the functions it exports can still be called through the relay, but it never receives `caller`, so a provider that decides by who is asking must negotiate 4.
- To export or call cross-plugin functions, the manifest declares `pluginFunctions`: `pluginAPIVersion` 3 or later, with `pluginFunctions: 1`, `backend: 2`, and `serverProcess: 1` in `requires`.
  It grants no manifest permission, and declaring an export does not register it: the backend still registers the function with `lithe/function/register`.
- Only the macOS host implements it so far; the three official SDKs still negotiate only up to protocol 3, have no wrapper for `functions/*`, and do not expose `caller`.

### 11. Verification

- Check your implementation against the contract's [`fixtures/backend/function-protocol.json`](../contract/v1/fixtures/backend/function-protocol.json):
  `hostRequests` has request cases such as unknown options, `invocationContexts` has invocation contexts, `hostFailures` has the failure code mapping,
  and `hostNotifications` has notification cases.
- The manifest example is [`fixtures/backend/backend-workspace.plugin.json`](../contract/v1/fixtures/backend/backend-workspace.plugin.json);
  the backend-only command example is [`fixtures/backend/command-backend.plugin.json`](../contract/v1/fixtures/backend/command-backend.plugin.json);
  the backend cases in [`fixtures/manifest/invalid-manifests.json`](../contract/v1/fixtures/manifest/invalid-manifests.json) are the forms that are refused.
