# Lithe 插件开发 / Lithe plugin development

[中文](#中文) · [English](#english)

## 中文

写插件、测试插件、发布插件，都在**你自己的仓库**里完成。不需要克隆 [Lithe](https://github.com/1lck/Lithe-IDEA) 主仓库，也不需要给它提 PR。

| 想做什么 | 看哪里 |
| --- | --- |
| 了解插件长什么样 | [插件是什么](#插件是什么) |
| 动手做第一个插件 | [五步做出第一个插件](#五步做出第一个插件) |
| 跟着教程完整做一遍 | [教程](#教程) |
| 写 `plugin.json` 时有提示和校验 | [在编辑器里校验](#在编辑器里校验-pluginjson) |
| 需要自己的后端进程 | [插件后端 SDK](#插件后端-sdk可选) |
| 接入宿主日志、排查插件故障 | [插件接入统一日志](guides/plugin-logging.md) |
| 参考现成插件 | [示例插件](#示例插件) |
| 查 API-01–44 现在能用什么、每种能力和协议有哪些版本、各宿主支持到哪一版 | [插件参考](reference/README.md) |
| 查字段的准确含义 | [插件契约](#插件契约) |

### 插件是什么

一个 Lithe 插件就是一个目录：`plugin.json` 声明插件提供什么，其余文件是它用到的资源，例如语言服务器、脚本和图标。

```text
io.github.<你的用户名>.zig-support/
├── plugin.json
└── …            # plugin.json 引用的资源
```

Lithe 读取并校验 `plugin.json`，然后自己启动其中声明的外部进程。插件代码不会加载进 Lithe 进程，所以同一个插件包不需要为每个 Lithe 版本重新构建。

社区插件必须遵守这几条：

- `id` 用你自己的反向域名，例如 `io.github.<你的用户名>.zig-support`。`dev.lithe` 和 `dev.lithe.*` 是官方保留命名空间。
- `vendor.signatureRequirement` 写 `"none"`。社区插件不带 Lithe 签名，用户安装前会看到完整的程序、权限和来源，并确认。
- 包内路径只用 `/`，不能有 `..`、绝对路径或符号链接。
- 插件包安装后是只读的。缓存和状态使用 Lithe 通过启动模板传入的 `${stateDir}`、`${workspaceStateDir}`；诊断日志使用 SDK 与宿主统一入口。确需原生文件日志时，先确认目标宿主支持 `${logDir}`，详见[插件接入统一日志](guides/plugin-logging.md)。

### 五步做出第一个插件

1. **复制模板。** 以 [Lua 插件](https://github.com/Lithe-IDEA/lithe-plugin-lua) 为起点，它是 Lithe Store 的示例和模板。
2. **改 `plugin.json`。** 换成你自己的 ID、名称、语言和语言服务器。先按[下一节](#在编辑器里校验-pluginjson)配置编辑器，写错字段时会直接提示。
3. **在构建阶段准备好所有文件。** 语言服务器、后端都在打包时放进插件包，运行时不下载任何东西。详见 [LSP 插件完整生命周期教程](guides/lsp-plugin-lifecycle.md)。
4. **在 Lithe 里试装。** 开发时可以用“从磁盘安装插件…”选择 zip 或目录（同样要确认安装审查；Lithe 安装的是一份快照，改完要重新安装）。发布前在自己仓库发一个 Release，并在仓库根目录放 `lithe-plugin-index.json`，然后在“管理插件仓库…”里加入你的仓库地址，走一遍用户的真实安装流程。
5. **发布。** 想让所有 Lithe 用户在插件市场看到，按 [Lithe Store 发布指南](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md) 收录；收录后新版本会被自动追踪。

需要运行、测试或构建工具集成时，再读 [构建、运行与测试接入指南](guides/build-tool-integration.md)。

#### 选 API 等级

`pluginAPIVersion` 决定 Lithe 需要支持哪些功能才能安装你的插件。选能满足需求的最低等级，老版本 Lithe 也能装。

| 等级 | 能做什么 | 示例 |
| --- | --- | --- |
| `1` | 语言、语言服务器（随包附带或使用用户安装的命令）、运行和测试 | Lua、Go、PHP |
| `2` | 增加：用 Node.js / Java 运行时启动、启动模板、插件设置、工具窗口和命令 | — |
| `3` | 增加：按能力声明版本（`requires`）、声明式构建系统、构建后端、VCS 提供者 | Gradle、Maven、Cargo |
| `4` | 增加：调试适配器（`debugAdapters`，DAP 进程）、用选定的运行时启动适配器 | LLDB（预发布） |

三个版本号不要混淆：`schemaVersion` 永远是 `2`，表示清单格式；`pluginAPIVersion` 是上表的等级；`version` 是你自己插件的版本。

API 3 及以上必须在 `requires` 里逐项列出实际用到的能力。例如只用声明式构建系统时写 `"requires": { "buildSystems": 1 }`，不要多写。每个能力有哪些版本、各宿主支持到哪一版，见[能力与协议版本对照](reference/capabilities-and-versions.md)。

API 4 需要 Lithe 明确支持 API 4：旧版 Lithe 在插件市场里会提示升级，而不是安装。目前只有 macOS 宿主实现了调试适配器。

### 在编辑器里校验 plugin.json

契约里的两份 JSON Schema 地址固定：

| 文件 | Schema |
| --- | --- |
| `plugin.json` | `https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-manifest.schema.json` |
| `lithe-plugin-index.json` | `https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-index.schema.json` |

> **不要在 `plugin.json` 里写 `"$schema"`。** Lithe 会拒绝清单中的未知字段，加了以后插件反而装不上。请在编辑器里按文件名关联 Schema。

VS Code：在插件仓库的 `.vscode/settings.json` 中加入

```json
{
  "json.schemas": [
    {
      "fileMatch": ["**/plugin.json"],
      "url": "https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-manifest.schema.json"
    },
    {
      "fileMatch": ["**/lithe-plugin-index.json"],
      "url": "https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-index.schema.json"
    }
  ]
}
```

JetBrains IDE：设置 → Languages & Frameworks → Schemas and DTDs → JSON Schema Mappings，添加上面的 URL，文件模式填 `plugin.json`。

Schema 只检查 JSON 结构。ID 引用、权限、激活事件、模板变量这类规则由 Lithe 在安装时检查，规则写在[插件契约](#插件契约)里。

### 插件后端 SDK（可选）

只用语言服务器、运行和测试的插件不需要后端。需要构建后端（`buildProviders`）、插件函数，或给插件自己的网页配一个后端进程（`backend`）时，用官方 SDK 写：协议、取消、进程清理和错误处理由 SDK 负责，你只写处理函数。

| SDK | 运行时 | Release 标签 | 下载内容 | 怎么放进插件 |
| --- | --- | --- | --- | --- |
| [Node.js](sdk/node/README.md) | Node.js 20.3+ | `sdk-node-v<版本>` | `lithe-plugin-sdk-node-<版本>.tgz` 和 `.sha256` | 解压后把 `package/` 复制进插件包，用相对路径导入 |
| [Java](sdk/java/README.md) | Java 8+ | `sdk-java-v<版本>` | `lithe-plugin-sdk-java-<版本>.jar` 和 `.sha256` | shade 进后端 JAR |
| [Rust](sdk/rust/README.md) | Rust 1.87+ | `sdk-rust-v<版本>` | `lithe-plugin-sdk-rust-<版本>.crate` 和 `.sha256` | 用 Git 依赖锁定标签（`Cargo.lock` 记录 commit），或解压后作为 `path` 依赖；编译成每个平台各一份的插件后端可执行文件 |

SDK 的源码都在本仓库的 `sdk/` 下，发布在[本仓库的 Releases](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases) 里，不在 npm、Maven 或 crates.io 上。Rust SDK 目前只实现插件后端（`backend`）：插件函数和宿主服务（插件函数协议 1–3）；构建后端请用 Node.js 或 Java SDK，VCS 后端请用 Node.js SDK。各 SDK 实现的协议版本见[宿主与官方 SDK 对照](reference/capabilities-and-versions.md#宿主与官方-sdk-对照)。Java SDK 0.1.0 是搬到这里之前从 Lithe 主仓库发布的，[那个 Release](https://github.com/1lck/Lithe-IDEA/releases/tag/sdk-java-v0.1.0) 继续可用，已经锁定它的插件不用改。

规则只有一条：**在构建时下载，并按你锁定的 SHA-256（或 Git commit）校验；运行时不下载任何东西。** 插件包安装后没有 `npm install` 步骤，也必须能离线运行。

[Maven 插件](https://github.com/Lithe-IDEA/lithe-plugin-maven) 的 `sdk.lock.json` 是锁定 Java SDK 的现成例子。

### 示例插件

| 插件 | API | 示范什么 |
| --- | --- | --- |
| [Lua](https://github.com/Lithe-IDEA/lithe-plugin-lua) | 1 | **模板**；随包附带语言服务器（LuaLS），运行当前文件 |
| [Go](https://github.com/Lithe-IDEA/lithe-plugin-go) | 1 | 使用用户自己安装的 gopls，运行 main 包和 `go test` |
| [PHP](https://github.com/Lithe-IDEA/lithe-plugin-php) | 1 | 使用用户自己安装的 Intelephense（许可证不允许随包分发），PHPUnit |
| [Gradle](https://github.com/Lithe-IDEA/lithe-plugin-gradle) | 3 | 纯声明式构建系统，不需要后端 |
| [Maven](https://github.com/Lithe-IDEA/lithe-plugin-maven) | 3 | 用 Java SDK 写的构建后端 |
| [Cargo](https://github.com/Lithe-IDEA/lithe-plugin-cargo) | 3 | 用 Node.js SDK 写的构建后端和插件设置 |
| [LLDB](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | 4 | 调试适配器的参考实现；用 Xcode 自带的 `lldb-dap` 调试 C、C++、Swift 和 Rust，含 `executable` 查找和调试控制台补全 |

### 教程

| 教程 | 内容 |
| --- | --- |
| [LSP 插件完整生命周期](guides/lsp-plugin-lifecycle.md) | 从 Lua 模板开始：清单、服务器打包、激活、状态目录、停止、升级与卸载 |
| [插件接入统一日志](guides/plugin-logging.md) | 三 SDK 的安全失败边界、受管诊断通道、macOS v2 / Windows v1 状态，以及 `${logDir}` 的本地会话生命周期 |
| [原生构建工作流](guides/native-build-workflows.md) | Node/Java SDK 的构建执行、精确测试、运行准备，以及未提交源码的真实 Core 验证 |
| [构建、运行与测试接入](guides/build-tool-integration.md) | 执行提供者示例、工程后端边界，以及 API 3 构建系统的现状和已发布示例 |
| [编写调试适配器插件](guides/debug-adapter-plugin.md) | API 4：从最小清单到启动方式、先编译再调试的 `executable`、调试控制台补全，以及预发布与转正 |
| [插件后端：工作区上下文与宿主服务](guides/plugin-backend-workspace.md) | `backend` 2 与协议 3：免页面启动、工作区上下文、纯后端命令（`ui` 4）、文档/文件/窗口宿主方法、宿主通知、失败码，以及协议 3 只增不改时后端要遵守的规则 |
| [插件自有的原生终端](guides/native-terminals.md) | 插件函数协议 3 的 `terminal/*`：创建、显示、关闭、重开和退出通知（macOS、本机工作区） |

收录和上架的规则由 Lithe Store 维护：[发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)讲怎么打包、提交收录和发布新版本，[版本追踪与更新](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md)讲 Lithe 怎么选择可安装的版本。

### 插件参考

[`reference/`](reference/README.md) 按功能整理契约，中英双语：

| 页面 | 内容 |
| --- | --- |
| [插件 API 总表](reference/api-catalog.md) | API-01–44 每一项现在能用的方法、尚未提供的部分和对应功能页 |
| [能力与协议版本对照](reference/capabilities-and-versions.md) | 插件 API 级别、`requires` 能力版本、进程协议版本、文档格式版本，按 Core、macOS、Windows 和官方 SDK 对照 |
| [插件协议更新记录](reference/protocol-changelog.md) | 每个协议、能力和 API 是哪天、哪个 PR 改的，以及每个版本首次出现在哪个 PR；只有中文 |
| [功能页](reference/README.md#功能页) | 15 个功能各一页：版本对照、字段、宿主现状、契约章节、示例与用例 |

### 插件契约

插件契约规定每个字段的含义、校验规则和错误码，完整内容在本仓库的 [`contract/v1/`](contract/v1/README.md)，以英文为准：

| 文件 | 内容 |
| --- | --- |
| [`README.md`](contract/v1/README.md) | 字段、API 等级、能力版本、权限和错误码的完整说明 |
| [`plugin-manifest.schema.json`](contract/v1/plugin-manifest.schema.json) | `plugin.json` 的 JSON Schema |
| [`plugin-index.schema.json`](contract/v1/plugin-index.schema.json) | `lithe-plugin-index.json` 的 JSON Schema |
| [`native-ui.schema.json`](contract/v1/native-ui.schema.json) | `ui` 3 宿主渲染的原生内容的 JSON Schema |
| [`toolchains/`](contract/v1/toolchains/README.md) | 插件提供的工具链：工具定义、版本管理器提供方、提供方后端协商（`litheToolchains`）、查询与回复校验（含稳定失败码）、固定值（pin）和四层解析。Core 已实现；macOS 设置页以只读方式显示结果，macOS 构建和提供者运行另有独立的解析适配器，已有的 Java 和语言服务器仍用原来的方式；Windows 不支持 |
| [`fixtures/`](contract/v1/fixtures) | 标准用例，按功能分目录（`manifest`、`languages`、`backend`、`build-systems`、`build-providers`、`marketplace` 等），和 [reference 功能页](reference/README.md#功能页)一一对应：合法和非法的清单、命令和构建规划、插件函数协议、市场索引，以及 Lithe 对每个用例的预期结果 |

fixture 放在 `fixtures/<功能>/` 下，文件按演示的能力命名，例如 `build-systems/build-command-stdout.plugin.json` 演示“解码命令的标准输出”。它们借 Cargo、Go、Gradle、Maven 等真实工具作为输入，但不是这些插件本身；要参考完整插件，看[示例插件](#示例插件)。每个目录的文件和用例数量见对应功能页的“示例与用例”。

契约由 Lithe 的核心代码统一执行，各平台的校验结果相同。它在 Lithe 主仓库的 `shared/contracts/plugins/v1/` 定义，和执行它的代码一起修改；本仓库的 `contract/v1/` 是固定版本的逐字节副本，`contract.lock.json` 记录来源 commit 和每个文件的 SHA-256。本仓库的指南和[版本对照](reference/capabilities-and-versions.md)以主仓库最新 `preview` 为准；副本和指南可能包含还没进入 Lithe 稳定版的能力，发布插件前以目标版本实际支持为准。

对契约有疑问或建议，到 [Lithe 主仓库提 issue](https://github.com/1lck/Lithe-IDEA/issues)。

### 维护本仓库

不要手改 `contract/` 下的文件。契约在主仓库合并后，用同步脚本把整个目录同步过来，新增、删除、修改的文件都会跟上，锁文件同时更新：

```bash
node scripts/contract.mjs sync --revision <主仓库的完整 commit>
```

已有主仓库的本地克隆时，可以加 `--checkout <克隆目录>` 从本地读取。其他命令：

| 命令 | 作用 |
| --- | --- |
| `node scripts/contract.mjs verify` | 离线检查：锁文件格式、SHA-256、每份 Schema 的 `$id` 是否等于它在本仓库的地址、契约 Markdown 的相对链接是否都留在副本内、`contract/` 下有没有未锁定的文件 |
| `node scripts/contract.mjs verify --upstream` | 联网检查：副本与主仓库锁定 commit 的契约目录完全一致，多出、缺少或内容不同的文件都会列出来 |
| `node scripts/contract.mjs drift` | 联网检查：主仓库 `preview` 分支（改动合入的分支；默认分支 `main` 只在发版时更新）上的契约是否已经变化 |
| `node --test scripts/*.test.mjs` | 同步脚本自身的测试 |

联网命令会调用 GitHub API；设置 `GITHUB_TOKEN` 可以避开匿名请求的频率限制。CI 在每次提交时运行前两项检查和测试，每周一运行一次 `drift`。主仓库的契约有变化时，每周检查会失败并提示重新同步。

契约变化后，同时检查 [`reference/`](reference/README.md) 里受影响的功能页和版本表：它们是手写的对照，不会随同步自动更新。每次同步还要在[插件协议更新记录](reference/protocol-changelog.md)里补上新合入的 PR，做法见该页的“维护”一节。

#### SDK

| 命令 | 作用 |
| --- | --- |
| `node --test sdk/node/test/*.test.mjs` | Node.js SDK 的单元测试 |
| `mvn -B -ntp -f sdk/java/pom.xml verify -Djvm=<java>` | Java SDK 的单元测试；`-Djvm` 指定运行测试的 Java（CI 用 Java 8） |
| `cargo test --locked --manifest-path sdk/rust/Cargo.toml` | Rust SDK 的单元测试（CI 用 Rust 1.87，另用 stable 跑 `cargo fmt` 和 `cargo clippy`） |
| `node scripts/sdk-e2e.mjs` | 端到端测试：各 SDK 的示例后端接入真实的 Lithe Rust Core。Node.js 和 Java 跑 `sdk/e2e` 里同一套构建后端场景，Rust 跑插件后端场景（`plugin_sdk/backend.rs`），测试代替宿主应答凭据请求 |
| `node scripts/sdk-e2e.mjs --ref preview` | 同上，但用主仓库 `preview` 分支的最新 Core |

端到端测试需要 Rust、Node.js 20.3+、Java 和 Maven。Rust Core 在主仓库，脚本会把锁文件锁定的那个主仓库 commit 稀疏检出到 `.artifacts/lithe`（只取 `rust/` 和 `shared/`），把 `sdk/e2e` 复制进它的 `rust/lithe-core/tests/` 再运行，所以 SDK 测的是和契约副本同一个版本的 Core。`LITHE_PLUGIN_SDK_JAVA_RUNTIME` 指定运行 Java 示例的 Java，`LITHE_PLUGIN_SDK_RUST_SAMPLE` 可以指定已编译好的 Rust 示例后端（否则脚本用 Cargo 编译）；已有主仓库本地克隆时可以加 `--source <克隆目录>`。CI 每次提交跑锁定版本，每周一再跑一次主仓库 `preview` 的最新版本，Core 改动影响 SDK 时会提前发现。

发版：先改版本号（Node.js 改 `sdk/node/package.json`；Java 运行 `mvn -f sdk/java/pom.xml versions:set -DnewVersion=<版本> -DgenerateBackupPoms=false`，三个 pom 一起改；Rust 改 `sdk/rust/Cargo.toml` 的 `workspace.package.version` 并更新 `Cargo.lock`），合并后推送对应标签 `sdk-node-v<版本>`、`sdk-java-v<版本>` 或 `sdk-rust-v<版本>`。发版工作流会检查标签和版本一致，测试通过后把包和 `.sha256` 发到本仓库的 Releases。发版后同步更新[SDK 发版历史](reference/capabilities-and-versions.md#sdk-发版历史)。

### 许可证

[Apache-2.0](LICENSE)

---

## English

Writing, testing, and publishing a plugin all happen in **your own repository**. You never need to clone the [Lithe](https://github.com/1lck/Lithe-IDEA) repository or send it a pull request.

| To | Go to |
| --- | --- |
| See what a plugin looks like | [What a plugin is](#what-a-plugin-is) |
| Build your first plugin | [Your first plugin in five steps](#your-first-plugin-in-five-steps) |
| Follow a complete walkthrough | [Guides](#guides) |
| Get completion and validation while writing `plugin.json` | [Validate in your editor](#validate-pluginjson-in-your-editor) |
| Run a backend process of your own | [Plugin backend SDKs](#plugin-backend-sdks-optional) |
| Connect to host logging and diagnose plugin failures | [Plugin logging](guides/plugin-logging.md) |
| Learn from existing plugins | [Example plugins](#example-plugins) |
| See what API-01–44 offer today, every version of each capability and protocol, and which version each host supports | [Plugin reference](reference/README.md) |
| Look up the exact meaning of a field | [Plugin contract](#plugin-contract) |

### What a plugin is

A Lithe plugin is a directory: `plugin.json` declares what the plugin provides, and the other files are the resources it uses, such as language servers, scripts, and icons.

```text
io.github.<your-user-name>.zig-support/
├── plugin.json
└── …            # resources referenced by plugin.json
```

Lithe reads and validates `plugin.json` and then starts the external processes it declares itself. Plugin code is never loaded into the Lithe process, so the same package does not need rebuilding for each Lithe version.

Community plugins must follow these rules:

- `id` uses your own reverse domain name, such as `io.github.<your-user-name>.zig-support`. `dev.lithe` and `dev.lithe.*` are reserved for official plugins.
- `vendor.signatureRequirement` is `"none"`. Community plugins carry no Lithe signature; before installing, users see and confirm the complete programs, permissions, and source.
- Package paths use only `/`, with no `..`, absolute paths, or symbolic links.
- An installed package is read-only. Caches and state go to `${stateDir}` and `${workspaceStateDir}`, which Lithe passes through launch templates; diagnostic logs go through the shared SDK and host path. If you really need native file logs, first check that the target host supports `${logDir}`; see [Plugin logging](guides/plugin-logging.md).

### Your first plugin in five steps

1. **Copy the template.** Start from the [Lua plugin](https://github.com/Lithe-IDEA/lithe-plugin-lua), Lithe Store's example and template.
2. **Edit `plugin.json`.** Use your own ID, name, language, and language server. Set up your editor as in the [next section](#validate-pluginjson-in-your-editor) first, so mistyped fields are flagged as you write.
3. **Prepare every file at build time.** Language servers and backends go into the package when you build it; nothing is downloaded at run time. See the [LSP plugin lifecycle guide](guides/lsp-plugin-lifecycle.md).
4. **Try it in Lithe.** During development, use "Install Plugin from Disk…" with a zip or directory (you still confirm the install review; Lithe installs a snapshot, so reinstall after each change). Before publishing, create a Release in your repository, put `lithe-plugin-index.json` at the repository root, and add your repository under "Manage Plugin Repositories…" to walk through the real user install flow.
5. **Publish.** To list the plugin in every Lithe user's marketplace, follow the [Lithe Store publishing guide](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md); new versions are tracked automatically once listed.

When you need run, test, or build tool integration, read the [build, run, and test integration guide](guides/build-tool-integration.md).

#### Choosing an API level

`pluginAPIVersion` decides which features Lithe must support to install your plugin. Choose the lowest level that meets your needs so older Lithe versions can install it too.

| Level | Adds | Examples |
| --- | --- | --- |
| `1` | Languages, language servers (packaged or a command the user installed), run and test | Lua, Go, PHP |
| `2` | Adds: launching with a Node.js or Java runtime, launch templates, plugin settings, tool windows and commands | — |
| `3` | Adds: per-capability versions (`requires`), declarative build systems, build backends, VCS providers | Gradle, Maven, Cargo |
| `4` | Adds: debug adapters (`debugAdapters`, DAP processes), launching an adapter with a selected runtime | LLDB (prerelease) |

Keep the three version numbers apart: `schemaVersion` is always `2` and names the manifest format; `pluginAPIVersion` is the level above; `version` is your plugin's own version.

At API 3 and above, `requires` must list exactly the capabilities you use. For example, a plugin that only uses declarative build systems writes `"requires": { "buildSystems": 1 }` and nothing more. For every version of each capability and which version each host supports, see [Capability and protocol versions](reference/capabilities-and-versions.md).

API 4 needs a Lithe that explicitly supports API 4: older versions show an update prompt in the marketplace instead of installing. Only the macOS host implements debug adapters today.

### Validate plugin.json in your editor

The contract's two JSON Schemas have fixed addresses:

| File | Schema |
| --- | --- |
| `plugin.json` | `https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-manifest.schema.json` |
| `lithe-plugin-index.json` | `https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-index.schema.json` |

> **Do not put `"$schema"` in `plugin.json`.** Lithe refuses unknown manifest fields, so adding it makes the plugin uninstallable. Associate the schema by file name in your editor instead.

VS Code: add this to `.vscode/settings.json` in your plugin repository:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["**/plugin.json"],
      "url": "https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-manifest.schema.json"
    },
    {
      "fileMatch": ["**/lithe-plugin-index.json"],
      "url": "https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/contract/v1/plugin-index.schema.json"
    }
  ]
}
```

JetBrains IDEs: Settings → Languages & Frameworks → Schemas and DTDs → JSON Schema Mappings; add the URL above with the file pattern `plugin.json`.

A schema checks only JSON structure. Rules such as ID references, permissions, activation events, and template variables are checked by Lithe at install time and are written in the [plugin contract](#plugin-contract).

### Plugin backend SDKs (optional)

A plugin that uses only language servers, run, and test needs no backend. When you need a build backend (`buildProviders`), plugin functions, or a backend process for the plugin's own web page (`backend`), write it with an official SDK: the SDK handles the protocol, cancellation, process cleanup, and errors, and you write only the handlers.

| SDK | Runtime | Release tag | Download | How it goes into the plugin |
| --- | --- | --- | --- | --- |
| [Node.js](sdk/node/README.md) | Node.js 20.3+ | `sdk-node-v<version>` | `lithe-plugin-sdk-node-<version>.tgz` and `.sha256` | Unpack it, copy `package/` into the plugin package, and import it by relative path |
| [Java](sdk/java/README.md) | Java 8+ | `sdk-java-v<version>` | `lithe-plugin-sdk-java-<version>.jar` and `.sha256` | Shade it into the backend JAR |
| [Rust](sdk/rust/README.md) | Rust 1.87+ | `sdk-rust-v<version>` | `lithe-plugin-sdk-rust-<version>.crate` and `.sha256` | Pin the tag as a Git dependency (`Cargo.lock` records the commit), or unpack it as a `path` dependency; compile one backend executable per platform |

All SDK sources live under `sdk/` in this repository and are published in [this repository's Releases](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases), not on npm, Maven, or crates.io. The Rust SDK implements only the plugin backend (`backend`): plugin functions and host services (plugin function protocol 1–3); write build backends with the Node.js or Java SDK and VCS backends with the Node.js SDK. For the protocol versions each SDK implements, see [Host and official SDKs](reference/capabilities-and-versions.md#host-and-official-sdks). Java SDK 0.1.0 was released from the Lithe repository before the move; [that Release](https://github.com/1lck/Lithe-IDEA/releases/tag/sdk-java-v0.1.0) stays available, and plugins pinned to it need no change.

There is one rule: **download at build time and verify against the SHA-256 (or Git commit) you pinned; download nothing at run time.** An installed package has no `npm install` step and must run offline.

The [Maven plugin](https://github.com/Lithe-IDEA/lithe-plugin-maven)'s `sdk.lock.json` is a ready example of pinning the Java SDK.

### Example plugins

| Plugin | API | Demonstrates |
| --- | --- | --- |
| [Lua](https://github.com/Lithe-IDEA/lithe-plugin-lua) | 1 | **Template**; a packaged language server (LuaLS) and running the current file |
| [Go](https://github.com/Lithe-IDEA/lithe-plugin-go) | 1 | The user's own gopls installation, running the main package and `go test` |
| [PHP](https://github.com/Lithe-IDEA/lithe-plugin-php) | 1 | The user's own Intelephense installation (its license forbids bundling), PHPUnit |
| [Gradle](https://github.com/Lithe-IDEA/lithe-plugin-gradle) | 3 | A purely declarative build system with no backend |
| [Maven](https://github.com/Lithe-IDEA/lithe-plugin-maven) | 3 | A build backend written with the Java SDK |
| [Cargo](https://github.com/Lithe-IDEA/lithe-plugin-cargo) | 3 | A build backend and plugin settings written with the Node.js SDK |
| [LLDB](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | 4 | Reference debug adapter; debugs C, C++, Swift, and Rust with Xcode's `lldb-dap`, including `executable` lookup and debug console completion |

### Guides

| Guide | Covers |
| --- | --- |
| [LSP plugin lifecycle](guides/lsp-plugin-lifecycle.md) | Starting from the Lua template: manifest, server packaging, activation, state directories, shutdown, upgrade, and uninstall |
| [Plugin logging](guides/plugin-logging.md) | The safe failure boundary of the three SDKs, the managed diagnostics channel, macOS v2 and Windows v1 status, and the local session lifecycle of `${logDir}` |
| [Native build workflows](guides/native-build-workflows.md) | Build execution, exact tests, and run preparation in the Node and Java SDKs, and verifying uncommitted sources against a real Core |
| [Build, run, and test integration](guides/build-tool-integration.md) | Execution provider examples, the project backend boundary, and the current status and published examples of API 3 build systems |
| [Writing a debug adapter plugin](guides/debug-adapter-plugin.md) | API 4: from a minimal manifest to launch modes, compile-then-debug `executable`, debug console completion, and prerelease to stable |
| [Plugin backend: workspace contexts and host services](guides/plugin-backend-workspace.md) | `backend` 2 and protocol 3: launching without a page, workspace contexts, backend-only commands (`ui` 4), document, file, and window host methods, host notifications, failure codes, and the rules a backend follows while protocol 3 only adds |
| [Plugin-owned native terminals](guides/native-terminals.md) | `terminal/*` in plugin function protocol 3: create, show, close, reopen, and exit notifications (macOS, local workspaces) |

Lithe Store maintains the listing rules: [Publishing community plugins](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md) covers packaging, submitting a listing, and releasing new versions, and [Version tracking and updates](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md) covers how Lithe chooses an installable version.

### Plugin reference

[`reference/`](reference/README.md) organizes the contract by feature, in Chinese and English:

| Page | Covers |
| --- | --- |
| [Plugin API catalog](reference/api-catalog.md) | For each of API-01–44, the methods usable today, what is still missing, and its feature page |
| [Capability and protocol versions](reference/capabilities-and-versions.md) | Plugin API levels, `requires` capability versions, process protocol versions, and document format versions, compared across Core, macOS, Windows, and the official SDKs |
| [Plugin protocol changelog](reference/protocol-changelog.md) | When, and in which PR, each protocol, capability, and API changed, and the PR that introduced each version; Chinese only |
| [Feature pages](reference/README.md#feature-pages) | One page for each of 15 features: versions compared, fields, host status, contract sections, examples and cases |

### Plugin contract

The plugin contract defines the meaning, validation rules, and error codes of every field. It lives in [`contract/v1/`](contract/v1/README.md) of this repository, and its English text is normative:

| File | Contents |
| --- | --- |
| [`README.md`](contract/v1/README.md) | Complete description of fields, API levels, capability versions, permissions, and error codes |
| [`plugin-manifest.schema.json`](contract/v1/plugin-manifest.schema.json) | JSON Schema for `plugin.json` |
| [`plugin-index.schema.json`](contract/v1/plugin-index.schema.json) | JSON Schema for `lithe-plugin-index.json` |
| [`native-ui.schema.json`](contract/v1/native-ui.schema.json) | JSON Schema for host-rendered native content (`ui` 3) |
| [`toolchains/`](contract/v1/toolchains/README.md) | Plugin-owned toolchains: tool definitions, version manager providers, provider backend negotiation (`litheToolchains`), query and reply validation (with stable failure codes), pins, and four-layer resolution. Core implements it; the macOS settings pages show results read-only, macOS Build and Provider Run have a separate resolution adapter, and existing Java and language-server consumers keep their previous behavior; Windows does not support it |
| [`fixtures/`](contract/v1/fixtures) | Conformance cases in one directory per feature (`manifest`, `languages`, `backend`, `build-systems`, `build-providers`, `marketplace`, and so on), matching the [reference feature pages](reference/README.md#feature-pages): valid and invalid manifests, command and build plans, plugin function protocol, marketplace indexes, and Lithe's expected result for each case |

Fixtures live under `fixtures/<feature>/` and are named for the capability they demonstrate; for example `build-systems/build-command-stdout.plugin.json` demonstrates decoding a command's standard output. They use real tools such as Cargo, Go, Gradle, and Maven as inputs but are not those plugins; for complete plugins, see the [example plugins](#example-plugins). Each feature page's "Examples and cases" section lists its files and case counts.

Lithe's core code enforces the contract, so every platform validates the same way. It is defined in `shared/contracts/plugins/v1/` of the Lithe repository and changes together with the code that enforces it; `contract/v1/` here is a byte-for-byte copy of a pinned revision, and `contract.lock.json` records the source commit and each file's SHA-256. The guides and [version tables](reference/capabilities-and-versions.md) here track the latest `preview` of the Lithe repository; the copy and the guides may contain capabilities not yet in a stable Lithe release, so check what your target version supports before publishing.

For questions or suggestions about the contract, [open an issue in the Lithe repository](https://github.com/1lck/Lithe-IDEA/issues).

### Maintaining this repository

Do not edit files under `contract/` by hand. After the contract merges in the Lithe repository, sync the whole directory with the sync script; added, removed, and changed files all follow, and the lock file updates with them:

```bash
node scripts/contract.mjs sync --revision <full Lithe repository commit>
```

With a local clone of the Lithe repository, add `--checkout <clone directory>` to read from it. Other commands:

| Command | Purpose |
| --- | --- |
| `node scripts/contract.mjs verify` | Offline check: lock file format, SHA-256s, that each schema's `$id` equals its address in this repository, that relative links in the contract Markdown stay inside the copy, and that `contract/` has no unlocked files |
| `node scripts/contract.mjs verify --upstream` | Online check: the copy matches the contract directory at the pinned Lithe commit exactly, listing extra, missing, or differing files |
| `node scripts/contract.mjs drift` | Online check: whether the contract on the Lithe repository's `preview` branch (where changes merge; the default branch `main` updates only at release) has changed |
| `node --test scripts/*.test.mjs` | Tests for the sync script itself |

Online commands call the GitHub API; set `GITHUB_TOKEN` to avoid anonymous rate limits. CI runs the first two checks and the tests on every commit and runs `drift` every Monday. When the Lithe repository's contract changes, the weekly check fails and asks for a resync.

After a contract change, also check the affected feature pages and version tables in [`reference/`](reference/README.md): they are hand-written comparisons and do not update with the sync. Each sync also adds the newly merged PRs to the [plugin protocol changelog](reference/protocol-changelog.md), as its maintenance section describes.

#### SDKs

| Command | Purpose |
| --- | --- |
| `node --test sdk/node/test/*.test.mjs` | Node.js SDK unit tests |
| `mvn -B -ntp -f sdk/java/pom.xml verify -Djvm=<java>` | Java SDK unit tests; `-Djvm` selects the Java that runs them (CI uses Java 8) |
| `cargo test --locked --manifest-path sdk/rust/Cargo.toml` | Rust SDK unit tests (CI uses Rust 1.87, plus stable for `cargo fmt` and `cargo clippy`) |
| `node scripts/sdk-e2e.mjs` | End-to-end tests: each SDK's sample backend against the real Lithe Rust Core. Node.js and Java run the same build backend scenarios in `sdk/e2e`, Rust runs the plugin backend scenario (`plugin_sdk/backend.rs`), and the test answers credential requests in place of the host |
| `node scripts/sdk-e2e.mjs --ref preview` | The same, against the latest Core on the Lithe repository's `preview` branch |

End-to-end tests need Rust, Node.js 20.3+, Java, and Maven. Rust Core lives in the Lithe repository: the script sparse-checks out the commit pinned in the lock file to `.artifacts/lithe` (only `rust/` and `shared/`), copies `sdk/e2e` into its `rust/lithe-core/tests/`, and runs it there, so the SDKs are tested against the same Core version as the contract copy. `LITHE_PLUGIN_SDK_JAVA_RUNTIME` selects the Java that runs the Java sample, and `LITHE_PLUGIN_SDK_RUST_SAMPLE` can point at a prebuilt Rust sample backend (otherwise the script builds it with Cargo); with a local Lithe clone, add `--source <clone directory>`. CI runs the pinned version on every commit and the latest `preview` every Monday, so Core changes that affect the SDKs show up early.

Releasing: first bump the version (Node.js in `sdk/node/package.json`; for Java run `mvn -f sdk/java/pom.xml versions:set -DnewVersion=<version> -DgenerateBackupPoms=false`, which updates all three poms; for Rust change `workspace.package.version` in `sdk/rust/Cargo.toml` and update `Cargo.lock`), then after merging push the tag `sdk-node-v<version>`, `sdk-java-v<version>`, or `sdk-rust-v<version>`. The release workflow checks that the tag matches the version and, once tests pass, publishes the package and its `.sha256` to this repository's Releases. After a release, update the [SDK release history](reference/capabilities-and-versions.md#sdk-release-history).

### License

[Apache-2.0](LICENSE)
