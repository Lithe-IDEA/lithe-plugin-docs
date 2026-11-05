# 版本控制提供者 / VCS providers

[中文](#中文) · [English](#english)

## 中文

插件可以为 Git 以外的版本控制系统（第一个是 Subversion）实现一个提供者。提供者是一个 LSP 基础协议进程，和[构建提供者](build-providers.md)同类，回答统一 VCS 模型的请求：根目录、文件状态、基准内容、历史、注解、提交、回滚、更新。版本控制界面、差异视图、提交窗口都归宿主；界面按提供者协商的能力决定显示什么，从不按 VCS 名字分支。Git 由内置支持负责，ID `git` 保留。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `vcsProviders[]`：`id`、`displayName`、`detection.markers`、`metadataPaths`、`tool`（VCS 客户端命令和校验参数）、`serverProcess` 启动字段、`mayUseNetwork` |
| 能力版本 | `vcsProviders` 1 加 `serverProcess` 1，需要级别 3。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | `litheVcs` 1：宿主在 `initialize` 里提供 `capabilities.experimental.litheVcs: {"versions": [1]}`，提供者回答 `{"version": 1, "vcs", "capabilities"}`，见[进程协议版本](../capabilities-and-versions.md#进程协议版本) |
| API 编号 | [API-31](../api-catalog.md#现在能用的-api)：VCS 提供者。读取内置 Git 的公共接口是另一项（API-42，计划中） |
| 权限 | `process.vcs`、`languageServer.launch` |
| 激活事件 | `onVcsRoot:<id>`；根目录检测只看标记文件，不启动进程，找到该 VCS 的根目录后才启动提供者 |
| Windows | 不支持；Windows 宿主不声明 `vcsProviders` |
| 官方 SDK | [Node SDK](../../sdk/node/README.md) 实现 `litheVcs` 1；Java、Rust SDK 不实现 |

### 版本对照

| 版本 | 新增 | Core | macOS | Windows | Node SDK |
| --- | --- | --- | --- | --- | --- |
| `vcsProviders` 1 / `litheVcs` 1 | 清单声明、标记检测、元数据监视、`vcs.detectRoots`、`vcs.ownerOf`、`lsp.vcsRequest`、`vcs.decodeReply`；必选能力 `status`、`baseContent`、`checkin`、`rollback`，可选能力 `history`、`fileHistory`、`annotate`、`update`、`push`、`staging`、`fileOperations`、`conflicts`、`cleanup` | 支持 | 支持 | 不支持 | 支持 |

`push` 在版本 1 里只是声明：协商了 `push` 的提供者告诉宿主"提交是本地的，发布是另一步"，版本 1 没有发布操作，以后的版本再建模。

### 能力与操作

| 能力 | 必选 | 回复 |
| --- | --- | --- |
| `status` | 是 | 根目录的文件状态 |
| `baseContent` | 是 | `{content, binary}`，基准版本的内容，最多 32 MiB；新增或未纳入版本控制的文件为 `null` |
| `checkin` | 是 | `{revision}`；VCS 不报告新版本时为 `null` |
| `rollback` | 是 | `{}` |
| `history`、`fileHistory` | 否 | `{entries, nextCursor}`，分页，保持提供者的顺序（新的在前） |
| `annotate` | 否 | `{revisions, lines}`，最多 500,000 行 |
| `update` | 否 | `{revision, conflicts}` |
| `fileOperations`、`conflicts`、`cleanup` | 否 | `fileOperation`、`resolve`、`cleanup` 回复 `{}` |
| `push`、`staging` | 否 | 只影响界面：没有 `push` 时宿主不提供"提交并推送" |

路径离开根目录、缺必填字段、超出大小限制的回复都无效。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [API level 3: VCS providers](../../contract/v1/README.md#api-level-3-vcs-providers) | 清单字段、保留 ID、检测、安装审查 |
| [VCS provider contract v1](https://github.com/1lck/Lithe-IDEA/blob/preview/shared/contracts/vcs-provider-v1.md) | 字段、Core 命令、协商、回复格式、提供者消息、错误码、Subversion 说明（在 Lithe 主仓库） |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`vcs/vcs-provider.plugin.json`](../../contract/v1/fixtures/vcs/vcs-provider.plugin.json) | Subversion 提供者：`.svn` 目录标记、监视 `.svn/wc.db`、`svn` 客户端、Node 运行时启动、`mayUseNetwork` |

共享示例里没有 VCS 拒绝用例；错误码（包括 `plugin.vcs.reserved`）见 VCS 契约的 [Error codes](https://github.com/1lck/Lithe-IDEA/blob/preview/shared/contracts/vcs-provider-v1.md#error-codes)。

### 相关指南

- [Node SDK：A VCS provider](../../sdk/node/README.md#a-vcs-provider)

---

## English

A plugin can implement a provider for a version control system other than Git (Subversion is the first). A provider is an LSP base-protocol process, the same kind as a [build provider](build-providers.md), and answers requests of one VCS-neutral model: roots, file status, base content, history, annotation, check-in, rollback, and update. The version control UI, diff views, and commit window belong to the host; the UI decides what to show from the capabilities the provider negotiated, never from the VCS name. Git is served by built-in support, and the ID `git` is reserved.

| Item | Value |
| --- | --- |
| Manifest field | `vcsProviders[]`: `id`, `displayName`, `detection.markers`, `metadataPaths`, `tool` (the VCS client commands and validation arguments), the `serverProcess` launch fields, `mayUseNetwork` |
| Capability versions | `vcsProviders` 1 with `serverProcess` 1, at level 3. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | `litheVcs` 1: the host offers `capabilities.experimental.litheVcs: {"versions": [1]}` in `initialize`, and the provider answers `{"version": 1, "vcs", "capabilities"}`; see [process protocol versions](../capabilities-and-versions.md#process-protocol-versions) |
| API IDs | [API-31](../api-catalog.md#apis-usable-now): VCS providers. Public reads of the built-in Git are a separate item (API-42, planned) |
| Permissions | `process.vcs`, `languageServer.launch` |
| Activation events | `onVcsRoot:<id>`; root detection reads marker files only and starts no process, and a provider starts only after a root of its VCS is found |
| Windows | Not supported; the Windows host does not advertise `vcsProviders` |
| Official SDKs | The [Node SDK](../../sdk/node/README.md) implements `litheVcs` 1; the Java and Rust SDKs do not |

### Versions compared

| Version | Adds | Core | macOS | Windows | Node SDK |
| --- | --- | --- | --- | --- | --- |
| `vcsProviders` 1 / `litheVcs` 1 | Manifest declaration, marker detection, metadata watching, `vcs.detectRoots`, `vcs.ownerOf`, `lsp.vcsRequest`, `vcs.decodeReply`; required capabilities `status`, `baseContent`, `checkin`, `rollback`; optional `history`, `fileHistory`, `annotate`, `update`, `push`, `staging`, `fileOperations`, `conflicts`, `cleanup` | Yes | Yes | No | Yes |

In version 1, `push` is only a declaration: a provider that negotiates it tells the host that check-in is local and publishing is a separate step. Version 1 has no publishing operation; a later version will model it.

### Capabilities and operations

| Capability | Required | Reply |
| --- | --- | --- |
| `status` | Yes | File status of a root |
| `baseContent` | Yes | `{content, binary}`, the content at the base revision, at most 32 MiB; `null` for added or unversioned files |
| `checkin` | Yes | `{revision}`; `null` when the VCS reports none |
| `rollback` | Yes | `{}` |
| `history`, `fileHistory` | No | `{entries, nextCursor}`, paged, in provider order (newest first) |
| `annotate` | No | `{revisions, lines}`, at most 500,000 lines |
| `update` | No | `{revision, conflicts}` |
| `fileOperations`, `conflicts`, `cleanup` | No | `fileOperation`, `resolve`, and `cleanup` reply `{}` |
| `push`, `staging` | No | UI only: without `push` the host offers no "Commit and Push" |

A reply with a path outside the root, a missing required field, or an exceeded size limit is invalid.

### Contract sections

| Section | Content |
| --- | --- |
| [API level 3: VCS providers](../../contract/v1/README.md#api-level-3-vcs-providers) | Manifest fields, the reserved ID, detection, install review |
| [VCS provider contract v1](https://github.com/1lck/Lithe-IDEA/blob/preview/shared/contracts/vcs-provider-v1.md) | Fields, Core commands, negotiation, reply shapes, provider messages, error codes, Subversion notes (in the Lithe main repository) |

### Examples and cases

| File | Content |
| --- | --- |
| [`vcs/vcs-provider.plugin.json`](../../contract/v1/fixtures/vcs/vcs-provider.plugin.json) | A Subversion provider: a `.svn` directory marker, watching `.svn/wc.db`, the `svn` client, a Node runtime launch, `mayUseNetwork` |

The shared fixtures have no VCS refusal cases; the error codes (including `plugin.vcs.reserved`) are in [Error codes](https://github.com/1lck/Lithe-IDEA/blob/preview/shared/contracts/vcs-provider-v1.md#error-codes) of the VCS contract.

### Related guides

- [Node SDK: A VCS provider](../../sdk/node/README.md#a-vcs-provider)
