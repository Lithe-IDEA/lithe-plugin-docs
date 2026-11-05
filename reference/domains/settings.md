# 插件设置 / Plugin settings

[中文](#中文) · [English](#english)

## 中文

插件在清单里声明用户可以修改的选项。宿主负责显示控件、保存值、区分全局和项目两层，并把生效值传给 Core；Core 合并后展开到启动计划、`initializationOptions` 或语言服务器的 `workspaceConfiguration`。插件不写设置页代码，宿主也不需要认识插件。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `settings`、`settingActions`（`settings` 2） |
| 能力版本 | `settings` 1–2；相关的 `languageServers` 2、`executableResolution` 1–2，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 协议 | 没有专用协议；`settingActions` 调用的是插件自己的[插件函数](backend.md) |
| API 编号 | [API-19](../api-catalog.md#现在能用的-api)：声明式设置。运行期读取、查看来源、修改设置和变化事件仍在计划中 |
| 权限 | 不需要额外权限；设置不会改变运行的程序，所以不进入安装审查的程序列表 |
| 激活事件 | 不需要；值随下一次启动、模型重载、运行或 `immediate` 推送生效 |
| Windows | 宿主声明 `settings` 1、2 |

### 版本对照

| 版本 | 最低级别 | 新增 |
| --- | --- | --- |
| `settings` 1 | 2 | 插件级设置，所有工作区共享；类型 `integer`（带 `minimum`、`maximum`、`unit`）、`enum`（1–16 个选项）、`boolean`；值变化时重启插件的语言服务器 |
| `settings` 2 | 3 | 类型 `multiSelect`、`text`（必须带 `pattern`）、`file`、`directory`；`scope`（`global` 或 `project`，项目值存在 `.lithe/plugin-settings.json`）；`effect`（`restart`、`immediate`、`reload`、`nextRun`）；`page` 放到某个语言或构建系统的设置页；`section` 分组；最多 16 个 `settingActions` 按钮 |
| `languageServers` 2 | 3 | `workspaceConfiguration`：`immediate` 设置不重启进程，直接推送给运行中的语言服务器 |
| `executableResolution` 1 | 3 | 可执行文件候选可以引用一个全局 `file` 设置（`effect` 为 `reload` 或 `nextRun`），配置了的路径失败时不回退 |
| `executableResolution` 2 | 3 | 工具定义候选和项目/子项目工具链绑定；macOS 只有只读设置页 |

生效顺序是项目值（只对 `scope: "project"`）、全局值、默认值；存储的值不再被声明接受时跳过，不会让启动失败。

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Plugin settings (API 2)](../../contract/v1/README.md#plugin-settings-api-2) | `settings` 1 的字段、类型和模板 `${setting:<id>}` |
| [Settings v2: placement, scope, effect, and page buttons](../../contract/v1/README.md#settings-v2-placement-scope-effect-and-page-buttons) | `settings` 2 的类型、层级、生效时机、放置位置和按钮 |
| [Workspace configuration (`languageServers` 2)](../../contract/v1/README.md#workspace-configuration-languageservers-2) | `workspaceConfiguration` 的展开和推送 |
| [Shared executable candidates (`executableResolution` 1/2)](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | 用设置选择工具路径、候选顺序和严格选择 |
| [Launch templates (API 2)](../../contract/v1/README.md#launch-templates-api-2) | 设置值在模板里的替换规则 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`settings/settings-v2.plugin.json`](../../contract/v1/fixtures/settings/settings-v2.plugin.json) | `settings` 2：放在 Elixir 语言页和 Mix 构建系统页的设置、`immediate` 设置、`settingActions` 按钮 |
| [`languages/node-runtime.plugin.json`](../../contract/v1/fixtures/languages/node-runtime.plugin.json) | `settings` 1：级别 2 的插件设置 |
| [`build-systems/build-tool-path.plugin.json`](../../contract/v1/fixtures/build-systems/build-tool-path.plugin.json) | `settings` 2 加 `executableResolution` 1：用 `file` 设置选择构建工具 |
| [`debug-adapters/debug-adapter.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) | `settings` 2：调试适配器用 `file` 设置（`nextRun`）选择要调试的程序 |

设置相关的拒绝用例在 [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json)。

### 相关指南

- [从零构建一个完整生命周期的 LSP 插件](../../guides/lsp-plugin-lifecycle.md)
- [为语言插件接入构建、运行与测试](../../guides/build-tool-integration.md)

---

## English

A plugin declares the options a user may change in its manifest. The host draws the controls, stores the values, keeps the global and project layers apart, and passes the effective values to Core; Core merges them and expands them into launch plans, `initializationOptions`, or a language server's `workspaceConfiguration`. The plugin writes no settings-page code, and the host never needs to know the plugin.

| Item | Value |
| --- | --- |
| Manifest fields | `settings`, `settingActions` (`settings` 2) |
| Capability versions | `settings` 1–2; related `languageServers` 2 and `executableResolution` 1–2; see [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Protocols | None of its own; `settingActions` call the plugin's own [plugin functions](backend.md) |
| API IDs | [API-19](../api-catalog.md#apis-usable-now): declarative settings. Runtime reads, source inspection, writes, and change events are still planned |
| Permissions | None; settings never change which program runs, so they are not in the install review's program list |
| Activation | Not needed; values take effect at the next start, model reload, run, or `immediate` push |
| Windows | Advertises `settings` 1 and 2 |

### Versions compared

| Version | Lowest level | Adds |
| --- | --- | --- |
| `settings` 1 | 2 | Plugin-wide settings shared by every workspace; types `integer` (with `minimum`, `maximum`, `unit`), `enum` (1–16 options), `boolean`; a change restarts the plugin's language servers |
| `settings` 2 | 3 | Types `multiSelect`, `text` (requires `pattern`), `file`, `directory`; `scope` (`global` or `project`, project values in `.lithe/plugin-settings.json`); `effect` (`restart`, `immediate`, `reload`, `nextRun`); `page` on a language or build-system settings page; `section` grouping; up to 16 `settingActions` buttons |
| `languageServers` 2 | 3 | `workspaceConfiguration`: `immediate` settings reach the running language server without a restart |
| `executableResolution` 1 | 3 | An executable candidate may reference a global `file` setting (`effect` `reload` or `nextRun`); a configured path that fails never falls back |
| `executableResolution` 2 | 3 | Tool-definition candidates and project/subproject toolchain bindings; macOS has read-only settings pages |

Values resolve as project (only for `scope: "project"`), then global, then default; a stored value the declaration no longer accepts is skipped and never fails a launch.

### Contract sections

| Section | Content |
| --- | --- |
| [Plugin settings (API 2)](../../contract/v1/README.md#plugin-settings-api-2) | `settings` 1 fields, types, and `${setting:<id>}` templates |
| [Settings v2: placement, scope, effect, and page buttons](../../contract/v1/README.md#settings-v2-placement-scope-effect-and-page-buttons) | `settings` 2 types, layers, effect timing, placement, and buttons |
| [Workspace configuration (`languageServers` 2)](../../contract/v1/README.md#workspace-configuration-languageservers-2) | Expanding and pushing `workspaceConfiguration` |
| [Shared executable candidates (`executableResolution` 1/2)](../../contract/v1/README.md#shared-executable-candidates-executableresolution-12) | Selecting a tool path with a setting, candidate order, strict selection |
| [Launch templates (API 2)](../../contract/v1/README.md#launch-templates-api-2) | How setting values are substituted in templates |

### Examples and cases

| File | Content |
| --- | --- |
| [`settings/settings-v2.plugin.json`](../../contract/v1/fixtures/settings/settings-v2.plugin.json) | `settings` 2: settings on the Elixir language page and the Mix build-system page, an `immediate` setting, `settingActions` buttons |
| [`languages/node-runtime.plugin.json`](../../contract/v1/fixtures/languages/node-runtime.plugin.json) | `settings` 1: plugin settings at level 2 |
| [`build-systems/build-tool-path.plugin.json`](../../contract/v1/fixtures/build-systems/build-tool-path.plugin.json) | `settings` 2 with `executableResolution` 1: a `file` setting selects the build tool |
| [`debug-adapters/debug-adapter.plugin.json`](../../contract/v1/fixtures/debug-adapters/debug-adapter.plugin.json) | `settings` 2: a debug adapter uses a `file` setting (`nextRun`) to pick the program to debug |

Settings rejections are in [`manifest/invalid-manifests.json`](../../contract/v1/fixtures/manifest/invalid-manifests.json).

### Related guides

- [Building an LSP plugin end to end](../../guides/lsp-plugin-lifecycle.md)
- [Build, run, and test for language plugins](../../guides/build-tool-integration.md)
