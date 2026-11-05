# 插件协议更新记录

本页记录插件协议和 API 的每次修改：哪天、哪个 PR、谁改的、改了什么。只有中文。

- **收录范围**：Lithe 主仓库 `preview` 分支上的 PR，只要它修改了下面三处中的任意一处：插件契约 `shared/contracts/plugins/v1/`、Core 能力表 [`capabilities.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/plugins/capabilities.rs)、插件函数协议 [`plugin_function.rs`](https://github.com/1lck/Lithe-IDEA/blob/preview/rust/lithe-core/src/lsp/interface/plugin_function.rs)。三处都没改、只改宿主内部实现的 PR 不收录。
- **日期**：PR 合入 `preview` 的日期。
- **修改人**：原 PR 的作者（GitHub 账号）。登记这些 PR 的同步 PR 会在描述里 @ 他们，请他们核对自己那一行。
- **记录到**：`preview` 的 [`22c14a96b`](https://github.com/1lck/Lithe-IDEA/commit/22c14a96b3e6a8152686e2abdd6cb075e2849365)（[#1763](https://github.com/1lck/Lithe-IDEA/pull/1763)），即本仓库 `contract/v1/` 当前副本的来源。
- **每一版的内容和宿主现状**：见[能力与协议版本对照](capabilities-and-versions.md)。字段和行为以 [`contract/v1/README.md`](../contract/v1/README.md) 为准，官方 SDK 的发版见 [SDK 发版历史](capabilities-and-versions.md#sdk-发版历史)。

时间线的两列这样读：

- **API 编号**：这个 PR 涉及的 API-01–44 规划编号，来自 [API 实施计划](https://github.com/1lck/Lithe-IDEA/blob/preview/docs/development/macos-plugin-api-plan.md)，和协议版本、能力版本分开记录。编号由 [#1548](https://github.com/1lck/Lithe-IDEA/pull/1548)（2026-10-28）引入，更早的 PR 没有编号，按计划“API-01–44 完成度总表”里各编号对应的既有能力补填，例如 `debugAdapters` 是 API-30，`vcsProviders` 是 API-31。— 表示没有改到具体 API，例如清单、市场、包信任、SDK 或契约文档整理。
- **协议 / 能力**：**加粗**的是这个 PR 新增的版本号；没有加粗的，是同一版本内的补充、宿主接入或说明修正。

## 按协议查首次出现

### 插件 API 级别与文档格式

| 名称 | 版本 | 日期 | PR |
| --- | --- | --- | --- |
| 插件 API 级别 `pluginAPIVersion` | 1 | 2026-10-14 | [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) |
| 插件 API 级别 `pluginAPIVersion` | 2 | 2026-10-14 | [#1214](https://github.com/1lck/Lithe-IDEA/pull/1214)；工作台 UI 由 [#1206](https://github.com/1lck/Lithe-IDEA/pull/1206) 加入 |
| 插件 API 级别 `pluginAPIVersion` | 3（`requires`） | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| 插件 API 级别 `pluginAPIVersion` | 4（调试适配器） | 2026-10-26 | [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) |
| 插件清单 `schemaVersion` | 2 | 2026-10-14 | [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) |
| 市场索引 `schemaVersion` | 1 | 2026-10-14 | [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) |
| 原生 UI 文档 `schemaVersion` | 1 | 2026-10-24 | [#1423](https://github.com/1lck/Lithe-IDEA/pull/1423) |
| `litheBuildModel`、`litheDependencies`、`litheTestResults` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `litheTestDiscovery` | 1 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `litheBuildConfiguration` | 1 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| 本机工具链文档 `.lithe/toolchains/local.json` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |

### 能力版本（`requires`）

`requires` 由 #1318 引入，首批 8 个能力都从版本 1 开始。其中 `languages`、`languageServers`、`serverProcess`、`executionProviders`、`settings`、`ui` 的内容在此之前已经随 API 级别 1、2 提供，#1318 只是给它们定了能力版本号。

| 能力 | 版本 | 日期 | PR |
| --- | --- | --- | --- |
| `languages` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `languages` | 2 | 2026-10-31 | [#1617](https://github.com/1lck/Lithe-IDEA/pull/1617) |
| `languageServers` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `languageServers` | 2 | 2026-10-21 | [#1395](https://github.com/1lck/Lithe-IDEA/pull/1395) |
| `languageServers` | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| `serverProcess` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `serverProcess` | 2 | 2026-10-26 | [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) |
| `serverProcess` | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| `serverProcess` | 4 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| `executionProviders` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `settings` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `settings` | 2 | 2026-10-21 | [#1395](https://github.com/1lck/Lithe-IDEA/pull/1395) |
| `ui` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `ui` | 2 | 2026-10-18 | [#1334](https://github.com/1lck/Lithe-IDEA/pull/1334) |
| `ui` | 3 | 2026-10-24 | [#1423](https://github.com/1lck/Lithe-IDEA/pull/1423) |
| `ui` | 4 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| `buildSystems` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `buildSystems` | 2 | 2026-10-23 | [#1420](https://github.com/1lck/Lithe-IDEA/pull/1420) |
| `buildSystems` | 3、4 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| `buildProviders` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) |
| `buildProviders` | 2–6 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `buildProviders` | 7–11 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| `buildModelInputs` | 1 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `executableResolution` | 1 | 2026-10-23 | [#1420](https://github.com/1lck/Lithe-IDEA/pull/1420) |
| `executableResolution` | 2 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |
| `vcsProviders` | 1 | 2026-10-23 | [#1430](https://github.com/1lck/Lithe-IDEA/pull/1430) |
| `projectGenerators` | 1 | 2026-10-24 | [#1452](https://github.com/1lck/Lithe-IDEA/pull/1452) |
| `projectGenerators` | 2 | 2026-11-05 | [#1740](https://github.com/1lck/Lithe-IDEA/pull/1740) |
| `backend` | 1 | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| `backend` | 2 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| `debugAdapters` | 1 | 2026-10-26 | [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) |
| `debugAdapters` | 2 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `debugAdapters` | 3 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| `packageTrust` | 1 | 2026-10-28 | [#1483](https://github.com/1lck/Lithe-IDEA/pull/1483) |
| `toolDefinitions` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |
| `toolchainProviders` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) |
| `keymapImportProviders` | 1 | 2026-11-03 | [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) |
| `outputLinkMatchers` | 1 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| `pluginFunctions` | 1 | 2026-11-05 | [#1742](https://github.com/1lck/Lithe-IDEA/pull/1742) |

### 进程协议版本

| 协议 | 版本 | 日期 | PR |
| --- | --- | --- | --- |
| `litheFunctions` | 1 | 2026-10-20 | [#1358](https://github.com/1lck/Lithe-IDEA/pull/1358) |
| `litheFunctions` | 2（宿主服务） | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| `litheFunctions` | 3（工作区上下文） | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| `litheFunctions` | 4（跨插件调用） | 2026-11-05 | [#1742](https://github.com/1lck/Lithe-IDEA/pull/1742) |
| `litheBuild` | 1 | 2026-10-18 | [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) 定义；macOS 宿主由 [#1356](https://github.com/1lck/Lithe-IDEA/pull/1356) 接入 |
| `litheBuild` | 2–6 | 2026-10-28 | [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) |
| `litheBuild` | 7–11 | 2026-11-04 | [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) |
| `litheVcs` | 1 | 2026-10-23 | [#1430](https://github.com/1lck/Lithe-IDEA/pull/1430) |
| `litheToolchains` | 1 | 2026-11-01 | [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) 实现；正式契约由 [#1752](https://github.com/1lck/Lithe-IDEA/pull/1752) 写入 |
| `litheKeymapImport` | 1 | 2026-11-03 | [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) |
| `litheLogging` | 1 | 2026-11-04 | [#1699](https://github.com/1lck/Lithe-IDEA/pull/1699) |

### 宿主方法（按 API 编号）

函数协议 3 里向后兼容地新增方法时不升协议版本，上面的版本表看不出某个方法是什么时候加的，所以宿主方法和通知在这里逐个登记。一个 API 编号下可以有多个方法，每个方法一行，记首次加入的日期和 PR。方法名省略前缀 `lithe/host/`；用途和权限见 [API 总表](api-catalog.md#已公开的宿主方法)。`keymapImport/*` 属于 `litheKeymapImport` 协议，不在 API-01–44 内。`functions/providers`、`functions/invoke` 属于函数协议 4，只对协商到 4 的后端开放。

| API | 方法 | 协议 | 日期 | PR |
| --- | --- | --- | --- | --- |
| API-01、API-37 | `getCapabilities` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-01 | `subscriptions/dispose` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-02 | `commands/didRun`（通知） | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-04、API-05 | `workspace/didChangeContexts`（通知） | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-05 | `workspace/listContexts` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-05 | `workspace/getContext` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-05 | `workspace/getFolders` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-06 | `workspace/readDirectory` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-06 | `workspace/stat` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-06 | `workspace/readFile` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-07 | `workspace/watch` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-07、API-24 | `subscriptions/notify`（通知） | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-08 | `workspace/findFiles` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-08 | `workspace/findText` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-09 | `documents/list` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-09 | `documents/getSnapshot` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-09 | `editor/getActive` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-09 | `editor/getSelections` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-10 | `editor/applyEdits` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-10 | `editor/replaceSelections` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-10 | `editor/insertText` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-12 | `documents/registerContentProvider` | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| API-15 | `editor/open` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-15 | `editor/reveal` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-16 | `preview/openHTML` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-16 | `browser/openExternal` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-17 | `window/showInformation` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-17 | `window/showWarning` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-17 | `window/showError` | 3 | 2026-10-30 | [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) |
| API-21 | `credentials/get` | 2 | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| API-21 | `credentials/set` | 2 | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| API-21 | `credentials/delete` | 2 | 2026-10-24 | [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) |
| API-22、API-25 | `languageServer/setStatus` | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| API-23 | `run/start` | 3 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| API-23 | `run/stop` | 3 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| API-23 | `run/didExit`（通知） | 3 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| API-24 | `terminal/create` | 3 | 2026-11-02 | [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) |
| API-24 | `terminal/show` | 3 | 2026-11-02 | [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) |
| API-24 | `terminal/close` | 3 | 2026-11-02 | [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) |
| API-24 | `terminal/reopen` | 3 | 2026-11-02 | [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) |
| API-24 | `terminal/dispose` | 3 | 2026-11-02 | [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) |
| API-24 | `terminal/onExit` | 3 | 2026-11-02 | [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) |
| API-25 | `editor/registerProviders` | 3 | 2026-11-05 | [#1730](https://github.com/1lck/Lithe-IDEA/pull/1730) |
| API-25 | `languageServer/request` | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| API-25 | `languageServer/notify` | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| API-25 | `languageServer/didChangeState`（通知） | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| API-25 | `languageServer/didReceiveNotification`（通知） | 3 | 2026-11-05 | [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) |
| API-27 | `editor/registerActions` | 3 | 2026-10-31 | [#1617](https://github.com/1lck/Lithe-IDEA/pull/1617) |
| API-30 | `debug/start` | 3 | 2026-11-05 | [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) |
| API-35 | `functions/providers` | 4 | 2026-11-05 | [#1742](https://github.com/1lck/Lithe-IDEA/pull/1742) |
| API-35 | `functions/invoke` | 4 | 2026-11-05 | [#1742](https://github.com/1lck/Lithe-IDEA/pull/1742) |
| — | `keymapImport/listInputs` | 3 | 2026-11-03 | [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) |
| — | `keymapImport/readChunk` | 3 | 2026-11-03 | [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) |

## 时间线（新的在上）

### 2026-11-05

| PR | 修改人 | 协议 / API | 改了什么 |
| --- | --- | --- | --- |
| [#1763](https://github.com/1lck/Lithe-IDEA/pull/1763) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 清单与工具定义：`hostIntegration`、短工具 ID（API-23、API-25） | 语言服务器的 `hostIntegration`（插件 API 级别 2）对所有插件开放，不再限 `dev.lithe` 命名空间：集成只读取声明插件自己包里的资源，交给同一插件启动的服务器或辅助进程，不额外授予权限；宿主未实现的 ID 仍返回 `plugin.integration.unsupported`。构建系统的 `hostIntegration` 仍只限官方插件。工具定义：任何插件都可以定义短 ID（如 `compiler`）或限定 ID `<pluginID>:<id>`，宿主不再提供来源证明；同一个短 ID 被多个已启用插件定义时不归任何一方，注册表不登记这些定义，并报告为冲突；限定 ID 不会冲突；所有权不按到达顺序或插件 ID 顺序决定。`toolchains.registry` 的输入从 `{plugins:[{manifest,verifiedOfficial}]}` 改为 `{plugins:[{manifest}]}`，输出新增 `conflicts:[{tool,pluginIDs}]`（按 `tool` 排序，`pluginIDs` 也排序）。macOS 设置页对每个冲突显示一条诊断，建议只保留其中一个插件。`plugin.integration.unsupported` 的含义改为：宿主未实现的集成，或官方命名空间以外的插件请求构建系统的宿主集成。`fixtures/manifest/invalid-manifests.json` 删除 1 条清单拒绝用例（“host integration requested outside the official namespace”）。不新增版本号，不新增宿主方法。 |
| [#1746](https://github.com/1lck/Lithe-IDEA/pull/1746) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`debugAdapters` 3**、**`serverProcess` 4**、**`outputLinkMatchers` 1**、函数协议 3：后端发起的运行与调试（API-22、API-23、API-30） | `debugAdapters` 3：适配器可声明 `features` 和 `defaults`（热代码替换、步进过滤器），新增 fixture `debug-adapter-features.plugin.json`。`serverProcess` 4：运行时按工具定义 ID 引用（清单 `requiresTools`），不再使用宿主的封闭运行时列表。`outputLinkMatchers` 1（API-22，新能力）：插件声明匹配器，让 Run 和测试控制台的输出出现链接；Core 负责匹配，宿主只上样式并转发点击，`resolve` 函数在点击时才调用。新增权限 `run.start`（需要 `backend` 2），授予 `lithe/host/run/start`、`lithe/host/run/stop` 和通知 `lithe/host/run/didExit`；新增权限 `debug.start`（需要 `backend` 2 和已声明的 `debugAdapters` 条目），只授予 `lithe/host/debug/start`。`run/start`（API-23）：参数 `context, label, process, temporaryFiles?, debug?, rerun?`，结果 `{runID}`，启动宿主拥有的 Run 会话（Run 窗口条目、控制台、停止按钮、退出状态）；`process.executable` 只能是 `{kind:"tool", tool}`（清单 `requiresTools` 列出的工具定义 ID）或 `{kind:"packaged", path}`（插件包内文件）；`temporaryFiles` 最多 4 个，每个不超过 4 MiB；`debug` 让它成为调试会话，`rerun` 让用户可以再次运行。`run/stop`（API-23）：参数 `{runID}` 或 `{sessionID}`，结果 `null`，停止本插件启动的运行或调试会话，不带 `context`。`debug/start`（API-30）：参数 `context, label, adapter, request, configuration`，结果 `{sessionID}`，在本插件自己的适配器上启动调试会话；`configuration` 不超过 256 KiB，`noDebug` 和 `__` 开头的键属于宿主，会被拒绝。通知 `lithe/host/run/didExit`：`{runID, exitCode?, signal?}`，用 `run/start` 启动的运行结束时发一次，宿主丢失进程时两个字段都没有。新增错误码 `plugin.host.toolUnresolved`（-32025）：工具解析不到可执行文件。新增 fixtures `fixtures/run/`（含 `run-start.json`）和 `fixtures/debug-adapters/debug-start.json`。方法和通知是函数协议 3 内的增量，不升协议版本。`outputLinkMatchers` 和后端发起的运行与调试目前只在 macOS 实现。 |
| [#1745](https://github.com/1lck/Lithe-IDEA/pull/1745) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`languageServers` 3**、**`serverProcess` 3**、函数协议 3：后端决定启动方式、语言服务器会话、内容 Provider（API-12、API-20、API-22、API-25） | `serverProcess` 3 新增后端解析的启动方式 `launch.kind: "backend"`：语言服务器的命令行由插件后端函数每次给出，只有语言服务器能用，需要 `backend` 2 和 `pluginAPIVersion` 不低于 4；新增 fixtures `fixtures/languages/language-server-backend-launch.plugin.json`、`fixtures/languages/language-server-launch-resolution.json`。`languageServers` 3 新增插件后端与自己的服务器交谈所用的会话字段：`requestMethods`、`commands`、`serializedCommands`、`forwardNotifications`、`state`、`remote`。新增权限 `languageServer.request`（需要 `backend` 2；只在 `languageServers` 3 的服务器列出 `requestMethods` 或 `forwardNotifications` 时才可声明），只授予本插件自己服务器的 `lithe/host/languageServer/*` 方法和两个通知；新增权限 `documents.contentProvider`（需要 `backend` 2），只授予 `lithe/host/documents/registerContentProvider`，不授予读写文档。新宿主方法均带 `context` 和 `serverID`，`serverID` 是本插件某个服务器的 `languageID`，别的插件的服务器返回 `plugin.host.foreignContribution`：`lithe/host/languageServer/request`（API-25）参数加 `method`、`params?`、`timeoutMilliseconds?`（1–300000，默认 60000），结果 `{value}`，向本插件自己的服务器发请求，受会话策略检查，第 129 个等待中的请求返回 `plugin.host.busy`；`lithe/host/languageServer/notify`（API-25）参数加 `method`、`params?`，结果 `{delivered}`；`lithe/host/languageServer/setStatus`（API-22、API-25）参数加 `state`（`preparing`、`ready`、`degraded`、`failed`）、`message?`、`percentage?`（0–100 整数），结果 `{}`，在宿主状态界面显示插件自己的准备状态（导入工程、建索引）；`lithe/host/documents/registerContentProvider`（API-12）参数 `{context, scheme, function}`，结果 `{subscriptionID}`，打开该 scheme 的文档时宿主调用函数取只读内容（不超过 1 MiB，宿主等 30 秒），每个工作区窗口每个 scheme 只能有一个 Provider，用 `subscriptions/dispose` 撤销。新宿主通知（权限 `languageServer.request`，API-25）：`lithe/host/languageServer/didChangeState`（`{context, serverID, state, generation}`，`state` 为 `starting`、`running`、`stopped`、`crashed`）和 `lithe/host/languageServer/didReceiveNotification`（`{context, serverID, generation, method, params?, truncated?}`，转发服务器发出且 `forwardNotifications` 列出的通知，超过 `limits.notificationBytes` 时只送一次 `truncated: true`，不带 `params`）。新增错误码：`plugin.host.languageServerUnavailable`（-32022，服务器未运行）、`plugin.host.languageServerError`（-32023，服务器返回错误）、`plugin.host.methodNotDeclared`（-32024，方法未在会话里声明）、`plugin.host.foreignContribution`（-32026）。语言服务器按工作区的状态目录按 `state.retention` 后台回收（API-20，由 Core 决定）。方法和通知是函数协议 3 内的增量，不升协议版本。仅 macOS；远程后端启动未支持；Windows 不声明 `backend`。 |
| [#1742](https://github.com/1lck/Lithe-IDEA/pull/1742) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`pluginFunctions` 1**、**函数协议 4**（API-35） | 插件函数跨插件中转的第一阶段：由 Core 中转调用，不是安装依赖。清单顶层新增 `pluginFunctions` 段，含导出 `exports` 和调用声明 `calls`；需要 `pluginAPIVersion` 不低于 3，以及 `requires.pluginFunctions: 1`、`backend: 2`、`serverProcess: 1`；不授予任何清单权限。函数协议 4 保留版本 3 的全部内容，新增两个宿主方法：`lithe/host/functions/providers`，参数 `{functionID}`，结果 `{providers:[{pluginID,version}]}`，列出导出该函数、已启用且不是调用方自己的插件，按插件 ID 排序，调用方必须声明过该函数，否则 `plugin.function.notDeclared`；`lithe/host/functions/invoke`，参数 `{pluginID, functionID, context:{contextID,generation}, params?, timeoutMilliseconds?, parentOperationID?}`，结果 `{value}`，Core 以 `lithe/function/invoke` 转给被调方，带顶层 `caller` 和 `operationID`。`timeoutMilliseconds` 默认 60000、上限 600000；未定义的字段返回 `-32018 plugin.host.unsupportedOption`；`parentOperationID` 为 1–128 字节字符串，否则 `-32602 plugin.host.invalidParams`。两个方法只对协商到协议 4 且声明了至少一个 call 的后端开放：协议 3 及以下返回 `-32601 plugin.host.unavailable`，未声明 call 返回 `-32010 plugin.host.permissionDenied`；`getCapabilities` 在 `functions` 服务下列出。新增错误码 `plugin.function.notDeclared`、`notExported`、`providerDisabled`、`versionMismatch`、`callCycle`、`depthExceeded`、`tooManyCalls`、`payloadTooLarge`、`providerUnavailable`。信任不传递：被调方用自己的权限执行，调用方不获得任何权限；不参与安装闭包，不做版本求解；远程工作区拒绝。新增 fixtures `fixtures/backend/plugin-functions-caller.plugin.json`、`fixtures/backend/plugin-functions-exporter.plugin.json`，`fixtures/backend/function-protocol.json` 增补用例。macOS 中转调用，Windows 不声明 `pluginFunctions`。官方 Node、Java、Rust SDK 目前只协商函数协议 1–3，还不能调用 `functions/*`。 |
| [#1730](https://github.com/1lck/Lithe-IDEA/pull/1730) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 函数协议 3：编辑器 Provider、编辑器动作第 2 版（API-25、API-27） | 新增权限 `editor.providers`（需要 `backend` 2 和 `documents.read`），只授予宿主方法 `lithe/host/editor/registerProviders`，不授予文档编辑。macOS 路由 7 种类型：`definition`、`hover`、`completion`、`codeLens`、`foldingRange`、`semanticTokens`、`documentLink`；语言服务器优先，插件结果只补充。宿主在 `services.editor.options.providerTypes` 里列出可用类型，类型不在其中时返回 `unsupportedOption`。编辑器动作第 2 版：`services.editor.options.actionIcons` 列出宿主绘制的图标（macOS 绘制 9 个），动作可带最多 4 个 `alternatives`，点击结果可带 `reveal`；宿主未报告 `actionIcons` 时只接受第一版形状。`limits.editorActionSourceBytes` 给出发给收集函数的文档上限（macOS 1 MiB，宿主未报告时 128 KiB，此前固定 128 KiB）。新增 3 个 fixtures（`editor-providers.plugin.json`、`editor-providers.json`、`editor-actions-v2.json`）和 1 条清单拒绝用例（声明 `editor.providers` 但缺 `documents.read`）。这是协议 3 内的增量方法，不升版本号；旧宿主不认识 `editor.providers`，会拒绝该包。 |
| [#1752](https://github.com/1lck/Lithe-IDEA/pull/1752) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | `litheToolchains` 1：写入正式契约 | 删除工具链设计草案 `toolchains/proposal/` 和 `fixtures/toolchains/proposal/`。草案里已实现的部分写进正式契约 `toolchains/README.md`：新增“Provider backend negotiation”一节（`initialize` 里的 `experimental.litheToolchains` 协商、`lsp.toolchainRequest`、超时）和“Error replies”小节（错误回复格式与 11 个稳定失败码的数字码对照）。Core 测试改用内联数据 `toolchain_vectors.rs`。协议行为没有变化。 |
| [#1740](https://github.com/1lck/Lithe-IDEA/pull/1740) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | **`projectGenerators` 2**（API-33） | 生成器新增 `tool` 字段：从插件声明的工具定义的已有安装里选一个（`tool`、`minimumVersion`、`pinAs`、`optional`），模板可用 `${field.<id>.executable}`；`pinAs: "project"` 时计划返回 `toolPins`，宿主把它存为新项目的工具固定。表单输入加 `tools`，结果字段带 `installations`，新增问题码 `toolTooOld`；目录条目带 `tools`（声明该工具的插件）。版本 2 不能用 `runtime` 字段，版本 1 不能用 `tool` 字段。macOS 声明 1、2，Windows 不声明。新增示例 `project-generator-tool-field.plugin.json`、12 个计划用例和 9 个清单拒绝用例。 |
| [#1747](https://github.com/1lck/Lithe-IDEA/pull/1747) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | 清单 schema：`requires.buildSystems` | schema 的枚举补齐到 1–4（Core 从 #1558 起已经支持 3、4，schema 当时只写到 3）。fixtures 按功能分目录，工具链提案移到 `toolchains/proposal/`，契约 README 的能力版本表随之修正。 |

### 2026-11-04

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1724](https://github.com/1lck/Lithe-IDEA/pull/1724) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 包签名 | 新增 `fixtures/signature-vectors.json`，是发布者签名的跨平台测试向量：RFC 8032 测试密钥、带非 ASCII 文件名的固定包和它的规范化载荷。同一 PR 的语言服务器启动失败清理、归档解压加固属于宿主行为，没有改字段。 |
| [#1558](https://github.com/1lck/Lithe-IDEA/pull/1558) | [@Mucheen](https://github.com/Mucheen) | API-29 | **`buildProviders` 7–11**、**`litheBuild` 7–11**、**`buildSystems` 3、4**、**`litheBuildConfiguration` 1** | 7 基于真实来源的原生配置，8 内联测试报告，9 不透明语义测试选择器，10 类型化运行选项与服务/任务运行准备，11 注册表连接调试。`buildSystems` 3 是实例级选项，4 是声明式项目准备角色。 |
| [#1720](https://github.com/1lck/Lithe-IDEA/pull/1720) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-22 | `litheLogging` 1：声明式插件 | 声明式贡献没有自己的进程，发不了 `lithe/log`，改由执行方在 `plugins/<id>/host` 通道代记 `plugins.operation.finished` 和 `plugins.operation.failed`。每次启动语言服务器或调试适配器、每次运行构建模型或依赖命令，各记为一次操作。失败时带一个错误码，例如 `host.planFailed`、`host.executableUnavailable`、`host.launchFailed`。 |
| [#1699](https://github.com/1lck/Lithe-IDEA/pull/1699) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-22 | **`litheLogging` 1** | 插件拥有的每个进程都可以协商 `capabilities.experimental.litheLogging`，然后用通知 `lithe/log` 发结构化诊断。参数只接受固定字段，未知字段会被拒绝。新增 `fixtures/plugin-logging.json`。 |

### 2026-11-03

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1695](https://github.com/1lck/Lithe-IDEA/pull/1695) | [@1lck](https://github.com/1lck) | — | **`keymapImportProviders` 1**、**`litheKeymapImport` 1**、函数协议 3 的宿主方法 | 快捷键导入提供者，需要 `backend` 2 和对应权限。后端在 `initialize` 中选出要启用的提供者，宿主向它发 `lithe/keymap/import`。单独授权后，后端可以调用 `lithe/host/keymapImport/listInputs` 和 `readChunk`。 |
| [#1676](https://github.com/1lck/Lithe-IDEA/pull/1676) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-22 | 启动模板 `${logDir}` | 新增模板变量 `${logDir}`，是当前会话 `plugins/<id>/files/` 的绝对路径，由宿主显式传入 `logDirectory`。语言服务器和插件后端可用，`validationArguments` 不可用。`logDirectory` 缺失、为空或不是绝对路径时返回 `plugin.launch.invalidRequest`。 |

### 2026-11-02

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1661](https://github.com/1lck/Lithe-IDEA/pull/1661) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-23 | `toolDefinitions` 1、`toolchainProviders` 1：宿主现状 | macOS 开始声明这两个能力，用于只读的工具链设置页。Windows 不声明。保存固定版本、`toolchainConsumers` 仍未实现。 |
| [#1653](https://github.com/1lck/Lithe-IDEA/pull/1653) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-24 | 函数协议 3：插件原生终端 | 新增权限 `terminal.create`（需要 `backend` 2），授予宿主方法 `lithe/host/terminal/create`、`show`、`close`、`reopen`、`dispose`、`onExit`。后端创建和管理自己的原生终端：`close` 后按字节预算保留显示历史和启动配置，`reopen` 经重新确认后用原配置启动新进程并恢复历史，`dispose` 彻底释放句柄，`onExit` 订阅每个 generation 的退出结果。只支持 macOS 和本机工作区。 |
| [#1650](https://github.com/1lck/Lithe-IDEA/pull/1650) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 函数协议说明 | 契约改为说明函数协议 1–3，并按实际情况描述官方 SDK 实现的协议。不改字段。 |
| [#1646](https://github.com/1lck/Lithe-IDEA/pull/1646) | [@FxRayHughes](https://github.com/FxRayHughes) | API-19、API-25 | 清单 schema：`settings` 2、`languageServers` 2 | schema 补上 Core 从 #1395 起已经支持的这两个版本。 |

### 2026-11-01

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1643](https://github.com/1lck/Lithe-IDEA/pull/1643) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 契约文档链接 | 工具链提案的链接和 schema `$id` 检查改为符合 lithe-plugin-docs 的同步规则。不改字段。 |
| [#1581](https://github.com/1lck/Lithe-IDEA/pull/1581) | [@FxRayHughes](https://github.com/FxRayHughes) | API-23 | **`toolDefinitions` 1**、**`toolchainProviders` 1**、**`executableResolution` 2**、**`litheToolchains` 1**（提案） | 插件注入的工具链框架。工具定义描述工具的身份、目录布局、发现方式和候选，需要 `executableResolution` 2。工具链提供者需要 `backend` 2，通过 `litheToolchains` 协议回答 `toolchains/query`。同时新增 `proposals/toolchains/` 提案和本机工具链文档 `.lithe/toolchains/local.json` 版本 1。 |

### 2026-10-31

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1617](https://github.com/1lck/Lithe-IDEA/pull/1617) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-25、API-27 | **`languages` 2**、函数协议 3：编辑器动作 | `languages[].grammar` 是包内的 Monarch 语法，`languages[].configuration` 是编辑器配置，用到其中任一项都要声明 `languages` 2。后端用 `lithe/host/editor/registerActions`（API-27）注册不依赖语言服务器的编辑器动作，用户触发时宿主按普通插件函数调用。 |

### 2026-10-30

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1595](https://github.com/1lck/Lithe-IDEA/pull/1595) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-01、API-02、API-04–10、API-15–17、API-37、API-40、API-41 | **`backend` 2**、**`ui` 4**、**`litheFunctions` 3** | `backend` 2：后端可以在 `onWorkspaceOpen` 或插件命令时启动，通过函数协议接收工作区上下文，并可读取 `workspace.metadata`。`ui` 4：不打开工具窗口的后端命令。函数协议 3：工作区上下文与能力发现（`lithe/host/getCapabilities`、`lithe/host/workspace/*`）、宿主通知订阅，以及公共的文档、文件、窗口方法。各方法见[宿主方法（按 API 编号）](#宿主方法按-api-编号)。 |

### 2026-10-28

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1517](https://github.com/1lck/Lithe-IDEA/pull/1517) | [@Mucheen](https://github.com/Mucheen) | API-29、API-30 | **`buildProviders` 2–6**、**`litheBuild` 2–6**、**`buildModelInputs` 1**、**`debugAdapters` 2**、**`litheTestDiscovery` 1** | 2 任务执行（`lithe/build/execute`），3 测试执行和测试发现（`lithe/build/test`、`lithe/build/discoverTests`），4 应用运行准备（`lithe/build/prepareRun`），5 原生调试准备（`lithe/build/prepareDebug`），6 只读模型输入。`buildModelInputs` 1 用于在已启用的工作区插件之间共享公开模型。`debugAdapters` 2 支持绑定调试目标的参数和环境变量。 |
| [#1483](https://github.com/1lck/Lithe-IDEA/pull/1483) | [@Mucheen](https://github.com/Mucheen) | — | **`packageTrust` 1** | 新增 `vendor.bundledSignatureRequirement`：随宿主发布、由发布者签名的插件副本按来源信任。Java Support 从此单独分发。 |

### 2026-10-27

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1525](https://github.com/1lck/Lithe-IDEA/pull/1525) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-30 | `debugAdapters` 1：远程 attach | `attach` 配置的 `targetBindings` 可以带 `connection`（远端主机和端口），`debug.attach` 权限也覆盖远程 attach。适配器仍在本机运行，只有被调试进程在远端。 |
| [#1520](https://github.com/1lck/Lithe-IDEA/pull/1520) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-30 | `debugAdapters` 1：`tcpLanguageServer` 传输 | 新增传输方式 `tcpLanguageServer`：调试适配器由插件自己的语言服务器承载，例如 JDT LS 里的 Java Debug Server。`endpoint` 用 `languageID` 指定语言服务器，用 `command` 指定启动命令。 |
| [#1512](https://github.com/1lck/Lithe-IDEA/pull/1512) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-30 | `debugAdapters` 1：macOS 宿主 | macOS 开始运行调试适配器，插件可热激活。新增 `configurations.launch.executable`：调试目标是适配器不能直接调试的源文件（C、C++、Swift）时，从插件的文件设置或最多 8 个候选路径找可执行文件。新增 `console.commandPrefix` 和 `console.commands[]`，供调试控制台使用。 |

### 2026-10-26

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1510](https://github.com/1lck/Lithe-IDEA/pull/1510) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-23、API-30 | **插件 API 级别 4**、**`debugAdapters` 1**、**`serverProcess` 2** | 新增 `debugAdapters[]`，用来声明 DAP 适配器的启动、传输、配置和权限，需要 API 级别 4。`serverProcess` 2 新增 `runSelectedRuntime`，只用于调试适配器。 |

### 2026-10-24

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1470](https://github.com/1lck/Lithe-IDEA/pull/1470) | [@1lck](https://github.com/1lck) | API-21 | **`backend` 1**、**`litheFunctions` 2** | 插件后端：插件自己的长驻进程，为自己的网页提供服务。函数协议 2 加入宿主服务，即凭据 `lithe/host/credentials/get`、`set`、`delete`；宿主把插件权限允许的服务列表传给后端。 |
| [#1424](https://github.com/1lck/Lithe-IDEA/pull/1424) | [@Mucheen](https://github.com/Mucheen) | API-14 | `ui` 2：Windows 宿主 | Windows 声明式插件宿主上线，用 WebView2 实现包内网页并声明 `ui` 1、2，不声明 `ui` 3。 |
| [#1423](https://github.com/1lck/Lithe-IDEA/pull/1423) | [@arkleselect](https://github.com/arkleselect) | API-13 | **`ui` 3**、**原生 UI 文档 schema 1** | 宿主渲染的原生控件，新增 `native-ui.schema.json`。 |
| [#1452](https://github.com/1lck/Lithe-IDEA/pull/1452) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-33 | **`projectGenerators` 1** | 新建项目窗口里的项目生成器，可以用模板文件、脚手架命令或归档下载生成项目，需要 `project.generate` 权限。 |

### 2026-10-23

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1430](https://github.com/1lck/Lithe-IDEA/pull/1430) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-31 | **`vcsProviders` 1**、**`litheVcs` 1** | VCS 提供者。协议在 `initialize` 的 `experimental.litheVcs` 中协商，定义见主仓库的 `shared/contracts/vcs-provider-v1.md`。 |
| [#1442](https://github.com/1lck/Lithe-IDEA/pull/1442) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 契约发布 | 契约开始发布到 lithe-plugin-docs，fixtures 按能力改名，SDK 移出主仓库。不改字段。 |
| [#1420](https://github.com/1lck/Lithe-IDEA/pull/1420) | [@Mucheen](https://github.com/Mucheen) | API-23、API-29 | **`buildSystems` 2**、**`executableResolution` 1** | 构建系统可以把可执行文件路径交给插件设置来配置。共享的可执行文件候选：统一的候选排序和严格选择。 |

### 2026-10-21

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1395](https://github.com/1lck/Lithe-IDEA/pull/1395) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-19、API-25 | **`languageServers` 2**、**`settings` 2** | `workspaceConfiguration` 用于向语言服务器推送工作区配置。设置 2 新增放置位置（`page`、`section`）、作用域、生效方式，`multiSelect`、`text`、`file`、`directory` 类型和页面按钮；插件可以注册设置页。 |
| [#1382](https://github.com/1lck/Lithe-IDEA/pull/1382) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-29 | `litheBuild` 1 | 构建提供者的请求可以带 `refresh`（用户要求重新加载）和 `tool.executable`（宿主解析出的构建工具绝对路径）。两者都是可选字段，协议仍是版本 1。 |

### 2026-10-20

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1379](https://github.com/1lck/Lithe-IDEA/pull/1379) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 官方 Java SDK | Java 后端 SDK 首发，当时从主仓库发布。契约补充：探测 `java` 运行时版本用 `-version`，Java 8 也能识别。 |
| [#1366](https://github.com/1lck/Lithe-IDEA/pull/1366) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-14 | 网页调用插件函数 | macOS 把每个构建提供者进程作为所属插件的会话启动。插件网页的桥接新增 `functions.list`、`functions.invoke`、`functions.cancel`、`functions.progress`。 |
| [#1365](https://github.com/1lck/Lithe-IDEA/pull/1365) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 官方 Node SDK | Node 后端 SDK 首发，实现函数协议和 `litheBuild` 1。 |
| [#1358](https://github.com/1lck/Lithe-IDEA/pull/1358) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-22 | **`litheFunctions` 1** | 插件函数协议：注册、撤回、调用、结果、进度、取消，在 `initialize` 的 `capabilities.experimental.litheFunctions` 中协商，由 Core 负责路由。 |
| [#1356](https://github.com/1lck/Lithe-IDEA/pull/1356) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-29 | `litheBuild` 1：macOS 宿主 | macOS 开始启动构建提供者并发送 `lithe/build/*` 请求。每个提供者是独立进程，工作区关闭、插件停用或插件包被替换前会停止。 |

### 2026-10-19

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1345](https://github.com/1lck/Lithe-IDEA/pull/1345) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | — | 社区插件安装 | 社区插件除了从第三方市场安装，还可以从本地 ZIP 或解包目录安装：宿主先取私有快照，再请用户确认安装审查。包里有签名文档时仍然必须通过验证。 |
| [#1339](https://github.com/1lck/Lithe-IDEA/pull/1339) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-14 | `ui` 2：宿主现状 | macOS 用 WKWebView 实现包内网页并声明 `ui` 1、2。Windows 当时只声明 `ui` 1，`ui` 2 插件返回 `plugin.api.unsupported`，直到 #1424。 |

### 2026-10-18

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1334](https://github.com/1lck/Lithe-IDEA/pull/1334) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-14 | **`ui` 2** | 工具窗口中的包内网页。 |
| [#1318](https://github.com/1lck/Lithe-IDEA/pull/1318) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-29 | **插件 API 级别 3**（`requires`）、**`buildSystems` 1**、**`buildProviders` 1**、**`litheBuild` 1**、**`litheBuildModel` / `litheDependencies` / `litheTestResults` 1** | 从级别 3 起，插件要按能力声明版本，首批 8 个能力都是版本 1（见上方能力表）；市场索引里级别 3 的版本必须带 `requires`。新增声明式构建工具插件，以及构建提供者协议 `litheBuild` 1（`lithe/build/model`、`lithe/build/dependencies`）。 |

### 2026-10-15

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1206](https://github.com/1lck/Lithe-IDEA/pull/1206) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-03 | 插件 API 级别 2：`ui` 与 `ui.contribute` | 级别 2 加入第一阶段工作台 UI（宿主渲染的工具窗口和命令）和 `ui.contribute` 权限。`languages` 不再必填，插件可以只提供 UI。 |

### 2026-10-14

| PR | 修改人 | API 编号 | 协议 / 能力 | 改了什么 |
| --- | --- | --- | --- | --- |
| [#1214](https://github.com/1lck/Lithe-IDEA/pull/1214) | [@xiaoyumuxi](https://github.com/xiaoyumuxi) | API-19、API-23、API-25 | **插件 API 级别 2** | 新增启动模板（`arguments` 可用变量）、`runtimeDependencies` 的 `java`、`initializationOptions`、`hostIntegration`（仅官方插件）、包内或已发现运行时启动（`runtime`、`packagedRuntime`）和插件设置。Java Support 改为声明式官方插件。 |
| [#1170](https://github.com/1lck/Lithe-IDEA/pull/1170) | [@1lck](https://github.com/1lck) | API-23、API-25 | **契约 v1**：**插件 API 级别 1**、**清单 `schemaVersion` 2**、**市场索引 `schemaVersion` 1** | 契约 v1 进入 `preview`：语言、语言服务器（`packaged` 和 `systemCommand` 启动）、`node` 和 `go` 运行时依赖、运行和测试提供者，以及市场索引和标准用例。 |

## 维护

每次在本仓库同步新的契约版本（`node scripts/contract.mjs sync --revision <commit>`）时，都要在同一个 PR 里更新本页：

1. 在 Lithe 主仓库列出两次同步之间改动契约的 PR：

   ```sh
   git log origin/preview --first-parent --format='%h %cd %s' --date=short <上次记录到的 commit>..<本次 commit> -- shared/contracts/plugins/v1 rust/lithe-core/src/plugins/capabilities.rs rust/lithe-core/src/lsp/interface/plugin_function.rs
   ```

2. 按日期把这些 PR 补到时间线最上面。新增的版本号要加粗，并写进“按协议查首次出现”的表。
3. “API 编号”填这个 PR 涉及的全部 API-01–44 编号，取自 PR 描述里的“原计划编号”。PR 没写时按 [API 实施计划](https://github.com/1lck/Lithe-IDEA/blob/preview/docs/development/macos-plugin-api-plan.md) 判断，不要自编新号；不属于任何编号的写 —。
4. PR 新增了宿主方法或通知时，每个方法在“宿主方法（按 API 编号）”表里加一行，写明 API 编号、协议、日期和 PR；同时在 [API 总表](api-catalog.md#已公开的宿主方法)中英两张方法表里各加一行，写明用途、权限和协议。同一个 API 下的方法分几个 PR 加入时，各自登记自己的 PR。
5. “修改人”填原 PR 作者的 GitHub 账号，写成 `[@账号](https://github.com/账号)`。账号可以这样查：

   ```sh
   gh pr view <PR 编号> --repo 1lck/Lithe-IDEA --json author --jq .author.login
   ```

6. 把开头的“记录到”改成本次同步的 commit。
7. 同时更新[能力与协议版本对照](capabilities-and-versions.md)和受影响的功能页。
8. 在同步 PR 的描述里列出本次登记的 PR，并直接写 `@账号` 提到每位修改人，请他们核对自己那一行。本页里的 `[@账号](…)` 只是链接，不会通知任何人。
