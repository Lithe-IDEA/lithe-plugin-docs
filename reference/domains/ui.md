# 界面：工具窗口、命令、网页与原生内容 / UI: tool windows, commands, web and native content

[中文](#中文) · [English](#english)

## 中文

插件可以在工作台里放工具窗口和命令。窗口的外壳（停靠、标题、活动栏、顺序、布局、主题、生命周期）始终归宿主；插件只提供窗口内容：纯文本、包内网页，或由宿主绘制的原生树、列表、表单。命令可以打开插件自己的窗口，从 `ui` 4 起也可以只通知后端、不打开窗口。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `ui.toolWindows`、`ui.commands`（可选 `icon`） |
| 能力版本 | `ui` 1–4；原生内容另有 [native schema](../../contract/v1/native-ui.schema.json) 1。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | 网页通过 `functions.*` 调用[插件后端](backend.md)；`ui` 4 命令通过 `litheFunctions` 3 的 `commands/didRun` 通知后端 |
| API 编号 | API-02、03、13、14、17，见 [API 总表](../api-catalog.md#现在能用的-api) |
| 权限 | `ui.contribute`；网页加 `ui.web`；`ui` 4 命令加 `backend.launch`；后端弹出消息用 `window.messages` |
| 激活事件 | 每个窗口 `onView:<window-id>`，每个命令 `onCommand:<command-id>` |
| Windows | 宿主声明 `ui` 1、2；不声明 `ui` 3（原生内容）和 `ui` 4，这些包在安装前被拒绝 |

### 版本对照

| 版本 | 最低级别 | 新增 | 其他要求 | macOS | Windows |
| --- | --- | --- | --- | --- | --- |
| `ui` 1 | 2 | 1–64 个文本工具窗口（`left`、`bottom`、`right`）；最多 128 个命令，放在 `commandPalette`、`toolbar`、`menu`，每个命令打开同一插件的一个窗口 | `ui.contribute` | 支持 | 支持 |
| `ui` 2 | 3 | 窗口内容可以是包内网页 `web.entry`（`.html`） | `ui.web`；声明 `ui` 2 却没有网页窗口会被拒绝 | 支持 | 支持 |
| `ui` 3 | 3 | 由宿主绘制的 `native` 内容：`tree`、`list`、`form`；全部节点最多 2000 个、深度 16；表单最多 32 个控件、16 个按钮 | 只有 `native` 一种窗口内容 | 支持 | 不支持 |
| `ui` 4 | 3 | 命令可以不带 `toolWindowID`，`toolWindows` 可以为空；命令运行时向后端发送 `commands/didRun` | `backend` 2、`backend.launch`；`didRun` 只是事件，不代表命令的业务动作成功 | 支持 | 不支持 |

所有 ID 以插件 ID 加点开头；标题最多 256 字节，窗口文本最多 65536 字节，按原样显示，不作为 HTML 或脚本执行。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [API level 2: first-stage host-owned workbench UI](../../contract/v1/README.md#api-level-2-first-stage-host-owned-workbench-ui) | `ui` 1：工具窗口、命令、ID、限制、宿主负责的部分 |
| [Web content (`ui` 2)](../../contract/v1/README.md#web-content-ui-2) | 包内网页、桥接、状态和生命周期 |
| [Host-rendered native content (`ui` 3, native schema 1)](../../contract/v1/README.md#host-rendered-native-content-ui-3-native-schema-1) | 原生树、列表、表单，控件种类与限制 |
| [Backend-only commands (`ui` 4)](../../contract/v1/README.md#backend-only-commands-ui-4) | 不打开窗口的命令与 `commands/didRun` |
| [UI icons (macOS host)](../../contract/v1/README.md#ui-icons-macos-host) | 宿主图标 ID（如 `{"host": "database"}`）与自带 SVG/PNG |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | 后端弹出的消息 `window/showInformation`、`showWarning`、`showError` |
| [Future public extension direction](../../contract/v1/README.md#future-public-extension-direction-not-implemented-in-api-level-2) | 尚未实现的方向，不代表当前能力 |
| [Current Windows availability](../../contract/v1/README.md#current-windows-availability) | Windows 声明的 `ui` 版本：1、2，不声明 3、4 |
| [Existing community store and Lua template](../../contract/v1/README.md#existing-community-store-and-lua-template) | 给社区 Lua 模板加窗口的步骤 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`manifest/hello-lithe.plugin.json`](../../contract/v1/fixtures/manifest/hello-lithe.plugin.json) | 级别 2、`ui` 1：一个文本窗口和打开它的命令 |
| [`ui/web-panel.plugin.json`](../../contract/v1/fixtures/ui/web-panel.plugin.json) | `ui` 2：两个网页窗口和一个命令 |
| [`ui/native-panel.plugin.json`](../../contract/v1/fixtures/ui/native-panel.plugin.json) | `ui` 3：原生树面板 |
| [`ui/native-form.plugin.json`](../../contract/v1/fixtures/ui/native-form.plugin.json) | `ui` 3：原生表单 |
| [`ui/native-controls.plugin.json`](../../contract/v1/fixtures/ui/native-controls.plugin.json) | `ui` 3：表单的各种控件 |
| [`backend/backend.plugin.json`](../../contract/v1/fixtures/backend/backend.plugin.json) | 网页窗口调用 `backend` 1 |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `ui` 4：不打开窗口的后端命令 |
| [`native-ui.schema.json`](../../contract/v1/native-ui.schema.json) | 原生内容的 JSON Schema |

界面相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。可运行的原生面板示例是 [Node SDK 的 Sample Explorer](../../sdk/node/examples/sample-build/plugin.json)。

### 相关指南

- [插件后端：工作区上下文与宿主服务](../../guides/plugin-backend-workspace.md)

---

## English

A plugin can place tool windows and commands in the workbench. The window shell (docking, title, activity bar, order, layout, theme, lifecycle) always belongs to the host; the plugin supplies only the content: plain text, a packaged web page, or a native tree, list, or form drawn by the host. A command opens one of the plugin's windows, and from `ui` 4 on it may instead just notify the backend without opening a window.

| Item | Value |
| --- | --- |
| Manifest fields | `ui.toolWindows`, `ui.commands` (optional `icon`) |
| Capability versions | `ui` 1–4; native content also has [native schema](../../contract/v1/native-ui.schema.json) 1. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | Pages call the [plugin backend](backend.md) through `functions.*`; `ui` 4 commands notify the backend with `commands/didRun` (`litheFunctions` 3) |
| API IDs | API-02, 03, 13, 14, 17; see the [API catalog](../api-catalog.md#apis-usable-now) |
| Permissions | `ui.contribute`; `ui.web` for web pages; `backend.launch` for `ui` 4 commands; `window.messages` for messages shown by the backend |
| Activation events | `onView:<window-id>` per window, `onCommand:<command-id>` per command |
| Windows | Advertises `ui` 1 and 2; not `ui` 3 (native content) or `ui` 4, so those packages are refused before installation |

### Versions compared

| Version | Lowest level | Adds | Other requirements | macOS | Windows |
| --- | --- | --- | --- | --- | --- |
| `ui` 1 | 2 | 1–64 text tool windows (`left`, `bottom`, `right`); up to 128 commands in `commandPalette`, `toolbar`, `menu`, each opening one of the same plugin's windows | `ui.contribute` | Yes | Yes |
| `ui` 2 | 3 | Window content may be a packaged web page, `web.entry` (`.html`) | `ui.web`; `ui` 2 without a web window is refused | Yes | Yes |
| `ui` 3 | 3 | Host-drawn `native` content: `tree`, `list`, `form`; at most 2000 nodes and depth 16 in total; a form has at most 32 controls and 16 buttons | `native` is the window's only content | Yes | No |
| `ui` 4 | 3 | Commands without `toolWindowID`, and `toolWindows` may be empty; running a command sends `commands/didRun` to the backend | `backend` 2, `backend.launch`; `didRun` is an event, not proof that the command's action succeeded | Yes | No |

Every ID starts with the plugin ID and a dot. Titles are at most 256 bytes and window text at most 65536 bytes; text is shown literally and never evaluated as HTML or script.

### Contract sections

| Section | Content |
| --- | --- |
| [API level 2: first-stage host-owned workbench UI](../../contract/v1/README.md#api-level-2-first-stage-host-owned-workbench-ui) | `ui` 1: tool windows, commands, IDs, limits, what the host owns |
| [Web content (`ui` 2)](../../contract/v1/README.md#web-content-ui-2) | Packaged pages, bridge, state, and lifecycle |
| [Host-rendered native content (`ui` 3, native schema 1)](../../contract/v1/README.md#host-rendered-native-content-ui-3-native-schema-1) | Native trees, lists, forms; control kinds and limits |
| [Backend-only commands (`ui` 4)](../../contract/v1/README.md#backend-only-commands-ui-4) | Commands without a window, and `commands/didRun` |
| [UI icons (macOS host)](../../contract/v1/README.md#ui-icons-macos-host) | Host icon IDs (such as `{"host": "database"}`) and packaged SVG/PNG |
| [Public document, file and window methods (protocol version 3)](../../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3) | Backend messages `window/showInformation`, `showWarning`, `showError` |
| [Future public extension direction](../../contract/v1/README.md#future-public-extension-direction-not-implemented-in-api-level-2) | Directions not implemented yet; not current capabilities |
| [Current Windows availability](../../contract/v1/README.md#current-windows-availability) | The `ui` versions Windows advertises: 1 and 2, not 3 or 4 |
| [Existing community store and Lua template](../../contract/v1/README.md#existing-community-store-and-lua-template) | Adding a window to the community Lua template |

### Examples and cases

| File | Content |
| --- | --- |
| [`manifest/hello-lithe.plugin.json`](../../contract/v1/fixtures/manifest/hello-lithe.plugin.json) | Level 2, `ui` 1: one text window and the command that opens it |
| [`ui/web-panel.plugin.json`](../../contract/v1/fixtures/ui/web-panel.plugin.json) | `ui` 2: two web windows and a command |
| [`ui/native-panel.plugin.json`](../../contract/v1/fixtures/ui/native-panel.plugin.json) | `ui` 3: a native tree panel |
| [`ui/native-form.plugin.json`](../../contract/v1/fixtures/ui/native-form.plugin.json) | `ui` 3: a native form |
| [`ui/native-controls.plugin.json`](../../contract/v1/fixtures/ui/native-controls.plugin.json) | `ui` 3: every form control kind |
| [`backend/backend.plugin.json`](../../contract/v1/fixtures/backend/backend.plugin.json) | A web window calling `backend` 1 |
| [`backend/command-backend.plugin.json`](../../contract/v1/fixtures/backend/command-backend.plugin.json) | `ui` 4: a backend command without a window |
| [`native-ui.schema.json`](../../contract/v1/native-ui.schema.json) | JSON Schema for native content |

UI rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json). A runnable native panel is the [Node SDK Sample Explorer](../../sdk/node/examples/sample-build/plugin.json).

### Related guides

- [Plugin backend: workspace contexts and host services](../../guides/plugin-backend-workspace.md)
