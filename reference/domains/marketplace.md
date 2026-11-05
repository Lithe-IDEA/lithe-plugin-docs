# 插件市场 / Marketplace

[中文](#中文) · [English](#english)

## 中文

插件市场是一份 JSON 索引，列出插件和各版本的发布压缩包。宿主负责获取、缓存和下载；索引是否有效、某个版本在当前平台能不能装、安装审查里列出哪些权限和程序，都由 Rust Core 的 `plugins.marketplace.*` 命令决定，所以各个产品看到同样的目录和同样的审查。

| 项目 | 内容 |
| --- | --- |
| 索引格式 | [`plugin-index.schema.json`](../../contract/v1/plugin-index.schema.json)，`schemaVersion` 固定为 `1`，根对象只有 `schemaVersion` 和 `plugins` |
| 社区索引文件名 | 仓库根目录的 `lithe-plugin-index.json` |
| Core 命令 | `plugins.marketplace.resolveSource`、`plugins.marketplace.validateIndex`、`plugins.marketplace.installReview` |
| 能力版本 | 没有单独的能力版本；版本的可用性按 `pluginAPIVersion` 和 `requires` 判断，见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| API 编号 | 不在 API-01–44 内（安装和打包层面的契约），见 [API 总表](../api-catalog.md#计划中的-api)下方的说明 |
| 信任 | 见[包信任](package-trust.md)；社区包使用 `vendor.signatureRequirement: "none"` |
| Windows | 没有声明式插件的市场浏览和安装入口，只能从磁盘安装；不调用 `validateIndex` |
| 官方 SDK | 与 SDK 无关 |

### 版本对照

| 版本 | 内容 | Core | macOS | Windows |
| --- | --- | --- | --- | --- |
| 索引 `schemaVersion` 1 | 来源、条目字段、下载地址规则 | 校验；其他 `schemaVersion` 整份拒绝（`plugin.index.schemaUnsupported`） | 支持：官方、Lithe Store 和第三方来源 | 不支持 |
| 条目里的 `pluginAPIVersion` 1 | 任何平台都可用 | 支持 | 支持 | 不适用（无市场） |
| 条目里的 `pluginAPIVersion` 2 | 只在 `darwin-*` 平台标为可用 | 支持 | 支持 | 不适用（无市场） |
| 条目里的 `pluginAPIVersion` 3 | 必须写 `requires`，按 `requires` 和宿主能力表判断；契约不认识的能力 ID 允许出现，但该版本不可用 | 支持 | 支持 | 不适用（无市场） |
| 条目里的 `pluginAPIVersion` 4 | 写了 `requires` 时同级别 3；索引 schema 不强制要求，不写时 Core 按级别 2 的规则只在 `darwin-*` 标为可用。发布时应写上包清单的 `requires` | 支持 | 支持 | 不适用（无市场） |
| 条目里的 `pluginAPIVersion` 5 及以上 | Core 不认识，该版本不可用（`plugin.index.apiUnsupported`） | 跳过 | 跳过 | 不适用（无市场） |
| `installReview` | 权限和程序清单、新增程序对照 | 支持 | 市场和磁盘安装都使用 | 磁盘安装使用 |

### 来源

| 来源 | 地址 | 规则 |
| --- | --- | --- |
| 官方 | Lithe 仓库的 `Plugins/marketplace/index.json` | 应用内带一份只读副本供离线使用；刷新时从 `https://raw.githubusercontent.com/Lithe-IDEA/Lithe-IDEA/HEAD/Plugins/marketplace/index.json` 获取 |
| Lithe Store | [Lithe-IDEA/Lithe-store](https://github.com/Lithe-IDEA/Lithe-store) 的 `lithe-plugin-index.json` | 预置的社区来源，各方面都按第三方处理（社区信任、拒绝 `dev.lithe` ID）；默认启用，可以删除，删除后宿主记住，不会再加回来 |
| 第三方 | 用户添加 | `resolveSource` 把用户粘贴的内容变成要获取的索引地址，见下表 |

| 用户输入 | 结果 |
| --- | --- |
| https 索引地址 | 原样使用，去掉 `#` 片段 |
| `https://github.com/<owner>/<repo>` | 默认分支（`HEAD`）根目录的 `lithe-plugin-index.json` |
| `…/blob/<ref>/<path>` | 对应的 raw 文件 |
| http、带账号密码的地址、其他 GitHub 页面 | `plugin.source.invalid` |

获取到的索引缓存在平台缓存或 Application Support，不写进安装目录。

### 索引条目

| 字段 | 规则 |
| --- | --- |
| `id` | 插件 ID，必须等于包里 `plugin.json` 的 `id` |
| `name`、`author`、`description`、`license` | 展示信息，不能为空 |
| `localizedDescriptions` | 可选，`description` 的翻译，键是语言标签（如 `zh-Hans`）；宿主按界面语言选，找不到时用 `description` |
| `repository` | `https://github.com/<owner>/<repo>`，拥有该插件的发布和问题跟踪 |
| `homepage` | 可选，https 地址 |
| `tags` | 可选，单个词，Core 转小写并排序 |
| `icon` | 可选，同一仓库在 `raw.githubusercontent.com` 上的文件 |
| `versions[]` | `version`、必填的 `pluginAPIVersion`、`requires`（级别 1、2 不能写；级别 3 必填；级别 4 可写；内容等于包清单的 `requires`）、`releaseDate`（`YYYY-MM-DD`）、`downloads[]` |
| `downloads[]` | `platforms`（如 `darwin-arm64`、`darwin-x64`、`windows-x64`、`windows-arm64`）、`downloadURL`、压缩包的小写十六进制 `sha256`；一个版本里每个平台最多出现在一个下载项 |

`downloadURL` 必须是条目自己仓库的 GitHub Release `.zip` 资源（`https://github.com/<owner>/<repo>/releases/download/<tag>/<asset>.zip`，owner 和仓库名不区分大小写），索引不能把插件指向别的项目的发布。

### 校验与可用性

`validateIndex` 的输入是 `{index, sourceKind: "official" | "thirdParty", platform, hostCapabilities?}`。索引根对象格式错误或 `schemaVersion` 不支持时整份拒绝；否则返回 `{plugins, rejected}`：

| 输出 | 内容 |
| --- | --- |
| `rejected` | 按索引顺序列出被拒绝的条目和错误：格式错误 `plugin.index.invalid`；第三方来源列出官方命名空间 ID `plugin.index.officialNamespace`；重复 ID 保留第一个，之后的 `plugin.index.duplicate` |
| `plugins` | 按 ID 排序；每个条目规范化，`versions` 从新到旧，带 `platforms`、`latestVersion`、`latestReleaseDate` 和 `availability` |

| `availability` | 条件 |
| --- | --- |
| `{status: "available", version, pluginAPIVersion, releaseDate, downloadURL, sha256}` | 当前平台有压缩包、宿主能运行的最新版本：级别 1；`darwin-*` 上的级别 2 和没写 `requires` 的级别 4；写了 `requires` 的版本里每项能力都被宿主支持 |
| `{status: "unsupported", code: "plugin.index.apiUnsupported", platform, message}` | 列出当前平台的版本都需要更新的插件 API 或宿主不支持的能力 |
| `{status: "unsupported", code: "plugin.index.platformUnsupported", platform, message}` | 没有版本列出当前平台；宿主显示“不支持此平台”而不是安装按钮 |

### 信任与安装审查

| 安装来源 | 流程 |
| --- | --- |
| 官方 | 只有来自官方来源、并通过 Lithe 发布者签名的条目才算官方 |
| 社区市场 | 第三方来源的条目：用索引的 SHA-256 校验压缩包（`plugin.archive.checksumMismatch`）→ 校验声明和包 → `installReview` → 显示审查 → 用户确认后安装。索引校验和只证明与那份索引一致，不代表发布者身份 |
| 社区本地磁盘 | 用户选的 ZIP 或解压目录没有索引和仓库：先复制私有快照，再用同一个 Core 命令校验和审查快照；本地校验和不能显示成仓库匹配 |

`installReview` 的输入是 `{manifest, expectedID?, expectedVersion?, approvedPrograms?, hostCapabilities?}`，返回 `{id, version, permissions, programs, newPrograms}`。ID 或版本与索引条目不符时返回 `plugin.package.mismatch`；级别 3 的包要求宿主没有的能力时返回 `plugin.api.unsupported`。`programs` 列出宿主可能为插件启动的全部程序，按 `kind`、语言和工具排序：

| `kind` | 含义 |
| --- | --- |
| `packagedLanguageServer`、`systemLanguageServer`、`runtimeLanguageServer` | 包内可执行文件、宿主发现的命令，或由运行时启动的包内入口，带启动参数和校验探针 |
| `buildSystem` | 构建系统能规划的每条命令；`provider` 是把构建请求发给语言服务器的共享形式 |
| `packagedBuildProvider`、`systemBuildProvider`、`runtimeBuildProvider` | 有独立进程的构建提供者，带 `buildSystemID`、`executesProjectCode`、`mayUseNetwork` |
| `packagedVcsProvider`、`systemVcsProvider`、`runtimeVcsProvider` | VCS 提供者进程，带 `vcs`、`vcsTool`、`mayUseNetwork` |
| `runtime` | 宿主发现并交给语言服务器、构建提供者或 VCS 提供者的运行时 |
| `run`、`test` | 运行和测试命令及参数模板 |
| `projectGenerator` | 项目生成器的 `command` 步骤；`files`、`archive` 步骤不启动程序 |

每项都带决定实际运行内容的全部字段，所以只改参数、参数模板或校验探针的更新也算新程序。宿主显示全部权限和完整程序清单，并把每个批准的程序按规范 JSON 原样保存。

| 情形 | 是否需要用户确认 |
| --- | --- |
| 本地磁盘：首次安装、更新、重装、同版本替换 | 每次都要，即使程序和权限都没变；`newPrograms` 为空也不能免确认 |
| 市场：首次安装、没有之前的批准记录、`newPrograms` 非空 | 要 |
| 市场：更新且没有新程序 | 可以沿用记录的批准 |

同一个 ID 出现在多个来源时，用户在安装时选择来源，之后只从这个来源更新。本地磁盘安装不是市场来源，也不会自动订阅更新。

### 宿主现状

| 宿主 | 现状 |
| --- | --- |
| macOS | 官方、Lithe Store 和第三方来源；市场安装和磁盘安装（`MacPluginDiskPackageImporter`）。磁盘快照在 `fileStorage.temporaryDirectory()` 下的 `lithe-plugin-disk-<UUID>/`（ZIP 为 `archive.zip`，文件在 `package/`），目录权限 `0700`，复制深度最多 64 层。安装来源分别记为 `PluginDiskProvenance` 和 `PluginMarketplaceProvenance`，每次扫描、暂存和回退都复查内容摘要 |
| Windows | 没有市场，不调用 `validateIndex`。磁盘安装把快照放在应用本地数据目录的 `plugin-temp/lithe-plugin-disk-<id>`，用 `installReview` 生成审查，确认时只比对 `plugin.json`，不记录已批准的程序和来源 |

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Marketplace](../../contract/v1/README.md#marketplace) | 市场的分工 |
| [Sources](../../contract/v1/README.md#sources) | 官方、Lithe Store、第三方来源和 `resolveSource` |
| [Index entries](../../contract/v1/README.md#index-entries) | 条目字段和下载地址规则 |
| [Validation and availability](../../contract/v1/README.md#validation-and-availability) | `validateIndex` 的输入输出和可用性 |
| [Trust and install review](../../contract/v1/README.md#trust-and-install-review) | 三条安装路径、`installReview`、程序种类和确认规则 |
| [Local disk snapshots and installation provenance (macOS)](../../contract/v1/README.md#local-disk-snapshots-and-installation-provenance-macos) | macOS 磁盘快照、来源记录、清理，以及 Windows 的差距 |
| [Existing community store and Lua template](../../contract/v1/README.md#existing-community-store-and-lua-template) | 给 Lua 模板加界面并发布到 Lithe Store 的步骤 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`marketplace/index.json`](../../contract/v1/fixtures/marketplace/index.json) | 5 个条目：两个版本的 Zig 插件、含未来级别版本的官方命名空间插件、只有未来级别的插件、三个级别 3 版本的 Elixir 插件、重复的 Zig 条目 |
| [`marketplace/index-cases.json`](../../contract/v1/fixtures/marketplace/index-cases.json) | 6 个 `validateIndex` 用例：Apple 芯片上的官方来源、第三方来源不能列官方命名空间、Windows 选自己的压缩包、按宿主能力退回到较旧版本、Core 不认识的能力永远不可用 |
| [`marketplace/invalid-index-entries.json`](../../contract/v1/fixtures/marketplace/invalid-index-entries.json) | 24 个条目错误用例：23 个 `plugin.index.invalid`，1 个 `plugin.index.officialNamespace` |
| [`marketplace/install-reviews.json`](../../contract/v1/fixtures/marketplace/install-reviews.json) | 19 个 `installReview` 用例：首次安装、无变化更新、改了参数或探针需要重新批准、包与条目不符、官方命名空间的未签名包、构建插件的能力要求 |
| [`marketplace/source-urls.json`](../../contract/v1/fixtures/marketplace/source-urls.json) | 11 个 `resolveSource` 用例：5 个解析成功，6 个 `plugin.source.invalid` |

### 相关指南

- [从零构建一个完整生命周期的 LSP 插件](../../guides/lsp-plugin-lifecycle.md)：发布、更新与回退演练，社区索引 `lithe-plugin-index.json`

---

## English

A marketplace is a JSON index that lists plugins and the release archives of each version. Hosts fetch, cache, and download; Rust Core's `plugins.marketplace.*` commands decide whether an index is valid, whether a version can be installed on the current platform, and which permissions and programs the install review lists, so every product shows the same catalog and the same review.

| Item | Details |
| --- | --- |
| Index format | [`plugin-index.schema.json`](../../contract/v1/plugin-index.schema.json); `schemaVersion` is always `1`, and the root has only `schemaVersion` and `plugins` |
| Community index file name | `lithe-plugin-index.json` at the repository root |
| Core commands | `plugins.marketplace.resolveSource`, `plugins.marketplace.validateIndex`, `plugins.marketplace.installReview` |
| Capability version | None of its own; a version's availability follows its `pluginAPIVersion` and `requires`. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| API number | Outside API-01–44 (an installation and packaging contract); see the note under the [API catalog](../api-catalog.md#planned-apis) |
| Trust | See [Package trust](package-trust.md); community packages use `vendor.signatureRequirement: "none"` |
| Windows | No marketplace browser or install path for declarative plugins; installation is from disk only, and `validateIndex` is never called |
| Official SDKs | Not SDK-related |

### Versions compared

| Version | Contents | Core | macOS | Windows |
| --- | --- | --- | --- | --- |
| Index `schemaVersion` 1 | Sources, entry fields, download URL rules | Validates; any other `schemaVersion` rejects the whole index (`plugin.index.schemaUnsupported`) | Supported: official, Lithe Store, and third-party sources | Not supported |
| Entry `pluginAPIVersion` 1 | Available on every platform | Supported | Supported | Not applicable (no marketplace) |
| Entry `pluginAPIVersion` 2 | Marked available only on `darwin-*` platforms | Supported | Supported | Not applicable (no marketplace) |
| Entry `pluginAPIVersion` 3 | `requires` is required; decided by `requires` and the host capability table; capability IDs the contract does not define are allowed but make the version unavailable | Supported | Supported | Not applicable (no marketplace) |
| Entry `pluginAPIVersion` 4 | With `requires`, as level 3; the index schema does not require it, and without it Core applies the level 2 rule and marks the version available only on `darwin-*`. Publishers should list the package manifest's `requires` | Supported | Supported | Not applicable (no marketplace) |
| Entry `pluginAPIVersion` 5 and above | Unknown to Core; the version is unavailable (`plugin.index.apiUnsupported`) | Skipped | Skipped | Not applicable (no marketplace) |
| `installReview` | Permission and program lists, new-program comparison | Supported | Used by marketplace and disk installs | Used by disk installs |

### Sources

| Source | Location | Rules |
| --- | --- | --- |
| Official | `Plugins/marketplace/index.json` in the Lithe repository | A read-only copy ships with the application for offline use; refreshing fetches `https://raw.githubusercontent.com/Lithe-IDEA/Lithe-IDEA/HEAD/Plugins/marketplace/index.json` |
| Lithe Store | `lithe-plugin-index.json` in [Lithe-IDEA/Lithe-store](https://github.com/Lithe-IDEA/Lithe-store) | A preset community source treated as third-party in every respect (community trust, `dev.lithe` IDs refused); enabled by default and removable, and the host remembers the removal instead of adding it back |
| Third-party | Added by the user | `resolveSource` turns what the user pasted into the index URL to fetch; see the table below |

| User input | Result |
| --- | --- |
| An https index URL | Used as is, with the `#` fragment removed |
| `https://github.com/<owner>/<repo>` | `lithe-plugin-index.json` at the root of the default branch (`HEAD`) |
| `…/blob/<ref>/<path>` | The matching raw file |
| Plain http, embedded credentials, other GitHub pages | `plugin.source.invalid` |

Fetched indexes are cached in platform cache or Application Support storage, never inside the installation.

### Index entries

| Field | Rules |
| --- | --- |
| `id` | Plugin ID; must equal `id` in the package's `plugin.json` |
| `name`, `author`, `description`, `license` | Display metadata; must not be empty |
| `localizedDescriptions` | Optional translations of `description`, keyed by language tag such as `zh-Hans`; hosts pick the interface language and fall back to `description` |
| `repository` | `https://github.com/<owner>/<repo>` that owns the plugin's releases and issue tracker |
| `homepage` | Optional https URL |
| `tags` | Optional single words, lowercased and sorted by Core |
| `icon` | Optional `raw.githubusercontent.com` file of the same repository |
| `versions[]` | `version`, required `pluginAPIVersion`, `requires` (not allowed at levels 1 and 2, required at level 3, optional at level 4; it equals the package manifest's `requires`), `releaseDate` (`YYYY-MM-DD`), and `downloads[]` |
| `downloads[]` | `platforms` (for example `darwin-arm64`, `darwin-x64`, `windows-x64`, `windows-arm64`), `downloadURL`, and the archive's lowercase hex `sha256`; a platform may appear in at most one download of a version |

A `downloadURL` must be a `.zip` GitHub Release asset of the entry's own repository (`https://github.com/<owner>/<repo>/releases/download/<tag>/<asset>.zip`, owner and repository compared case-insensitively), so an index cannot point a plugin at another project's release.

### Validation and availability

`validateIndex` takes `{index, sourceKind: "official" | "thirdParty", platform, hostCapabilities?}`. A malformed index root or an unsupported `schemaVersion` rejects the whole index; otherwise it returns `{plugins, rejected}`:

| Output | Contents |
| --- | --- |
| `rejected` | Refused entries in index order with their error: malformed entries `plugin.index.invalid`; an official-namespace ID from a third-party source `plugin.index.officialNamespace`; a repeated ID keeps the first listing and refuses later ones with `plugin.index.duplicate` |
| `plugins` | Sorted by ID; each entry is normalized, lists `versions` newest first, and carries `platforms`, `latestVersion`, `latestReleaseDate`, and `availability` |

| `availability` | When |
| --- | --- |
| `{status: "available", version, pluginAPIVersion, releaseDate, downloadURL, sha256}` | The newest version with this platform's archive that the host can run: level 1; level 2, and level 4 without `requires`, on `darwin-*`; a version with `requires` when every capability in it is hosted |
| `{status: "unsupported", code: "plugin.index.apiUnsupported", platform, message}` | Every version listing this platform needs a newer plugin API or a capability the host does not run |
| `{status: "unsupported", code: "plugin.index.platformUnsupported", platform, message}` | No version lists this platform; hosts show "not supported on <platform>" instead of an install action |

### Trust and install review

| Install route | Flow |
| --- | --- |
| Official | An entry is official only when it comes from the official source and its package passes the Lithe publisher signature |
| Community marketplace | Entries from third-party sources: check the archive against the index SHA-256 (`plugin.archive.checksumMismatch`) → validate the declaration and package → `installReview` → show the review → install after the user confirms. The index checksum proves a match to that index's bytes, not an authenticated publisher |
| Community local disk | A user-selected ZIP or unpacked directory has no index or repository: take a private snapshot first, then validate and review the snapshot with the same Core command; a local checksum is never shown as a repository match |

`installReview` takes `{manifest, expectedID?, expectedVersion?, approvedPrograms?, hostCapabilities?}` and returns `{id, version, permissions, programs, newPrograms}`. An ID or version that differs from the index entry returns `plugin.package.mismatch`; a level 3 package that requires a capability the host lacks returns `plugin.api.unsupported`. `programs` lists everything the host may start for the plugin, sorted by `kind`, language, and tool:

| `kind` | Meaning |
| --- | --- |
| `packagedLanguageServer`, `systemLanguageServer`, `runtimeLanguageServer` | An executable in the package, a command found by host discovery, or a package entry started by a runtime, with launch arguments and validation probe |
| `buildSystem` | Every command line the build system can plan; `provider` is the shared form that sends build requests to a language server |
| `packagedBuildProvider`, `systemBuildProvider`, `runtimeBuildProvider` | A build provider with a process of its own, with `buildSystemID`, `executesProjectCode`, and `mayUseNetwork` |
| `packagedVcsProvider`, `systemVcsProvider`, `runtimeVcsProvider` | A VCS provider process, with `vcs`, `vcsTool`, and `mayUseNetwork` |
| `runtime` | A runtime the host discovers and passes to a language server, build provider, or VCS provider |
| `run`, `test` | Run and test commands with their argument templates |
| `projectGenerator` | A `command` step of a project generator; `files` and `archive` steps start no program |

Each entry carries every field that decides what actually runs, so an update that only changes arguments, argument templates, or the validation probe is a new program. Hosts show all permissions and the complete program list, and persist each approved program as canonical JSON without dropping fields.

| Case | Needs user confirmation |
| --- | --- |
| Local disk: first install, update, reinstall, same-version replacement | Always, even when programs and permissions are unchanged; an empty `newPrograms` never waives it |
| Marketplace: first install, no previous approval, or nonempty `newPrograms` | Yes |
| Marketplace: update with no new programs | May reuse the recorded approval |

When several sources list the same ID, the user chooses the source at install time and later updates come only from it. A local disk installation is never a marketplace source or an automatic update subscription.

### Host status

| Host | Status |
| --- | --- |
| macOS | Official, Lithe Store, and third-party sources; marketplace and disk installation (`MacPluginDiskPackageImporter`). Disk snapshots go to `lithe-plugin-disk-<UUID>/` under `fileStorage.temporaryDirectory()` (`archive.zip` for a ZIP, files in `package/`), with mode `0700` and a copy depth of at most 64. Provenance is recorded as `PluginDiskProvenance` or `PluginMarketplaceProvenance`, and the content digest is rechecked on every scan, staging, and rollback |
| Windows | No marketplace and no `validateIndex` call. Disk installation puts the snapshot in `plugin-temp/lithe-plugin-disk-<id>` under the application's local data directory, builds the review with `installReview`, checks only `plugin.json` at confirmation, and records no approved programs or provenance |

### Contract sections

| Section | Covers |
| --- | --- |
| [Marketplace](../../contract/v1/README.md#marketplace) | Division of work between hosts and Core |
| [Sources](../../contract/v1/README.md#sources) | Official, Lithe Store, and third-party sources, and `resolveSource` |
| [Index entries](../../contract/v1/README.md#index-entries) | Entry fields and download URL rules |
| [Validation and availability](../../contract/v1/README.md#validation-and-availability) | `validateIndex` input, output, and availability |
| [Trust and install review](../../contract/v1/README.md#trust-and-install-review) | The three install routes, `installReview`, program kinds, and confirmation rules |
| [Local disk snapshots and installation provenance (macOS)](../../contract/v1/README.md#local-disk-snapshots-and-installation-provenance-macos) | macOS disk snapshots, provenance records, cleanup, and the Windows gap |
| [Existing community store and Lua template](../../contract/v1/README.md#existing-community-store-and-lua-template) | Steps to add a view to the Lua template and publish it to Lithe Store |

### Examples and cases

| File | Contents |
| --- | --- |
| [`marketplace/index.json`](../../contract/v1/fixtures/marketplace/index.json) | 5 entries: a Zig plugin with two versions, an official-namespace plugin with a future-level version, a plugin with only a future level, an Elixir plugin with three level 3 versions, and a duplicate Zig entry |
| [`marketplace/index-cases.json`](../../contract/v1/fixtures/marketplace/index-cases.json) | 6 `validateIndex` cases: the official source on Apple silicon, a third-party source listing the official namespace, Windows picking its own archive, falling back to an older version by host capabilities, and a capability Core cannot validate never being offered |
| [`marketplace/invalid-index-entries.json`](../../contract/v1/fixtures/marketplace/invalid-index-entries.json) | 24 entry error cases: 23 `plugin.index.invalid` and 1 `plugin.index.officialNamespace` |
| [`marketplace/install-reviews.json`](../../contract/v1/fixtures/marketplace/install-reviews.json) | 19 `installReview` cases: first install, an unchanged update, argument or probe changes needing new approval, a package that does not match its entry, an unsigned official-namespace package, and the capability requirements of build plugins |
| [`marketplace/source-urls.json`](../../contract/v1/fixtures/marketplace/source-urls.json) | 11 `resolveSource` cases: 5 resolved and 6 `plugin.source.invalid` |

### Related guides

- [Building a complete-lifecycle LSP plugin](../../guides/lsp-plugin-lifecycle.md): release, update, and rollback drills, and the community index `lithe-plugin-index.json`
