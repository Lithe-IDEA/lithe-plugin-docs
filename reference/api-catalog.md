# 插件 API 总表（API-01–44） / Plugin API catalog (API-01–44)

[中文](#中文) · [English](#english)

## 中文

Lithe 的插件公共 API 按 [API 实施计划](https://github.com/1lck/Lithe-IDEA/blob/preview/docs/development/macos-plugin-api-plan.md)统一编号为 API-01 到 API-44。本页列出每个编号**现在能用什么**、**还没有什么**，并把能用的部分链接到契约章节和功能页。参数、返回值、错误码以 [`contract/v1/README.md`](../contract/v1/README.md) 为准；版本号见[能力与协议版本对照](capabilities-and-versions.md)。按主仓库最新 `preview` 核对，日期 2026-11-05。

读这张表前先知道三件事：

- **宿主方法只有 macOS 实现。** 所有 `lithe/host/*` 方法由 Rust Core 校验、macOS 宿主执行。Windows 没有插件后端宿主，也不声明 `backend` 能力。清单里的声明式能力（语言、设置、界面等）在 Windows 上的支持见[版本对照页](capabilities-and-versions.md#能力版本requires)。
- **运行时以 `getCapabilities` 为准。** 某个方法出现在本表，不代表当前宿主一定提供；后端调用前先查 `lithe/host/getCapabilities` 返回的 `services`。
- **计划中的条目没有可用方法名。** 第二张表只描述要做的能力，不列候选方法名；在契约正式写入之前，不要按猜测的名字调用。

状态含义：

| 状态 | 含义 |
| --- | --- |
| 已公开 | 方法已进入 Core 白名单并由 macOS 宿主分发，插件按权限可以调用 |
| 既有能力 | 在这次编号之前就已存在的契约能力，继续可用 |
| 部分完成 | 一部分已经可用，表中写明缺口 |
| 进行中 | 正在实现，尚不可用 |
| 未做 | 尚未开放；不代表 IDE 内部没有相关功能 |

### 现在能用的 API

| ID | 名称 | 状态 | 现在能用 | 尚未提供 | 功能页 |
| --- | --- | --- | --- | --- | --- |
| API-01 | 能力发现、激活与订阅 | 已公开 | [`getCapabilities`](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3)、`backend` 2 的 `onWorkspaceOpen` 激活、`subscriptions/dispose` | 新安装的后端仍需重启后才能启动 | [插件后端](domains/backend.md) |
| API-02 | 命令 | 部分完成 | [`ui` 4 不打开工具窗口的命令](../contract/v1/README.md#backend-only-commands-ui-4)和 `commands/didRun` 通知 | 运行期动态注册命令、带返回结果的执行 | [界面](domains/ui.md) |
| API-03 | 菜单、快捷键、上下文键、状态栏 | 部分完成 | 已有的命令放置位置（`ui.commands`） | 通用菜单、快捷键贡献、上下文键、状态栏 | [界面](domains/ui.md) |
| API-04 | 工作区与文档事件 | 部分完成 | `workspace/didChangeContexts`、`workspace/watch` 的文件事件 | 文档、选区、活动编辑器的变化事件 | [插件后端](domains/backend.md) |
| API-05 | 工作区上下文 | 已公开 | [`workspace/listContexts`、`getContext`、`getFolders`](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3)，带 `contextID`/`generation` 与 `rootID` | — | [插件后端](domains/backend.md) |
| API-06 | 文件读取 | 已公开 | [`workspace/readDirectory`、`stat`、`readFile`](../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3)；每次最多读 128 KiB | — | [插件后端](domains/backend.md) |
| API-07 | 文件监听 | 已公开 | `workspace/watch` 加 `subscriptions/notify`、`dispose`；上下文或会话结束时回收，溢出时要求重扫 | — | [插件后端](domains/backend.md) |
| API-08 | 搜索 | 部分完成 | `workspace/findFiles`、`findText`，分页、可取消 | 只搜磁盘内容，不含未保存的修改；匹配的精确范围目前是 `null` | [插件后端](domains/backend.md) |
| API-09 | 文档与选区读取 | 已公开 | `documents/list`、`documents/getSnapshot`、`editor/getActive`、`editor/getSelections`，包含未保存状态、版本、编码、换行符 | — | [编辑器](domains/editor.md) |
| API-10 | 版本化编辑 | 已公开 | `editor/applyEdits`、`replaceSelections`、`insertText`；校验版本，拒绝输入法组字中和只读文档，一批编辑是一次撤销 | — | [编辑器](domains/editor.md) |
| API-13 | 原生视图数据源 | 部分完成 | [`ui` 3 原生 Tree、List、Form](../contract/v1/README.md#host-rendered-native-content-ui-3-native-schema-1) | 独立的后端数据源、懒加载 | [界面](domains/ui.md) |
| API-14 | 网页消息 | 部分完成 | [包内网页](../contract/v1/README.md#web-content-ui-2)的桥接、状态和生命周期 | 通用的页面双向消息 | [界面](domains/ui.md) |
| API-15 | 打开与定位 | 部分完成 | `editor/open`、`editor/reveal`，返回 `documentID` 和版本 | 传入范围时只把光标放到起点；不能替换预览标签 | [编辑器](domains/editor.md) |
| API-16 | HTML 预览与外部浏览器 | 已公开 | `preview/openHTML`（`editorAndPreview` 或 `previewOnly`）、`browser/openExternal`（HTTP(S) 地址或已保存的 HTML） | — | [编辑器](domains/editor.md) |
| API-17 | 消息、选择、输入 | 部分完成 | `window/showInformation`、`showWarning`、`showError`，带按钮和取消 | 通用的快速选择、输入框、经授权的文件句柄 | [界面](domains/ui.md) |
| API-19 | 设置 | 部分完成 | [声明式设置 `settings` 1、2](../contract/v1/README.md#plugin-settings-api-2) | 运行期读取、查看来源、修改设置和设置变化事件 | [设置](domains/settings.md) |
| API-20 | 存储 | 部分完成 | 插件级 `${stateDir}`、后端和工作区的可写目录（`stateDirectory`）、网页状态 | 统一的键值存储、配额 | [插件后端](domains/backend.md) |
| API-21 | 凭据 | 既有能力 | [协议 2 的 `credentials/get`、`set`、`delete`](../contract/v1/README.md#host-services-protocol-version-2)，按插件隔离 | — | [插件后端](domains/backend.md) |
| API-22 | 输出、日志、进度 | 部分完成 | [插件函数的进度和取消](../contract/v1/README.md#progress)；面向开发者的[结构化诊断日志 `litheLogging` 1](../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) | 独立的输出面板、面向用户的日志和进度 API | [插件后端](domains/backend.md) |
| API-23 | 工具链与受管进程 | 部分完成 | 进程声明（`serverProcess`）、运行时解析、[共享可执行文件候选](../contract/v1/README.md#shared-executable-candidates-executableresolution-12)、工具定义、工具链提供者（macOS） | 后端可调用的通用工具链和进程服务；保存固定（pin）、`toolchainConsumers`、Windows 支持 | [工具链](domains/toolchains.md) |
| API-24 | 终端 | 部分完成 | [插件自有原生终端](../contract/v1/README.md#plugin-owned-native-terminals-function-protocol-3) `terminal/create`、`show`、`close`、`reopen`、`dispose`、`onExit`（macOS、本机工作区） | 向终端发送文本、读取终端输出；Windows | [插件后端](domains/backend.md) |
| API-25 | 语言、语法、LSP | 部分完成 | [语言](../contract/v1/README.md#languages)与[语言服务器](../contract/v1/README.md#language-servers)；[`languages` 2 的包内 Monarch 语法和编辑器配置](../contract/v1/README.md#grammars-and-editor-configuration-languages-2)；[编辑器语言功能 Provider `editor/registerProviders`](../contract/v1/README.md#editor-language-feature-providers-protocol-version-3)，把 `definition`、`hover`、`completion`、`codeLens`、`foldingRange`、`semanticTokens`、`documentLink` 这 7 种类型绑定到普通插件函数，语言服务器优先、插件结果只补充（macOS） | 符号、格式化、重命名、Code Action、签名帮助、内联提示、运行期传入 JSON 动态注册高亮 | [语言](domains/languages.md)、[编辑器](domains/editor.md) |
| API-27 | 编辑器动作、主题、装饰 | 部分完成 | [独立编辑器动作 `editor/registerActions`](../contract/v1/README.md#independent-editor-actions-protocol-version-3)，绑定普通函数和文件范围，不依赖 LSP、构建或运行；编辑器动作第 2 版增加行首多图标、至多 4 个 `alternatives`、点击结果的 `reveal` | 主题、代码片段、其他装饰 | [编辑器](domains/editor.md) |
| API-29 | 构建、任务、测试、运行准备 | 部分完成 | [`buildSystems` 1–4](../contract/v1/README.md#api-level-3-build-systems)、[`buildProviders` 1–11](../contract/v1/README.md#build-providers)、`buildModelInputs` 1 | 开发指南还不完整 | [声明式构建](domains/build-systems.md)、[构建提供者](domains/build-providers.md) |
| API-30 | 调试适配器 | 既有能力 | [`debugAdapters` 1、2](../contract/v1/README.md#api-level-4-debug-adapters) 与宿主的 DAP 会话 | — | [调试适配器](domains/debug-adapters.md) |
| API-31 | VCS 提供者 | 既有能力 | [`vcsProviders` 1](../contract/v1/README.md#api-level-3-vcs-providers) | 不含内置 Git 的公共读取（见 API-42） | [VCS](domains/vcs.md) |
| API-33 | 项目生成器 | 既有能力 | [`projectGenerators` 1、2](../contract/v1/README.md#project-generators) | — | [项目生成器](domains/project-generators.md) |
| API-37 | 环境信息与语言包 | 部分完成 | `getCapabilities` 返回协议版本、可用方法、限制和后端运行位置（`backendAuthority`） | 语言包、环境和主题变化事件 | [插件后端](domains/backend.md) |
| API-40 | 工作区 authority | 部分完成 | 工作区上下文带 `local` 或 `remote`；文件和编辑器方法对远程上下文明确返回 `plugin.host.authorityUnsupported` | 远程执行 | [插件后端](domains/backend.md) |
| API-41 | 激活事件与启动预算 | 部分完成 | [`onWorkspaceOpen`](../contract/v1/README.md#permissions-and-activation)、`onCommand`、后台两路并发启动、`initialize` 超时、有界通知队列 | 按工作区内容匹配激活、全局冷启动预算 | [插件后端](domains/backend.md) |

### 计划中的 API

| ID | 名称 | 状态 | 计划提供 |
| --- | --- | --- | --- |
| API-11 | 保存与文件变更 | 未做 | 保存、另存为、关闭，以及工作区文件的创建、改名、删除 |
| API-12 | 多文件编辑、虚拟文档、diff | 未做 | 跨文件编辑、按 URI scheme 提供的只读虚拟文档、对比视图 |
| API-18 | 剪贴板、Finder | 未做 | 剪贴板读写、在 Finder 中显示 |
| API-26 | 诊断集合 | 未做 | 后端推送的诊断集合，带文档版本；现有的 LSP 诊断不属于这个 API |
| API-28 | 自定义编辑器 | 未做 | 插件自有文档的编辑器，含保存、撤销、恢复 |
| API-32 | 服务 | 未做 | 通用服务提供者（仍是提案） |
| API-34 | IDE/AI/MCP | 未做 | 插件可用的 IDE、AI、MCP 能力与授权 |
| API-35 | 插件间调用与依赖 | 进行中 | 第一阶段：经 Core 中转调用另一个插件的函数（不是安装依赖）；之后是依赖与导出、版本求解 |
| API-36 | WASM | 未做 | WASM/WASI 运行时 |
| API-38 | 受管网络 | 未做 | 宿主代为执行的网络请求和下载 |
| API-39 | 身份验证 | 未做 | 身份验证会话和提供者 |
| API-42 | 内置 Git 只读快照 | 未做 | 内置 Git 状态的只读快照和变化事件 |
| API-43 | 保存前事件 | 未做 | 保存前事件，以及有时限的保存前编辑 |
| API-44 | Agent 侧贡献 | 未做 | 插件贡献 Skills、Hooks、MCP，并单独审查授权 |

清单、市场、包信任、快捷键导入是安装和打包层面的契约，不在 API-01–44 编号内，见各自的功能页：[清单](domains/manifest.md)、[市场](domains/marketplace.md)、[包信任](domains/package-trust.md)、[快捷键导入](domains/keymap-import.md)。

### 已公开的宿主方法

后端向宿主发送的请求，前缀都是 `lithe/host/`，下表省略前缀。一个 API 编号下可以有多个方法，每个方法单独一行，按 API 编号排列。

- **“API”列是规划编号**，来自 [API 实施计划](https://github.com/1lck/Lithe-IDEA/blob/preview/docs/development/macos-plugin-api-plan.md)，和“协议”列的 `litheFunctions` 版本、清单里的能力版本分开记录。同一个 API 的方法可以分几个 PR 陆续加入；协议 3 里向后兼容地新增方法，也不升协议版本。每个方法哪天、由哪个 PR 首次加入，见[插件协议更新记录](protocol-changelog.md#宿主方法按-api-编号)。
- **“协议”列**是最低的 `litheFunctions` 版本。除 `getCapabilities`、`subscriptions/dispose`、`workspace/listContexts` 外，协议 3 的方法都要带 `context: {contextID, generation}`。
- **“权限”列**：除 `credentials.store` 外，都要求清单声明 `backend` 2，否则安装时拒绝。

| API | 方法 | 用途 | 权限 | 协议 |
| --- | --- | --- | --- | --- |
| API-01、API-37 | `getCapabilities` | 查询协商的协议版本、后端运行位置（`backendAuthority`）、当前可用的服务和限制；由 Core 直接回答 | 每个插件会话都有 | 3 |
| API-01 | `subscriptions/dispose` | 取消一个订阅，即 `workspace/watch`、`editor/registerActions`、`terminal/onExit` 返回的 `subscriptionID`；未知或已取消的 ID 也返回 `null` | 每个插件会话都有 | 3 |
| API-05 | `workspace/listContexts` | 列出当前的工作区上下文，带变化序号 `revision` | `workspace.metadata` | 3 |
| API-05 | `workspace/getContext` | 读取一个工作区上下文 | `workspace.metadata` | 3 |
| API-05 | `workspace/getFolders` | 列出上下文的根目录，每个根目录带 `rootID` | `workspace.metadata` | 3 |
| API-06 | `workspace/readDirectory` | 分页列出目录项 | `workspace.files.read` | 3 |
| API-06 | `workspace/stat` | 读取文件或目录的类型、大小和修改时间 | `workspace.files.read` | 3 |
| API-06 | `workspace/readFile` | 读取磁盘上的文件，返回文本或 base64，超过上限时截断 | `workspace.files.read` | 3 |
| API-07 | `workspace/watch` | 监听文件或目录的变化，事件经 `subscriptions/notify` 送达 | `workspace.files.watch` | 3 |
| API-08 | `workspace/findFiles` | 在一个根目录下按名称查找文件，分页 | `workspace.search` | 3 |
| API-08 | `workspace/findText` | 在磁盘内容中搜索文本，按行返回匹配，分页 | `workspace.search` | 3 |
| API-09 | `documents/list` | 列出已打开文档的摘要 | `documents.read` | 3 |
| API-09 | `documents/getSnapshot` | 读取文档当前缓冲区的全文，包含未保存的修改 | `documents.read` | 3 |
| API-09 | `editor/getActive` | 读取前台文本标签的文档摘要，没有时为 `null` | `documents.read` | 3 |
| API-09 | `editor/getSelections` | 读取文档的全部选区和主选区 | `documents.read` | 3 |
| API-10 | `editor/applyEdits` | 按范围批量编辑，先校验 `expectedVersion` | `editor.edit` | 3 |
| API-10 | `editor/replaceSelections` | 替换选区的文本：一个替换文本用于全部选区，或每个选区一个 | `editor.edit` | 3 |
| API-10 | `editor/insertText` | 在指定位置插入文本 | `editor.edit` | 3 |
| API-15 | `editor/open` | 在编辑器中打开文件，没有缓冲区时从磁盘读取；可以前台或后台打开，并定位光标 | `editor.open` | 3 |
| API-15 | `editor/reveal` | 参数和 `editor/open` 相同，但只处理已打开的文件；文件没打开时返回 `plugin.host.notFound`，不会从磁盘打开 | `editor.open` | 3 |
| API-16 | `preview/openHTML` | 打开 HTML 文件的预览（`editorAndPreview` 或 `previewOnly`），预览使用实时缓冲区 | `preview.open` | 3 |
| API-16 | `browser/openExternal` | 在外部浏览器打开 HTTP(S) 地址或已保存的 HTML 文件 | `browser.open` | 3 |
| API-17 | `window/showInformation` | 显示信息消息，可以带按钮，返回用户点的按钮序号 | `window.messages` | 3 |
| API-17 | `window/showWarning` | 显示警告消息，用法同 `showInformation` | `window.messages` | 3 |
| API-17 | `window/showError` | 显示错误消息，用法同 `showInformation` | `window.messages` | 3 |
| API-21 | `credentials/get` | 读取本插件保存的密钥，没有时为 `null` | `credentials.store` | 2 |
| API-21 | `credentials/set` | 保存密钥，最多 16 KiB | `credentials.store` | 2 |
| API-21 | `credentials/delete` | 删除密钥，没有保存过也返回 `null` | `credentials.store` | 2 |
| API-24 | `terminal/create` | 经执行确认创建插件自有原生终端，返回 `terminalID` 和 `generation` | `terminal.create` | 3 |
| API-24 | `terminal/show` | 显示并选中本插件的终端 | `terminal.create` | 3 |
| API-24 | `terminal/close` | 结束进程树并关闭标签，按字节预算保留显示历史和启动配置 | `terminal.create` | 3 |
| API-24 | `terminal/reopen` | 经重新确认，用原启动配置创建新进程，`generation` 加 1，并恢复有界的显示历史 | `terminal.create` | 3 |
| API-24 | `terminal/dispose` | 关闭终端，彻底释放句柄、启动配置和历史 | `terminal.create` | 3 |
| API-24 | `terminal/onExit` | 订阅终端每个 `generation` 的退出结果 | `terminal.create` | 3 |
| API-25 | `editor/registerProviders` | 注册编辑器语言功能 Provider：把 `definition`、`hover`、`completion`、`codeLens`、`foldingRange`、`semanticTokens`、`documentLink` 绑定到后端的普通函数和文件范围（`selector`），返回 `subscriptionID`；语言服务器优先，插件结果只补充 | `editor.providers`（同时需要 `documents.read`） | 3 |
| API-27 | `editor/registerActions` | 注册编辑器动作：把后端的普通函数绑定到文件范围（`selector`），返回 `subscriptionID` | `editor.actions`（同时需要 `documents.read`） | 3 |

快捷键导入提供者另有 `keymapImport/listInputs`、`keymapImport/readChunk` 两个方法，属于 `litheKeymapImport` 协议，不在 API-01–44 内，见[快捷键导入](domains/keymap-import.md)。

宿主发给后端的通知（后端不回复），前缀同样是 `lithe/host/`：

| API | 通知 | 用途 | 授予条件 |
| --- | --- | --- | --- |
| API-02 | `commands/didRun` | 用户运行了本插件的命令；只是事件，不表示命令的业务操作已经成功 | 每个插件会话都有 |
| API-04、API-05 | `workspace/didChangeContexts` | 工作区上下文有变化，带新的 `revision`；后端再调用 `listContexts` 读取 | `workspace.metadata` |
| API-07、API-24 | `subscriptions/notify` | 送达一个订阅的事件，例如文件变化（`workspace/watch`）、终端退出（`terminal/onExit`） | 每个插件会话都有 |

### 限制

| 项目 | 上限 |
| --- | --- |
| 单次文件读取、文档快照 | 128 KiB 源文本或字节 |
| 发给编辑器动作收集函数的文档 | 宿主的 `limits.editorActionSourceBytes`（macOS 1 MiB；宿主未报告时 128 KiB） |
| 一批编辑或选区 | 1,000 项 |
| 目录分页 | 默认 200，最多 1,000 |
| 搜索分页 | 默认 100，最多 200（搜索总结果 200） |
| 机器请求的宿主时限 | 30 秒（等待用户选择的消息不计） |
| 窗口消息 | 每个插件每 10 秒 5 条，每个插件最多 3 条待回答，每个窗口最多 10 条；正文 2,000 字符，最多 3 个按钮，每个 40 字符 |
| 未回答的宿主请求 | 每个会话 16 个 |
| 通知队列 | 每个会话 128 条 |
| 插件自有终端 | 每个插件最多保留 8 个句柄 |
| 编辑器 Provider 注册 | 每个选择器 1–32 项，每个插件跨上下文最多 32 个注册 |
| 编辑器 Provider 单次调用时限 | hover、completion 400 ms，definition 1500 ms，其余类型 2000 ms；迟到的答复直接丢弃，注册不能自定义时限 |

失败时的错误码和处理方式见契约的 [Outcomes and error codes](../contract/v1/README.md#outcomes-and-error-codes) 与 [Protocol 3 宿主失败码表](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3)。取消或超时**不保证回滚**：编辑可能已经生效，不要自动重放。

---

## English

Lithe numbers its public plugin APIs API-01 to API-44 in the [API implementation plan](https://github.com/1lck/Lithe-IDEA/blob/preview/docs/development/macos-plugin-api-plan.md). This page lists, for each ID, **what is usable now** and **what is still missing**, and links the usable parts to contract sections and domain pages. [`contract/v1/README.md`](../contract/v1/README.md) is normative for params, results, and error codes; versions are in [Capability and protocol versions](capabilities-and-versions.md). Checked against the latest `preview` of the Lithe repository on 2026-11-05.

Three things to know before reading the tables:

- **Host methods are implemented on macOS only.** Rust Core checks every `lithe/host/*` method and the macOS host executes it. Windows has no plugin backend host and does not advertise the `backend` capability. For declarative manifest capabilities (languages, settings, UI, and so on) on Windows, see the [version page](capabilities-and-versions.md#capability-versions-requires).
- **At runtime, `getCapabilities` decides.** A method listed here is not necessarily offered by the current host; check `services` from `lithe/host/getCapabilities` before calling it.
- **Planned entries have no usable method names.** The second table only describes the capability to come and lists no candidate method names; until the contract defines them, do not call guessed names.

Status meanings:

| Status | Meaning |
| --- | --- |
| Public | The method is on Core's allowlist and dispatched by the macOS host; a plugin with the permission can call it |
| Existing | A contract capability that predates the numbering and stays available |
| Partial | Part of it is usable; the table names the gaps |
| In progress | Being implemented, not usable yet |
| Not started | Not open yet; this does not mean the IDE lacks the feature internally |

### APIs usable now

| ID | Name | Status | Usable now | Not yet | Domain page |
| --- | --- | --- | --- | --- | --- |
| API-01 | Capability discovery, activation, subscriptions | Public | [`getCapabilities`](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3), `onWorkspaceOpen` activation of `backend` 2, `subscriptions/dispose` | A newly installed backend still needs a restart before it can start | [Plugin backend](domains/backend.md) |
| API-02 | Commands | Partial | [`ui` 4 commands without a tool window](../contract/v1/README.md#backend-only-commands-ui-4) and the `commands/didRun` notification | Registering commands at runtime, result-bearing execution | [UI](domains/ui.md) |
| API-03 | Menus, keybindings, context keys, status bar | Partial | Existing command placements (`ui.commands`) | General menus, keybinding contributions, context keys, status bar | [UI](domains/ui.md) |
| API-04 | Workspace and document events | Partial | `workspace/didChangeContexts`, file events from `workspace/watch` | Document, selection, and active editor change events | [Plugin backend](domains/backend.md) |
| API-05 | Workspace contexts | Public | [`workspace/listContexts`, `getContext`, `getFolders`](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3) with `contextID`/`generation` and `rootID` | — | [Plugin backend](domains/backend.md) |
| API-06 | File reads | Public | [`workspace/readDirectory`, `stat`, `readFile`](../contract/v1/README.md#public-document-file-and-window-methods-protocol-version-3); at most 128 KiB per read | — | [Plugin backend](domains/backend.md) |
| API-07 | File watching | Public | `workspace/watch` with `subscriptions/notify` and `dispose`; reclaimed when the context or session ends, with a rescan request on overflow | — | [Plugin backend](domains/backend.md) |
| API-08 | Search | Partial | `workspace/findFiles` and `findText`, paged and cancellable | Searches disk content only, not unsaved changes; the exact match range is `null` for now | [Plugin backend](domains/backend.md) |
| API-09 | Documents and selections | Public | `documents/list`, `documents/getSnapshot`, `editor/getActive`, `editor/getSelections`, with dirty state, version, encoding, and line ending | — | [Editor](domains/editor.md) |
| API-10 | Versioned edits | Public | `editor/applyEdits`, `replaceSelections`, `insertText`; version checks, refusal during IME composition and in read-only documents, one undo step per batch | — | [Editor](domains/editor.md) |
| API-13 | Native view data sources | Partial | [`ui` 3 native Tree, List, Form](../contract/v1/README.md#host-rendered-native-content-ui-3-native-schema-1) | An independent backend data source, lazy loading | [UI](domains/ui.md) |
| API-14 | Webview messaging | Partial | The [package web page](../contract/v1/README.md#web-content-ui-2) bridge, state, and lifecycle | General two-way page messaging | [UI](domains/ui.md) |
| API-15 | Open and reveal | Partial | `editor/open` and `editor/reveal`, returning `documentID` and version | A range only places the caret at its start; no preview-tab replacement | [Editor](domains/editor.md) |
| API-16 | HTML preview and external browser | Public | `preview/openHTML` (`editorAndPreview` or `previewOnly`), `browser/openExternal` (HTTP(S) URLs or saved HTML) | — | [Editor](domains/editor.md) |
| API-17 | Messages, quick pick, input | Partial | `window/showInformation`, `showWarning`, `showError` with buttons and dismissal | General quick pick, input box, authorized file handles | [UI](domains/ui.md) |
| API-19 | Settings | Partial | [Declarative settings `settings` 1 and 2](../contract/v1/README.md#plugin-settings-api-2) | Reading, inspecting the source of, and updating settings at runtime, and settings change events | [Settings](domains/settings.md) |
| API-20 | Storage | Partial | The plugin-wide `${stateDir}`, writable backend and workspace directories (`stateDirectory`), web page state | A unified key-value store, quotas | [Plugin backend](domains/backend.md) |
| API-21 | Credentials | Existing | [Protocol 2 `credentials/get`, `set`, `delete`](../contract/v1/README.md#host-services-protocol-version-2), isolated per plugin | — | [Plugin backend](domains/backend.md) |
| API-22 | Output, log, progress | Partial | [Plugin function progress and cancellation](../contract/v1/README.md#progress); developer-facing [structured diagnostics `litheLogging` 1](../contract/v1/README.md#structured-plugin-diagnostics-logging-protocol-1) | A dedicated output panel, user-facing log and progress APIs | [Plugin backend](domains/backend.md) |
| API-23 | Toolchains and managed processes | Partial | Process declarations (`serverProcess`), runtime resolution, [shared executable candidates](../contract/v1/README.md#shared-executable-candidates-executableresolution-12), tool definitions, toolchain providers (macOS) | A general toolchain and process service backends can call; saving pins, `toolchainConsumers`, Windows support | [Toolchains](domains/toolchains.md) |
| API-24 | Terminals | Partial | [Plugin-owned native terminals](../contract/v1/README.md#plugin-owned-native-terminals-function-protocol-3) `terminal/create`, `show`, `close`, `reopen`, `dispose`, `onExit` (macOS, local workspaces) | Sending text to a terminal, reading its output; Windows | [Plugin backend](domains/backend.md) |
| API-25 | Languages, grammars, LSP | Partial | [Languages](../contract/v1/README.md#languages) and [language servers](../contract/v1/README.md#language-servers); [packaged Monarch grammars and editor configuration in `languages` 2](../contract/v1/README.md#grammars-and-editor-configuration-languages-2); [editor language-feature providers `editor/registerProviders`](../contract/v1/README.md#editor-language-feature-providers-protocol-version-3), binding 7 types (`definition`, `hover`, `completion`, `codeLens`, `foldingRange`, `semanticTokens`, `documentLink`) to ordinary plugin functions, where the language server answers first and plugin results only add (macOS) | Symbols, formatting, rename, code actions, signature help, inlay hints, dynamic registration of highlighting from JSON passed at runtime | [Languages](domains/languages.md), [Editor](domains/editor.md) |
| API-27 | Editor actions, themes, decorations | Partial | [Independent editor actions `editor/registerActions`](../contract/v1/README.md#independent-editor-actions-protocol-version-3), binding ordinary functions to file ranges without LSP, build, or run; editor actions version 2 adds several icons at a line start, up to 4 `alternatives`, and a `reveal` in the click result | Themes, snippets, other decorations | [Editor](domains/editor.md) |
| API-29 | Build, tasks, tests, run preparation | Partial | [`buildSystems` 1–4](../contract/v1/README.md#api-level-3-build-systems), [`buildProviders` 1–11](../contract/v1/README.md#build-providers), `buildModelInputs` 1 | The developer guides are incomplete | [Build systems](domains/build-systems.md), [Build providers](domains/build-providers.md) |
| API-30 | Debug adapters | Existing | [`debugAdapters` 1 and 2](../contract/v1/README.md#api-level-4-debug-adapters) and the host's DAP sessions | — | [Debug adapters](domains/debug-adapters.md) |
| API-31 | VCS providers | Existing | [`vcsProviders` 1](../contract/v1/README.md#api-level-3-vcs-providers) | Public reads of built-in Git are not included (see API-42) | [VCS](domains/vcs.md) |
| API-33 | Project generators | Existing | [`projectGenerators` 1, 2](../contract/v1/README.md#project-generators) | — | [Project generators](domains/project-generators.md) |
| API-37 | Environment info and l10n | Partial | `getCapabilities` returns the protocol version, usable methods, limits, and where the backend runs (`backendAuthority`) | Language packs, environment and theme change events | [Plugin backend](domains/backend.md) |
| API-40 | Workspace authority | Partial | Workspace contexts carry `local` or `remote`; file and editor methods refuse a remote context with `plugin.host.authorityUnsupported` | Remote execution | [Plugin backend](domains/backend.md) |
| API-41 | Activation events and startup budget | Partial | [`onWorkspaceOpen`](../contract/v1/README.md#permissions-and-activation), `onCommand`, two concurrent background starts, `initialize` timeout, bounded notification queues | Activation by workspace content, a global cold-start budget | [Plugin backend](domains/backend.md) |

### Planned APIs

| ID | Name | Status | Planned |
| --- | --- | --- | --- |
| API-11 | Save and file operations | Not started | Save, save as, close, and creating, renaming, and deleting workspace files |
| API-12 | Multi-file edits, content providers, diff | Not started | Cross-file edits, read-only virtual documents provided per URI scheme, a diff view |
| API-18 | Clipboard and Finder | Not started | Reading and writing the clipboard, revealing in Finder |
| API-26 | Diagnostics collections | Not started | Diagnostics collections pushed by a backend, with document versions; existing LSP diagnostics are not this API |
| API-28 | Custom editors | Not started | Editors for plugin-owned documents, with save, undo, and restore |
| API-32 | Services | Not started | General service providers (still a proposal) |
| API-34 | IDE, AI, MCP | Not started | IDE, AI, and MCP capabilities for plugins, with authorization |
| API-35 | Cross-plugin calls and dependencies | In progress | Phase one: calling another plugin's function through Core (not an install dependency); then dependencies and exports, version solving |
| API-36 | WASM | Not started | A WASM/WASI runtime |
| API-38 | Managed network | Not started | Network requests and downloads run by the host |
| API-39 | Authentication | Not started | Authentication sessions and providers |
| API-42 | Built-in Git read-only snapshot | Not started | A read-only snapshot of built-in Git state and change events |
| API-43 | Will-save events | Not started | Will-save events and time-bounded edits before save |
| API-44 | Agent contributions | Not started | Skills, hooks, and MCP contributed by plugins, with separate authorization review |

The manifest, marketplace, package trust, and shortcut import are installation and packaging contracts outside API-01–44; see their domain pages: [Manifest](domains/manifest.md), [Marketplace](domains/marketplace.md), [Package trust](domains/package-trust.md), [Shortcut import](domains/keymap-import.md).

### Public host methods

Requests a backend sends to the host, all prefixed `lithe/host/`; the tables omit the prefix. One API ID can cover several methods; each method has its own row, ordered by API ID.

- **The “API” column holds planning IDs** from the [API implementation plan](https://github.com/1lck/Lithe-IDEA/blob/preview/docs/development/macos-plugin-api-plan.md). They are recorded separately from the `litheFunctions` version in the “Protocol” column and from manifest capability versions. The methods of one API can arrive over several PRs, and a backward-compatible method added within protocol 3 does not bump the protocol version. The date and PR that first added each method are in the [protocol changelog](protocol-changelog.md#宿主方法按-api-编号) (Chinese only).
- **The “Protocol” column** is the lowest `litheFunctions` version. Apart from `getCapabilities`, `subscriptions/dispose`, and `workspace/listContexts`, every protocol 3 method takes `context: {contextID, generation}`.
- **The “Permission” column**: apart from `credentials.store`, every permission requires the manifest to declare `backend` 2, or installation refuses it.

| API | Method | Purpose | Permission | Protocol |
| --- | --- | --- | --- | --- |
| API-01, API-37 | `getCapabilities` | Reports the negotiated protocol version, where the backend runs (`backendAuthority`), and the services and limits available now; Core answers it itself | Every plugin-owned session | 3 |
| API-01 | `subscriptions/dispose` | Cancels a subscription, that is, a `subscriptionID` returned by `workspace/watch`, `editor/registerActions`, or `terminal/onExit`; an unknown or already disposed ID also returns `null` | Every plugin-owned session | 3 |
| API-05 | `workspace/listContexts` | Lists the current workspace contexts with the change counter `revision` | `workspace.metadata` | 3 |
| API-05 | `workspace/getContext` | Reads one workspace context | `workspace.metadata` | 3 |
| API-05 | `workspace/getFolders` | Lists the context's root folders, each with a `rootID` | `workspace.metadata` | 3 |
| API-06 | `workspace/readDirectory` | Lists directory entries page by page | `workspace.files.read` | 3 |
| API-06 | `workspace/stat` | Reads the kind, size, and modification time of a file or directory | `workspace.files.read` | 3 |
| API-06 | `workspace/readFile` | Reads a file from disk as text or base64, truncated past the limit | `workspace.files.read` | 3 |
| API-07 | `workspace/watch` | Watches a file or directory for changes; events arrive through `subscriptions/notify` | `workspace.files.watch` | 3 |
| API-08 | `workspace/findFiles` | Finds files by name under one root, page by page | `workspace.search` | 3 |
| API-08 | `workspace/findText` | Searches disk content for text and returns matching lines, page by page | `workspace.search` | 3 |
| API-09 | `documents/list` | Lists summaries of the open documents | `documents.read` | 3 |
| API-09 | `documents/getSnapshot` | Reads the full text of a document's current buffer, unsaved changes included | `documents.read` | 3 |
| API-09 | `editor/getActive` | Reads the document summary of the foreground text tab, or `null` | `documents.read` | 3 |
| API-09 | `editor/getSelections` | Reads all of a document's selections and the primary one | `documents.read` | 3 |
| API-10 | `editor/applyEdits` | Applies a batch of range edits after checking `expectedVersion` | `editor.edit` | 3 |
| API-10 | `editor/replaceSelections` | Replaces the text of selections: one replacement for all of them, or one per selection | `editor.edit` | 3 |
| API-10 | `editor/insertText` | Inserts text at a position | `editor.edit` | 3 |
| API-15 | `editor/open` | Opens a file in the editor, reading it from disk when no buffer exists; opens it in the foreground or background and can place the caret | `editor.open` | 3 |
| API-15 | `editor/reveal` | Takes the same params as `editor/open` but only handles files already open; a file that is not open returns `plugin.host.notFound` and is not opened from disk | `editor.open` | 3 |
| API-16 | `preview/openHTML` | Opens a preview of an HTML file (`editorAndPreview` or `previewOnly`) that uses the live buffer | `preview.open` | 3 |
| API-16 | `browser/openExternal` | Opens an HTTP(S) URL or a saved HTML file in the external browser | `browser.open` | 3 |
| API-17 | `window/showInformation` | Shows an information message, optionally with buttons, and returns the index of the button the user chose | `window.messages` | 3 |
| API-17 | `window/showWarning` | Shows a warning message; used like `showInformation` | `window.messages` | 3 |
| API-17 | `window/showError` | Shows an error message; used like `showInformation` | `window.messages` | 3 |
| API-21 | `credentials/get` | Reads a secret this plugin stored, or `null` | `credentials.store` | 2 |
| API-21 | `credentials/set` | Stores a secret of at most 16 KiB | `credentials.store` | 2 |
| API-21 | `credentials/delete` | Deletes a secret; also returns `null` when nothing was stored | `credentials.store` | 2 |
| API-24 | `terminal/create` | Creates a plugin-owned native terminal after execution consent and returns `terminalID` and `generation` | `terminal.create` | 3 |
| API-24 | `terminal/show` | Shows and selects one of this plugin's terminals | `terminal.create` | 3 |
| API-24 | `terminal/close` | Ends the process tree and closes the tab, keeping display history within a byte budget and the launch settings | `terminal.create` | 3 |
| API-24 | `terminal/reopen` | After renewed consent, starts a new process with the original launch settings, increments `generation`, and restores bounded display history | `terminal.create` | 3 |
| API-24 | `terminal/dispose` | Closes the terminal and releases its handle, launch settings, and history for good | `terminal.create` | 3 |
| API-24 | `terminal/onExit` | Subscribes to the exit outcome of each `generation` of a terminal | `terminal.create` | 3 |
| API-25 | `editor/registerProviders` | Registers editor language-feature providers: binds `definition`, `hover`, `completion`, `codeLens`, `foldingRange`, `semanticTokens`, or `documentLink` to an ordinary backend function and file ranges (`selector`), and returns a `subscriptionID`; the language server answers first and plugin results only add | `editor.providers` (also needs `documents.read`) | 3 |
| API-27 | `editor/registerActions` | Registers editor actions that bind an ordinary backend function to file ranges (`selector`) and returns a `subscriptionID` | `editor.actions` (also needs `documents.read`) | 3 |

Shortcut import providers also have two methods, `keymapImport/listInputs` and `keymapImport/readChunk`. They belong to the `litheKeymapImport` protocol, outside API-01–44; see [Shortcut import](domains/keymap-import.md).

Notifications the host sends to a backend (the backend answers none), also prefixed `lithe/host/`:

| API | Notification | Purpose | Granted by |
| --- | --- | --- | --- |
| API-02 | `commands/didRun` | The user ran one of this plugin's commands; it is an event only and does not mean the command's business action succeeded | Every plugin-owned session |
| API-04, API-05 | `workspace/didChangeContexts` | The workspace contexts changed, with the new `revision`; the backend then calls `listContexts` to read them | `workspace.metadata` |
| API-07, API-24 | `subscriptions/notify` | Delivers an event of one subscription, such as a file change (`workspace/watch`) or a terminal exit (`terminal/onExit`) | Every plugin-owned session |

### Limits

| Item | Limit |
| --- | --- |
| One file read or document snapshot | 128 KiB of source text or bytes |
| The document sent to an editor action collector | The host's `limits.editorActionSourceBytes` (1 MiB on macOS; 128 KiB when the host does not report it) |
| One batch of edits or selections | 1,000 entries |
| Directory pages | 200 by default, at most 1,000 |
| Search pages | 100 by default, at most 200 (200 search results in total) |
| Host deadline for machine requests | 30 seconds (messages waiting on the user are exempt) |
| Window messages | 5 per plugin per 10 seconds, at most 3 pending per plugin and 10 per window; 2,000 text characters, at most 3 buttons of 40 characters |
| Unanswered host requests | 16 per session |
| Notification queue | 128 entries per session |
| Plugin-owned terminals | At most 8 retained handles per plugin |
| Editor provider registrations | 1–32 selector entries per registration, at most 32 registrations per plugin across contexts |
| Editor provider call budget | 400 ms for hover and completion, 1500 ms for definition, 2000 ms for the other types; a late answer is dropped, and a registration cannot set its own budget |

For failure codes and how to handle them, see the contract's [Outcomes and error codes](../contract/v1/README.md#outcomes-and-error-codes) and the [protocol 3 host failure table](../contract/v1/README.md#workspace-contexts-and-capability-discovery-protocol-version-3). Cancellation or timeout **does not promise rollback**: an edit may already have taken effect, so do not replay it automatically.
