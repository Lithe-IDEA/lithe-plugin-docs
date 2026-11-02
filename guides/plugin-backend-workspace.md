# 插件后端：工作区上下文与宿主服务（backend 2、协议 3，开发中）

这篇指南面向写插件后端（`backend`）的作者。目标读者已经能让自己的网页调用后端函数（`backend` 1），
现在想让后端在没有页面时也能启动、知道用户打开了哪些工作区、响应命令，并按需调用宿主能力。

字段与错误码的权威定义在[契约](../contract/v1/README.md#backend-2-workspace-contexts)，这里只讲怎么用、为什么这样设计。

> **宿主支持。** `requires.backend: 2` 和插件函数协议 3 还在 Lithe `preview` 分支上开发，尚未进入稳定版。
> 稳定版宿主只提供 `backend: [1]` 时会以 `plugin.api.unsupported` 拒绝 `backend` 2 的插件，所以老版本 Lithe
> 不会启动一个期待工作区上下文的进程。官方 SDK 目前实现到协议 2，用协议 3 需要自己处理下文的请求和通知。

## 1. 三个版本号各管什么

| 写在哪里 | 值 | 决定什么 |
| --- | --- | --- |
| `requires.backend` | `1` 或 `2` | 后端进程**什么时候启动**、能不能声明 `workspace.metadata` 和 `onWorkspaceOpen` |
| `initialize` 应答的 `litheFunctions.version` | `1`、`2` 或 `3` | 进程启动后**线路协议**的规则：`2` 起有宿主服务（凭据），`3` 起有能力发现、工作区上下文、宿主通知和带类型的失败码 |
| `getCapabilities.services` | 运行时查询结果 | 本次会话**实际能调用**哪些宿主方法、会收到哪些通知 |

清单只做声明，宿主读清单决定装不装、什么时候启动；进程起来以后能做什么，靠协商和 `getCapabilities` 判断。

## 2. 最小清单

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

## 3. 启动时机

- `onWorkspaceOpen` 不会拖慢打开工作区。宿主在窗口显示之后逐个启动这些后端；`initialize` 超时的后端会被停止，
  并像其他启动失败一样报告，不会挡住工作区。
- 后端在 Lithe 所在的机器上运行，远程工作区也一样。
- 激活只看插件是否已安装、启用、通过审查。Lithe 目前没有工作区信任状态；会运行或修改东西的操作，由宿主逐次询问用户。

## 4. 协商协议 3

宿主在 `initialize` 里提供 `capabilities.experimental.litheFunctions: { "versions": [1, 2, 3] }`。后端从中选自己实现的最高版本作答：

```json
"experimental": { "litheFunctions": { "version": 3, "functions": [{ "id": "orders.refresh" }] } }
```

只回答你真正实现了的版本。回答 `2` 的后端继续按协议 2 工作，永远看不到协议 3 新增的通知、上下文字段和失败码。

## 5. 先问宿主能做什么

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

## 6. 工作区上下文

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

## 7. 宿主通知

宿主用 JSON-RPC 通知告诉后端发生了变化，后端不应答：

| 通知 | 参数 | 什么时候来 |
| --- | --- | --- |
| `lithe/host/workspace/didChangeContexts` | `{ revision }` | 有上下文打开、关闭或变化；收到后重新 `listContexts` |
| `lithe/host/commands/didRun` | `{ commandID, context? }` | 用户运行了插件自己的命令。后端没在运行且声明了 `onCommand:<commandID>` 时，宿主先启动它，就绪后再送达 |
| `lithe/host/subscriptions/notify` | `{ subscriptionID, sequence, event }` | 某个订阅的事件。`sequence` 每个订阅从 1 起逐一递增，可以据此发现丢失；用 `lithe/host/subscriptions/dispose` 取消订阅 |

后端晚启动时不会补发之前的 `didChangeContexts`，启动后自己调一次 `listContexts`。

`didRun` 只是事件，不代表命令要做的事已经成功；宿主也不会为了送达它而打开一个空工具窗口。

### 纯后端命令（`ui` 4）

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
[`fixtures/command-backend.plugin.json`](../contract/v1/fixtures/command-backend.plugin.json)。

## 8. 文档、文件与窗口方法

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

## 9. 宿主失败与错误码

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

## 10. 协议 3 只增不改：后端要遵守的四条

从协议 3 起，宿主再开放新能力（新服务及其方法和通知、可选参数、结果字段、错误码）都在协议 3 内新增，
不再升协议版本；只有会让已有后端读错的改动（删除或改名、改字段含义或类型、把可选参数改成必填、改错误码号码）
才会出协议 4。这样插件只要实现一次协议 3，就能在今后所有协议 3 宿主上工作，前提是遵守下面四条：

1. **只调用 `getCapabilities.services` 列出的方法。** 其他方法会得到 `-32601` `plugin.host.unavailable`。
2. **忽略不认识的字段和通知。** 结果、通知、`details`、`limits` 里多出来的字段，以及不认识的 `lithe/host/*` 通知，都直接忽略。
3. **新选项不被支持时会得到明确回答。** 请求里带了宿主不认识的字段，Core 在宿主看到请求之前回
   `-32018` `plugin.host.unsupportedOption`，`details: { "option": "<字段名>" }`。宿主不会悄悄忽略它，
   你可以去掉这个选项重试，或告诉用户需要更新 Lithe。
4. **不认识的号码按宿主失败处理。** 新宿主可能带来你没见过的错误码；认识 `error.data.code` 时按它分支，否则当作一般失败。

新服务总是配一个新的清单权限。旧宿主不认识这个权限时，会在安装时以 `plugin.permission.unknown` 拒绝整个插件包，
权限不会被悄悄授予或忽略。

## 11. 验证

- 用契约里的 [`fixtures/function-protocol.json`](../contract/v1/fixtures/function-protocol.json) 核对你的实现：
  `hostRequests` 有不认识的选项等请求用例，`invocationContexts` 有调用上下文，`hostFailures` 有失败码映射，
  `hostNotifications` 有通知用例。
- 清单示例是 [`fixtures/backend-workspace.plugin.json`](../contract/v1/fixtures/backend-workspace.plugin.json)；
  纯后端命令的示例是 [`fixtures/command-backend.plugin.json`](../contract/v1/fixtures/command-backend.plugin.json)；
  `fixtures/invalid-manifests.json` 里的 backend 用例是会被拒绝的写法。
