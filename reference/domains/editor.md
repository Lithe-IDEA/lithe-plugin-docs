# 编辑器、文档与预览 / Editor, documents, and previews

[中文](#中文) · [English](#english)

## 中文

插件后端通过插件函数协议 3 读取打开的文档和选区、做版本化编辑、打开和定位文件、打开 HTML 预览或外部浏览器，并在编辑器里注册自己的动作和语言功能 Provider（定义、悬浮、补全等），还能把没有对应文件的文档（例如反编译出的库类源码）按 URI scheme 交给宿主打开。这些都是宿主方法，由 Rust Core 按权限校验、macOS 宿主执行；插件不直接操作编辑器。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | 没有专用字段；用 `backend` 加下面的权限 |
| 能力版本 | `backend` 2，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheFunctions` 3，见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | API-09、10、12、15、16、25、27，见 [API 总表](../api-catalog.md#现在能用的-api) |
| 权限 | `documents.read`、`editor.edit`、`editor.open`、`preview.open`、`browser.open`、`editor.actions`（同时需要 `documents.read`）、`editor.providers`（同时需要 `documents.read`）、`documents.contentProvider`（只授予 `documents/registerContentProvider`，不授予读写文档），见[已公开的宿主方法](../api-catalog.md#已公开的宿主方法) |
| 激活事件 | 与后端相同：`onWorkspaceOpen`、`onCommand:<id>` 或网页调用 |
| Windows | 不支持；Windows 没有插件后端宿主 |

### 方法对照

| 方法 | 权限 | API | 说明 |
| --- | --- | --- | --- |
| `documents/list`、`documents/getSnapshot`、`editor/getActive`、`editor/getSelections` | `documents.read` | API-09 | 包含未保存状态、版本、编码、换行符；快照最多 128 KiB |
| `editor/applyEdits`、`editor/replaceSelections`、`editor/insertText` | `editor.edit` | API-10 | 校验文档版本；输入法组字中和只读文档会被拒绝；一批编辑是一次撤销 |
| `editor/open`、`editor/reveal` | `editor.open` | API-15 | 返回 `documentID` 和版本；传入范围时只把光标放到起点 |
| `preview/openHTML` | `preview.open` | API-16 | `editorAndPreview` 或 `previewOnly` |
| `browser/openExternal` | `browser.open` | API-16 | HTTP(S) 地址或已保存的 HTML |
| `editor/registerActions` | `editor.actions` + `documents.read` | API-27 | 先用 `getCapabilities` 发现，再按工作区上下文注册；动作绑定普通函数和文件范围，不依赖 LSP、构建或运行。第 2 版支持多个图标、最多 4 个 `alternatives` 和点击结果 `reveal`，可用图标由宿主在 `services.editor.options.actionIcons` 中声明 |
| `editor/registerProviders` | `editor.providers` + `documents.read` | API-25 | 先用 `getCapabilities` 发现方法和 `services.editor.options.providerTypes`，再按工作区上下文注册；把 `definition`、`hover`、`completion`、`codeLens`、`foldingRange`、`semanticTokens`、`documentLink` 这 7 种类型绑定到普通函数，不在 `providerTypes` 中的类型返回 `unsupportedOption`；语言服务器的结果优先，插件结果只补充；预算由宿主决定 |

所有方法都要带 `context: {contextID, generation}`；除 `documents/registerContentProvider` 外，远程工作区上下文返回 `plugin.host.authorityUnsupported`。`documents/registerContentProvider` 见下一节。

### 内容 Provider（API-12）

语言服务器可能给出没有文件的文档（例如库类的反编译源码）。插件把这些文档当作一个 URI scheme 提供，用 `documents/registerContentProvider`，权限 `documents.contentProvider`（需要 `backend` 2）。

| 方法 | 权限 | API | 参数 | 结果 |
| --- | --- | --- | --- | --- |
| `documents/registerContentProvider` | `documents.contentProvider` | API-12 | `{context, scheme, function}` | `{subscriptionID}` |

- 注册只记录绑定，不会调用后端；`function` 必须已经在当前会话里注册。用已有的 `subscriptions/dispose` 撤销。
- `scheme` 是 1 到 64 个小写 ASCII 字母、数字、`+`、`-` 或 `.`，以字母开头。宿主或平台自己解析的 scheme 会被拒绝：`about`、`blob`、`data`、`file`、`ftp`、`http`、`https`、`javascript`、`lithe`、`mailto`、`untitled`、`ws`、`wss`。scheme 不合法、保留或 `function` 不是函数 ID，返回 `plugin.host.invalidParams`。
- 每个工作区窗口里每个 scheme 只能有一个 Provider；再注册同一个 scheme 返回 `plugin.host.preconditionFailed`。
- 打开 URI 带这个 scheme 的文档时，宿主用工作区 `context` 和 `{uri}` 通过 `lithe/function/invoke` 调用该函数。函数返回字符串或 `{content: string}`，最多 1 MiB，文档以只读方式打开；其他返回、失败或超时（宿主等 30 秒）都会向用户显示错误，不打开任何文档。
- Provider 是文本的唯一来源：文档关闭后宿主不保留副本，重新打开时再问一次。会话结束或上下文关闭会撤销 Provider。
- 只有 `documents.contentProvider` 授予这个方法；聚合服务名 `documents` 和其他授权都不会放行，返回 `plugin.host.permissionDenied`（`-32010`）。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | 文档、编辑、打开、预览、浏览器方法的参数、限制和失败码 |
| [Independent editor actions (protocol version 3)](../../contract/v1/README.md#independent-editor-actions-protocol-version-3) | `editor/registerActions` 的注册、范围、调用和撤回，以及第 2 版的图标、`alternatives` 和点击结果 |
| [Editor language-feature providers (protocol version 3)](../../contract/v1/README.md#editor-language-feature-providers-protocol-version-3) | `editor/registerProviders` 的注册、选择器、结果形状、预算和合并规则 |
| [Content providers](../../contract/v1/README.md#content-providers) | `documents/registerContentProvider` 的 scheme 规则、调用、只读文档和撤回 |
| [Workspace contexts and capability discovery (protocol version 3)](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) | `contextID`/`generation` 与 `getCapabilities` |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | 结果与错误码 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`editor/editor-actions.plugin.json`](../../contract/v1/fixtures/editor/editor-actions.plugin.json) | `backend` 2 加 `editor.actions`、`documents.read`：`onWorkspaceOpen` 启动后注册编辑器动作 |
| [`editor/editor-actions-v2.json`](../../contract/v1/fixtures/editor/editor-actions-v2.json) | 第 2 版动作的 Core 用例：宿主声明的图标和源码上限、`alternatives`、点击结果 `reveal`，以及整个结果被拒绝的情形 |
| [`editor/editor-providers.plugin.json`](../../contract/v1/fixtures/editor/editor-providers.plugin.json) | `backend` 2 加 `editor.providers`、`documents.read`：`onWorkspaceOpen` 启动后注册编辑器语言功能 Provider |
| [`editor/editor-providers.json`](../../contract/v1/fixtures/editor/editor-providers.json) | 注册与结果的 Core 用例：选择器和类型校验（`unsupportedOption`）、7 种类型的结果形状与被丢弃的结果 |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `ui` 4 命令读取文档并编辑 |
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | `editor/applyEdits`、`editor/registerActions` 的标准请求与回复，以及 `versionConflict`、`readOnly` 等宿主失败码；还有 `documents/registerContentProvider` 的用例：需要 `documents.contentProvider` 授权、聚合名 `documents` 不够、保留 scheme、大写或数字开头的 scheme、非法函数名、缺少 `context` |

### 相关指南

- [插件后端：工作区上下文与宿主服务](../../guides/plugin-backend-workspace.md)

---

## English

Through plugin-function protocol 3, a plugin backend reads open documents and selections, applies versioned edits, opens and reveals files, opens an HTML preview or an external browser, and registers its own editor actions and language-feature providers (definition, hover, completion, and so on). It can also serve documents that have no file (such as the decompiled source of a library class) to the host as a URI scheme. These are host methods: Rust Core checks permissions and the macOS host executes them. The plugin never drives the editor directly.

| Item | Value |
| --- | --- |
| Manifest field | None of its own; use `backend` with the permissions below |
| Capability versions | `backend` 2; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheFunctions` 3; see [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | API-09, 10, 12, 15, 16, 25, 27; see the [API catalog](../api-catalog.md#apis-usable-now) |
| Permissions | `documents.read`, `editor.edit`, `editor.open`, `preview.open`, `browser.open`, `editor.actions` (also needs `documents.read`), `editor.providers` (also needs `documents.read`), `documents.contentProvider` (grants only `documents/registerContentProvider`, not reading or editing documents); see [public host methods](../api-catalog.md#public-host-methods) |
| Activation | As for the backend: `onWorkspaceOpen`, `onCommand:<id>`, or a page call |
| Windows | Not supported; Windows has no plugin backend host |

### Methods compared

| Method | Permission | API | Notes |
| --- | --- | --- | --- |
| `documents/list`, `documents/getSnapshot`, `editor/getActive`, `editor/getSelections` | `documents.read` | API-09 | Includes unsaved state, version, encoding, line endings; snapshots up to 128 KiB |
| `editor/applyEdits`, `editor/replaceSelections`, `editor/insertText` | `editor.edit` | API-10 | Checks the document version; refused during IME composition and on read-only documents; one batch is one undo step |
| `editor/open`, `editor/reveal` | `editor.open` | API-15 | Returns `documentID` and version; a range only places the caret at its start |
| `preview/openHTML` | `preview.open` | API-16 | `editorAndPreview` or `previewOnly` |
| `browser/openExternal` | `browser.open` | API-16 | An HTTP(S) URL or saved HTML |
| `editor/registerActions` | `editor.actions` + `documents.read` | API-27 | Discover it with `getCapabilities`, then register per workspace context; actions bind ordinary functions and file scopes, independent of LSP, build, or run. Version 2 adds multiple icons, up to 4 `alternatives`, and a click result `reveal`; the host declares the usable icons in `services.editor.options.actionIcons` |
| `editor/registerProviders` | `editor.providers` + `documents.read` | API-25 | Discover the method and `services.editor.options.providerTypes` with `getCapabilities`, then register per workspace context; binds the 7 types `definition`, `hover`, `completion`, `codeLens`, `foldingRange`, `semanticTokens`, and `documentLink` to ordinary functions, and a type outside `providerTypes` returns `unsupportedOption`; language-server results come first and plugin results only add to them; the host sets the budgets |

Every method takes `context: {contextID, generation}`; except for `documents/registerContentProvider`, a remote workspace context returns `plugin.host.authorityUnsupported`. `documents/registerContentProvider` is described in the next section.

### Content providers (API-12)

A language server may hand out documents that have no file (such as the decompiled source of a library class). The plugin serves them as a URI scheme with `documents/registerContentProvider`, granted by `documents.contentProvider` (needs `backend` 2).

| Method | Permission | API | Params | Result |
| --- | --- | --- | --- | --- |
| `documents/registerContentProvider` | `documents.contentProvider` | API-12 | `{context, scheme, function}` | `{subscriptionID}` |

- Registration only records the binding and never calls the backend; `function` must already be registered on the session. Withdraw it with the existing `subscriptions/dispose`.
- `scheme` is 1 to 64 lowercase ASCII letters, digits, `+`, `-`, or `.`, and starts with a letter. The schemes the host or the platform resolves are refused: `about`, `blob`, `data`, `file`, `ftp`, `http`, `https`, `javascript`, `lithe`, `mailto`, `untitled`, `ws`, and `wss`. A malformed or reserved scheme, or a `function` that is not a function ID, returns `plugin.host.invalidParams`.
- A scheme has one provider per workspace window; registering it again returns `plugin.host.preconditionFailed`.
- When a document whose URI has that scheme is opened, the host calls the function through `lithe/function/invoke` with the workspace `context` and `{uri}`. The function answers a string, or `{content: string}`, of at most 1 MiB, and the document opens read-only. Any other answer, a failure, or a timeout (the host waits 30 seconds) shows the user an error and opens nothing.
- The provider is the only source of the text: the host keeps no copy once the document closes and asks again when it is reopened. Ending the session or closing the context withdraws the provider.
- Only `documents.contentProvider` grants this method; the aggregate service name `documents` and other grants do not, and return `plugin.host.permissionDenied` (`-32010`).

### Contract sections

| Section | Content |
| --- | --- |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | Parameters, limits, and failure codes for document, edit, open, preview, and browser methods |
| [Independent editor actions (protocol version 3)](../../contract/v1/README.md#independent-editor-actions-protocol-version-3) | Registering, scoping, invoking, and withdrawing `editor/registerActions`, plus version 2 icons, `alternatives`, and click results |
| [Editor language-feature providers (protocol version 3)](../../contract/v1/README.md#editor-language-feature-providers-protocol-version-3) | Registering `editor/registerProviders`, selectors, result shapes, budgets, and merge rules |
| [Content providers](../../contract/v1/README.md#content-providers) | Scheme rules, invoking, read-only documents, and withdrawing for `documents/registerContentProvider` |
| [Workspace contexts and capability discovery (protocol version 3)](../../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) | `contextID`/`generation` and `getCapabilities` |
| [Outcomes and error codes](../../contract/v1/README.md#outcomes-and-error-codes) | Outcomes and error codes |

### Examples and cases

| File | Content |
| --- | --- |
| [`editor/editor-actions.plugin.json`](../../contract/v1/fixtures/editor/editor-actions.plugin.json) | `backend` 2 with `editor.actions` and `documents.read`: registers editor actions after `onWorkspaceOpen` |
| [`editor/editor-actions-v2.json`](../../contract/v1/fixtures/editor/editor-actions-v2.json) | Core cases for version 2 actions: the host-declared icons and source limit, `alternatives`, click-result `reveal`, and results refused whole |
| [`editor/editor-providers.plugin.json`](../../contract/v1/fixtures/editor/editor-providers.plugin.json) | `backend` 2 with `editor.providers` and `documents.read`: registers editor language-feature providers after `onWorkspaceOpen` |
| [`editor/editor-providers.json`](../../contract/v1/fixtures/editor/editor-providers.json) | Core cases for registrations and results: selector and type validation (`unsupportedOption`), result shapes for the 7 types, and dropped results |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | A `ui` 4 command that reads and edits documents |
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | Canonical requests and replies for `editor/applyEdits` and `editor/registerActions`, plus host failure codes such as `versionConflict` and `readOnly`; also the `documents/registerContentProvider` cases: it needs the `documents.contentProvider` grant, the aggregate `documents` name is not enough, reserved schemes, an uppercase or digit-first scheme, a malformed function name, and a missing `context` |

### Related guides

- [Plugin backend: workspace contexts and host services](../../guides/plugin-backend-workspace.md)
