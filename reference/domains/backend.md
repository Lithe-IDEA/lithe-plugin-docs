# 插件后端与插件函数 / Plugin backend and plugin functions

[中文](#中文) · [English](#english)

## 中文

插件后端是插件自己的长驻进程，用来运行网页做不到的事：数据库驱动、网络客户端、文件格式解析、调用宿主服务。它只服务插件自己，不属于任何构建系统、语言或工作区，也不回答 `lithe/build/*`、VCS 等领域请求。后端与宿主之间说插件函数协议（`litheFunctions`），从版本 2 起可以按权限请求宿主服务。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `backend`（与 `serverProcess` 的启动字段相同） |
| 能力版本 | `backend` 1–2、`serverProcess` 1；`ui` 4 的后端命令需要 `backend` 2，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheFunctions` 1–3、`litheLogging` 1，见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | API-01、04、05、06、07、08、20、21、22、24、37、40、41，见 [API 总表](../api-catalog.md#现在能用的-api) |
| 权限 | `backend.launch`；宿主服务按方法另加权限（`credentials.store`、`workspace.metadata`、`workspace.files.read`、`workspace.files.watch`、`workspace.search`、`terminal.create` 等），见[已公开的宿主方法](../api-catalog.md#已公开的宿主方法) |
| 激活事件 | `backend` 1：插件网页第一次调用 `functions.*` 时启动。`backend` 2 另可用 `onCommand:<id>` 或可选的 `onWorkspaceOpen` |
| Windows | 没有插件后端宿主，不声明 `backend` |
| 官方 SDK | [Node](../../sdk/node/README.md)、[Java](../../sdk/java/README.md)、[Rust](../../sdk/rust/README.md)；各 SDK 支持的协议版本见[宿主与官方 SDK 对照](../capabilities-and-versions.md#宿主与官方-sdk-对照) |

### 版本对照

| 版本 | 新增 | 契约章节 |
| --- | --- | --- |
| `backend` 1 | 每个插件一个长驻进程，只有插件自己的网页能调用；没有网页工具窗口的 `backend` 1 会被拒绝 | [Plugin backend](../../contract/v1/README.md#plugin-backend) |
| `backend` 2 | 可以不依赖网页启动（网页、`ui.commands` 或 `onWorkspaceOpen` 至少一种）；所有工作区共享一个进程，每次调用显式带工作区上下文 | [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) |
| `litheFunctions` 1 | 注册、撤回、调用、结果、进度、取消；不提供宿主服务 | [Plugin functions](../../contract/v1/README.md#plugin-functions) |
| `litheFunctions` 2 | 宿主服务：凭据 `credentials/get`、`set`、`delete` | [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) |
| `litheFunctions` 3 | 工作区上下文与能力发现、宿主通知、公共文件/文档/窗口方法、独立编辑器动作、插件自有原生终端 | [Protocol 3](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) |
| `litheLogging` 1 | 结构化诊断通知 `lithe/log`；当前官方 SDK 要求宿主提供 | [Structured plugin diagnostics](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) |

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Plugin backend](../../contract/v1/README.md#plugin-backend) | 后端声明、状态目录、启动与停止时机、按平台发布 |
| [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) | 启动方式、`onWorkspaceOpen` 不阻塞工作区、`workspace.metadata` |
| [Plugin functions](../../contract/v1/README.md#plugin-functions) | 协议总览 |
| [Capability ownership](../../contract/v1/README.md#capability-ownership) | 插件函数不能代替领域契约 |
| [Declarative or backend implementation](../../contract/v1/README.md#declarative-or-backend-implementation) | 什么时候用声明式数据、什么时候写后端 |
| [Negotiation](../../contract/v1/README.md#negotiation) | `initialize` 里的版本协商 |
| [References and IDs](../../contract/v1/README.md#references-and-ids) | 函数引用和 ID 规则 |
| [Registration and withdrawal](../../contract/v1/README.md#registration-and-withdrawal) | 注册和撤回函数 |
| [Invocation](../../contract/v1/README.md#invocation)、[Progress](../../contract/v1/README.md#progress) | 调用、结果、进度、取消 |
| [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) | 凭据服务 |
| [Workspace contexts and capability discovery (protocol version 3)](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) | `getCapabilities`、上下文 `contextID`/`generation`、宿主失败码 |
| [Host notifications (protocol version 3)](../../contract/v1/README.md#host-notifications-protocol-version-3) | `workspace/didChangeContexts`、`subscriptions/notify`、`commands/didRun` |
| [Extending version 3](../../contract/v1/README.md#extending-version-3) | 协议 3 如何不升版本地增加方法 |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | 文件读取、监听、搜索与窗口消息（文档和编辑器方法见[编辑器](editor.md)） |
| [Plugin-owned native terminals (function protocol 3)](../../contract/v1/README.md#plugin-owned-native-terminals-function-protocol-3)、[Close, retained context and reopen](../../contract/v1/README.md#close-retained-context-and-reopen) | 插件自有原生终端 |
| [Structured plugin diagnostics (logging protocol 1)](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) | `lithe/log` 的字段和限制 |
| [Compatibility](../../contract/v1/README.md#compatibility)、[Official backend SDK](../../contract/v1/README.md#official-backend-sdk) | 版本兼容规则与官方 SDK |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | 结果与错误码 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`backend/backend.plugin.json`](../../contract/v1/fixtures/backend/backend.plugin.json) | `backend` 1：网页工具窗口调用后端，使用凭据服务 |
| [`backend/backend-workspace.plugin.json`](../../contract/v1/fixtures/backend/backend-workspace.plugin.json) | `backend` 2：`onWorkspaceOpen` 启动，`workspace.metadata` |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `backend` 2 加 `ui` 4：不打开工具窗口的命令，读文档并编辑 |
| [`backend/terminal-backend.plugin.json`](../../contract/v1/fixtures/backend/terminal-backend.plugin.json) | `backend` 2：插件自有原生终端的最小清单 |
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | 插件函数协议的标准报文：协商、注册、调用、结果、进度、42 个宿主请求、18 个宿主失败、10 个宿主通知 |
| [`backend/plugin-logging.json`](../../contract/v1/fixtures/backend/plugin-logging.json) | `litheLogging` 1：提供与回答、通知、拒绝用例、级别 |

后端相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。

### 相关指南

- [插件后端：工作区上下文与宿主服务](../../guides/plugin-backend-workspace.md)
- [从插件打开原生终端](../../guides/native-terminals.md)
- [插件接入统一日志](../../guides/plugin-logging.md)

---

## English

A plugin backend is the plugin's own long-lived process. It runs what a web page cannot: database drivers, network clients, file-format parsers, calls to host services. It serves only its plugin, belongs to no build system, language, or workspace, and never answers domain requests such as `lithe/build/*` or VCS. Backend and host speak the plugin-function protocol (`litheFunctions`); from version 2 on, the backend may request host services its permissions grant.

| Item | Value |
| --- | --- |
| Manifest field | `backend` (the same launch fields as `serverProcess`) |
| Capability versions | `backend` 1–2, `serverProcess` 1; `ui` 4 backend commands need `backend` 2; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheFunctions` 1–3, `litheLogging` 1; see [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | API-01, 04, 05, 06, 07, 08, 20, 21, 22, 24, 37, 40, 41; see the [API catalog](../api-catalog.md#apis-usable-now) |
| Permissions | `backend.launch`; host services need their own permission per method (`credentials.store`, `workspace.metadata`, `workspace.files.read`, `workspace.files.watch`, `workspace.search`, `terminal.create`, …); see [public host methods](../api-catalog.md#public-host-methods) |
| Activation | `backend` 1 starts on the first `functions.*` call from the plugin's page. `backend` 2 may also start on `onCommand:<id>` or the optional `onWorkspaceOpen` |
| Windows | No plugin backend host; `backend` is not advertised |
| Official SDKs | [Node](../../sdk/node/README.md), [Java](../../sdk/java/README.md), [Rust](../../sdk/rust/README.md); protocol versions per SDK are in [Host and official SDKs](../capabilities-and-versions.md#host-and-official-sdks) |

### Versions compared

| Version | Adds | Contract section |
| --- | --- | --- |
| `backend` 1 | One long-lived process per plugin, called only by the plugin's own web pages; a `backend` 1 without a web tool window is refused | [Plugin backend](../../contract/v1/README.md#plugin-backend) |
| `backend` 2 | Starts without a page (a web window, `ui.commands`, or `onWorkspaceOpen`, at least one); one process shared by every workspace, each call names its workspace context | [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) |
| `litheFunctions` 1 | Registration, withdrawal, invocation, results, progress, cancellation; no host services | [Plugin functions](../../contract/v1/README.md#plugin-functions) |
| `litheFunctions` 2 | Host services: credentials `credentials/get`, `set`, `delete` | [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) |
| `litheFunctions` 3 | Workspace contexts and capability discovery, host notifications, public file/document/window methods, independent editor actions, plugin-owned native terminals | [Protocol 3](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) |
| `litheLogging` 1 | Structured diagnostics notification `lithe/log`; current official SDKs require the host to offer it | [Structured plugin diagnostics](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) |

### Contract sections

| Section | Content |
| --- | --- |
| [Plugin backend](../../contract/v1/README.md#plugin-backend) | Declaration, state directory, when it starts and stops, per-platform archives |
| [Backend 2: workspace contexts](../../contract/v1/README.md#backend-2-workspace-contexts) | Start routes, `onWorkspaceOpen` never blocks the workspace, `workspace.metadata` |
| [Plugin functions](../../contract/v1/README.md#plugin-functions) | Protocol overview |
| [Capability ownership](../../contract/v1/README.md#capability-ownership) | Plugin functions never replace a domain contract |
| [Declarative or backend implementation](../../contract/v1/README.md#declarative-or-backend-implementation) | When to use declarative data and when to write a backend |
| [Negotiation](../../contract/v1/README.md#negotiation) | Version negotiation in `initialize` |
| [References and IDs](../../contract/v1/README.md#references-and-ids) | Function references and ID rules |
| [Registration and withdrawal](../../contract/v1/README.md#registration-and-withdrawal) | Registering and withdrawing functions |
| [Invocation](../../contract/v1/README.md#invocation), [Progress](../../contract/v1/README.md#progress) | Calls, results, progress, cancellation |
| [Host services (protocol version 2)](../../contract/v1/README.md#host-services-protocol-version-2) | Credentials service |
| [Workspace contexts and capability discovery (protocol version 3)](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) | `getCapabilities`, `contextID`/`generation` contexts, host failure codes |
| [Host notifications (protocol version 3)](../../contract/v1/README.md#host-notifications-protocol-version-3) | `workspace/didChangeContexts`, `subscriptions/notify`, `commands/didRun` |
| [Extending version 3](../../contract/v1/README.md#extending-version-3) | How protocol 3 gains methods without a new version |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | File reads, watches, search, and window messages (document and editor methods are on [Editor](editor.md)) |
| [Plugin-owned native terminals (function protocol 3)](../../contract/v1/README.md#plugin-owned-native-terminals-function-protocol-3), [Close, retained context and reopen](../../contract/v1/README.md#close-retained-context-and-reopen) | Plugin-owned native terminals |
| [Structured plugin diagnostics (logging protocol 1)](../../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) | `lithe/log` fields and limits |
| [Compatibility](../../contract/v1/README.md#compatibility), [Official backend SDK](../../contract/v1/README.md#official-backend-sdk) | Compatibility rules and the official SDKs |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | Outcomes and error codes |

### Examples and cases

| File | Content |
| --- | --- |
| [`backend/backend.plugin.json`](../../contract/v1/fixtures/backend/backend.plugin.json) | `backend` 1: a web tool window calls the backend, which uses the credentials service |
| [`backend/backend-workspace.plugin.json`](../../contract/v1/fixtures/backend/backend-workspace.plugin.json) | `backend` 2: starts on `onWorkspaceOpen`, uses `workspace.metadata` |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `backend` 2 with `ui` 4: a command without a tool window that reads and edits documents |
| [`backend/terminal-backend.plugin.json`](../../contract/v1/fixtures/backend/terminal-backend.plugin.json) | `backend` 2: the smallest manifest for plugin-owned native terminals |
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | Canonical plugin-function messages: negotiation, registration, invocation, results, progress, 42 host requests, 18 host failures, 10 host notifications |
| [`backend/plugin-logging.json`](../../contract/v1/fixtures/backend/plugin-logging.json) | `litheLogging` 1: offer and answer, notifications, rejections, levels |

Backend rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json).

### Related guides

- [Plugin backend: workspace contexts and host services](../../guides/plugin-backend-workspace.md)
- [Open a native Lithe terminal from a plugin](../../guides/native-terminals.md)
- [Plugin logging](../../guides/plugin-logging.md)
