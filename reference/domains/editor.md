# 编辑器、文档与预览 / Editor, documents, and previews

[中文](#中文) · [English](#english)

## 中文

插件后端通过插件函数协议 3 读取打开的文档和选区、做版本化编辑、打开和定位文件、打开 HTML 预览或外部浏览器，并在编辑器里注册自己的动作和语言功能 Provider（定义、悬浮、补全等）。这些都是宿主方法，由 Rust Core 按权限校验、macOS 宿主执行；插件不直接操作编辑器。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | 没有专用字段；用 `backend` 加下面的权限 |
| 能力版本 | `backend` 2，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheFunctions` 3，见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | API-09、10、15、16、25、27，见 [API 总表](../api-catalog.md#现在能用的-api) |
| 权限 | `documents.read`、`editor.edit`、`editor.open`、`preview.open`、`browser.open`、`editor.actions`（同时需要 `documents.read`）、`editor.providers`（同时需要 `documents.read`），见[已公开的宿主方法](../api-catalog.md#已公开的宿主方法) |
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

所有方法都要带 `context: {contextID, generation}`；远程工作区上下文返回 `plugin.host.authorityUnsupported`。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | 文档、编辑、打开、预览、浏览器方法的参数、限制和失败码 |
| [Independent editor actions (protocol version 3)](../../contract/v1/README.md#independent-editor-actions-protocol-version-3) | `editor/registerActions` 的注册、范围、调用和撤回，以及第 2 版的图标、`alternatives` 和点击结果 |
| [Editor language-feature providers (protocol version 3)](../../contract/v1/README.md#editor-language-feature-providers-protocol-version-3) | `editor/registerProviders` 的注册、选择器、结果形状、预算和合并规则 |
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
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | `editor/applyEdits`、`editor/registerActions` 的标准请求与回复，以及 `versionConflict`、`readOnly` 等宿主失败码 |

### 相关指南

- [插件后端：工作区上下文与宿主服务](../../guides/plugin-backend-workspace.md)

---

## English

Through plugin-function protocol 3, a plugin backend reads open documents and selections, applies versioned edits, opens and reveals files, opens an HTML preview or an external browser, and registers its own editor actions and language-feature providers (definition, hover, completion, and so on). These are host methods: Rust Core checks permissions and the macOS host executes them. The plugin never drives the editor directly.

| Item | Value |
| --- | --- |
| Manifest field | None of its own; use `backend` with the permissions below |
| Capability versions | `backend` 2; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheFunctions` 3; see [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | API-09, 10, 15, 16, 25, 27; see the [API catalog](../api-catalog.md#apis-usable-now) |
| Permissions | `documents.read`, `editor.edit`, `editor.open`, `preview.open`, `browser.open`, `editor.actions` (also needs `documents.read`), `editor.providers` (also needs `documents.read`); see [public host methods](../api-catalog.md#public-host-methods) |
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

Every method takes `context: {contextID, generation}`; a remote workspace context returns `plugin.host.authorityUnsupported`.

### Contract sections

| Section | Content |
| --- | --- |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | Parameters, limits, and failure codes for document, edit, open, preview, and browser methods |
| [Independent editor actions (protocol version 3)](../../contract/v1/README.md#independent-editor-actions-protocol-version-3) | Registering, scoping, invoking, and withdrawing `editor/registerActions`, plus version 2 icons, `alternatives`, and click results |
| [Editor language-feature providers (protocol version 3)](../../contract/v1/README.md#editor-language-feature-providers-protocol-version-3) | Registering `editor/registerProviders`, selectors, result shapes, budgets, and merge rules |
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
| [`backend/function-protocol.json`](../../contract/v1/fixtures/backend/function-protocol.json) | Canonical requests and replies for `editor/applyEdits` and `editor/registerActions`, plus host failure codes such as `versionConflict` and `readOnly` |

### Related guides

- [Plugin backend: workspace contexts and host services](../../guides/plugin-backend-workspace.md)
