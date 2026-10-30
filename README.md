# Lithe 插件开发

> English: The entry point for building [Lithe](https://github.com/1lck/Lithe-IDEA) plugins:
> guides, the full plugin contract, the official SDKs, and example plugins. Writing,
> testing, and publishing a plugin happens in your own repository; you never need to
> clone the Lithe repository or send it a pull request.

写插件、测试插件、发布插件，都在**你自己的仓库**里完成。不需要克隆 Lithe 主仓库，也不需要给它提 PR。

| 想做什么 | 看哪里 |
| --- | --- |
| 了解插件长什么样 | [插件是什么](#插件是什么) |
| 动手做第一个插件 | [五步做出第一个插件](#五步做出第一个插件) |
| 跟着教程完整做一遍 | [教程](#教程) |
| 写 `plugin.json` 时有提示和校验 | [在编辑器里校验](#在编辑器里校验-pluginjson) |
| 需要自己的后端进程 | [插件后端 SDK](#插件后端-sdk可选) |
| 参考现成插件 | [示例插件](#示例插件) |
| 查字段的准确含义 | [插件契约](#插件契约) |

## 插件是什么

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
- 插件包安装后是只读的。缓存、状态和日志写到 Lithe 通过启动模板传给你的目录（`${stateDir}`、`${workspaceStateDir}`），不要写进插件包。

## 五步做出第一个插件

1. **复制模板。** 以 [Lua 插件](https://github.com/Lithe-IDEA/lithe-plugin-lua) 为起点，它是 Lithe Store 的示例和模板。
2. **改 `plugin.json`。** 换成你自己的 ID、名称、语言和语言服务器。先按[下一节](#在编辑器里校验-pluginjson)配置编辑器，写错字段时会直接提示。
3. **在构建阶段准备好所有文件。** 语言服务器、后端都在打包时放进插件包，运行时不下载任何东西。详见 [LSP 插件完整生命周期教程](guides/lsp-plugin-lifecycle.md)。
4. **在 Lithe 里试装。** 开发时可以用“从磁盘安装插件…”选择 zip 或目录（同样要确认安装审查；Lithe 安装的是一份快照，改完要重新安装）。发布前在自己仓库发一个 Release，并在仓库根目录放 `lithe-plugin-index.json`，然后在“管理插件仓库…”里加入你的仓库地址，走一遍用户的真实安装流程。
5. **发布。** 想让所有 Lithe 用户在插件市场看到，按 [Lithe Store 发布指南](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md) 收录；收录后新版本会被自动追踪。

需要运行、测试或构建工具集成时，再读 [构建、运行与测试接入指南](guides/build-tool-integration.md)。

### 选 API 等级

`pluginAPIVersion` 决定 Lithe 需要支持哪些功能才能安装你的插件。选能满足需求的最低等级，老版本 Lithe 也能装。

| 等级 | 能做什么 | 示例 |
| --- | --- | --- |
| `1` | 语言、语言服务器（随包附带或使用用户安装的命令）、运行和测试 | Lua、Go、PHP |
| `2` | 增加：用 Node.js / Java 运行时启动、启动模板、插件设置、工具窗口和命令 | — |
| `3` | 增加：按能力声明版本（`requires`）、声明式构建系统、构建后端、VCS 提供者 | Gradle、Maven、Cargo |
| `4` | 增加：调试适配器（`debugAdapters`，DAP 进程）、用选定的运行时启动适配器 | LLDB（预发布） |

三个版本号不要混淆：`schemaVersion` 永远是 `2`，表示清单格式；`pluginAPIVersion` 是上表的等级；`version` 是你自己插件的版本。

API 3 及以上必须在 `requires` 里逐项列出实际用到的能力。例如只用声明式构建系统时写 `"requires": { "buildSystems": 1 }`，不要多写。

API 4 需要 Lithe 明确支持 API 4：旧版 Lithe 在插件市场里会提示升级，而不是安装。目前只有 macOS 宿主实现了调试适配器。

## 在编辑器里校验 plugin.json

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

## 插件后端 SDK（可选）

只用语言服务器、运行和测试的插件不需要后端。需要构建后端（`buildProviders`）、插件函数，或给插件自己的网页配一个后端进程（`backend`）时，用官方 SDK 写：协议、取消、进程清理和错误处理由 SDK 负责，你只写处理函数。

| SDK | 运行时 | Release 标签 | 下载内容 | 怎么放进插件 |
| --- | --- | --- | --- | --- |
| [Node.js](sdk/node/README.md) | Node.js 20.3+ | `sdk-node-v<版本>` | `lithe-plugin-sdk-node-<版本>.tgz` 和 `.sha256` | 解压后把 `package/` 复制进插件包，用相对路径导入 |
| [Java](sdk/java/README.md) | Java 8+ | `sdk-java-v<版本>` | `lithe-plugin-sdk-java-<版本>.jar` 和 `.sha256` | shade 进后端 JAR |
| [Rust](sdk/rust/README.md) | Rust 1.87+ | `sdk-rust-v<版本>` | `lithe-plugin-sdk-rust-<版本>.crate` 和 `.sha256` | 用 Git 依赖锁定标签（`Cargo.lock` 记录 commit），或解压后作为 `path` 依赖；编译成每个平台各一份的插件后端可执行文件 |

SDK 的源码都在本仓库的 `sdk/` 下，发布在[本仓库的 Releases](https://github.com/Lithe-IDEA/lithe-plugin-docs/releases) 里，不在 npm、Maven 或 crates.io 上。Rust SDK 目前只实现插件后端（`backend`）：插件函数和宿主凭据服务；构建后端和 VCS 后端请用 Node.js 或 Java SDK。Java SDK 0.1.0 是搬到这里之前从 Lithe 主仓库发布的，[那个 Release](https://github.com/1lck/Lithe-IDEA/releases/tag/sdk-java-v0.1.0) 继续可用，已经锁定它的插件不用改。

规则只有一条：**在构建时下载，并按你锁定的 SHA-256（或 Git commit）校验；运行时不下载任何东西。** 插件包安装后没有 `npm install` 步骤，也必须能离线运行。

[Maven 插件](https://github.com/Lithe-IDEA/lithe-plugin-maven) 的 `sdk.lock.json` 是锁定 Java SDK 的现成例子。

## 示例插件

| 插件 | API | 示范什么 |
| --- | --- | --- |
| [Lua](https://github.com/Lithe-IDEA/lithe-plugin-lua) | 1 | **模板**；随包附带语言服务器（LuaLS），运行当前文件 |
| [Go](https://github.com/Lithe-IDEA/lithe-plugin-go) | 1 | 使用用户自己安装的 gopls，运行 main 包和 `go test` |
| [PHP](https://github.com/Lithe-IDEA/lithe-plugin-php) | 1 | 使用用户自己安装的 Intelephense（许可证不允许随包分发），PHPUnit |
| [Gradle](https://github.com/Lithe-IDEA/lithe-plugin-gradle) | 3 | 纯声明式构建系统，不需要后端 |
| [Maven](https://github.com/Lithe-IDEA/lithe-plugin-maven) | 3 | 用 Java SDK 写的构建后端 |
| [Cargo](https://github.com/Lithe-IDEA/lithe-plugin-cargo) | 3 | 用 Node.js SDK 写的构建后端和插件设置 |
| [LLDB](https://github.com/Lithe-IDEA/lithe-plugin-lldb) | 4 | 调试适配器的参考实现；用 Xcode 自带的 `lldb-dap` 调试 C、C++、Swift 和 Rust，含 `executable` 查找和调试控制台补全 |

## 教程

| 教程 | 内容 |
| --- | --- |
| [LSP 插件完整生命周期](guides/lsp-plugin-lifecycle.md) | 从 Lua 模板开始：清单、服务器打包、激活、状态目录、停止、升级与卸载 |
| [原生构建工作流（开发中）](guides/native-build-workflows.md) | Node/Java SDK 的构建执行、精确测试、运行准备，以及未提交源码的真实 Core 验证 |
| [构建、运行与测试接入](guides/build-tool-integration.md) | 可用的执行提供者示例、工程后端边界，以及开发中的 API 3 状态 |
| [编写调试适配器插件](guides/debug-adapter-plugin.md) | API 4：从最小清单到启动方式、先编译再调试的 `executable`、调试控制台补全，以及预发布与转正 |
| [插件后端：工作区上下文与宿主服务（开发中）](guides/plugin-backend-workspace.md) | `backend` 2 与协议 3：免页面启动、工作区上下文、宿主通知、失败码，以及协议 3 只增不改时后端要遵守的规则 |

收录和上架的规则由 Lithe Store 维护：[发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)讲怎么打包、提交收录和发布新版本，[版本追踪与更新](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md)讲 Lithe 怎么选择可安装的版本。

## 插件契约

插件契约规定每个字段的含义、校验规则和错误码，完整内容在本仓库的 [`contract/v1/`](contract/v1/README.md)：

| 文件 | 内容 |
| --- | --- |
| [`README.md`](contract/v1/README.md) | 字段、API 等级、能力版本、权限和错误码的完整说明 |
| [`plugin-manifest.schema.json`](contract/v1/plugin-manifest.schema.json) | `plugin.json` 的 JSON Schema |
| [`plugin-index.schema.json`](contract/v1/plugin-index.schema.json) | `lithe-plugin-index.json` 的 JSON Schema |
| [`fixtures/`](contract/v1/fixtures) | 标准用例：合法和非法的清单、命令规划、构建规划、插件函数协议、市场索引，以及 Lithe 对每个用例的预期结果 |

fixture 按演示的能力命名，例如 `build-command-stdout.plugin.json` 演示“解码命令的标准输出”。它们借 Cargo、Go、Gradle、Maven 等真实工具作为输入，但不是这些插件本身；要参考完整插件，看[示例插件](#示例插件)。

契约由 Lithe 的核心代码统一执行，各平台的校验结果相同。它在 Lithe 主仓库的 `shared/contracts/plugins/v1/` 定义，和执行它的代码一起修改；本仓库的 `contract/v1/` 是固定版本的逐字节副本，`contract.lock.json` 记录来源 commit 和每个文件的 SHA-256。副本可能包含还没进入正式版 Lithe 的能力，发布插件前以目标版本实际支持为准。

对契约有疑问或建议，到 [Lithe 主仓库提 issue](https://github.com/1lck/Lithe-IDEA/issues)。

## 维护本仓库

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

### SDK

| 命令 | 作用 |
| --- | --- |
| `node --test sdk/node/test/*.test.mjs` | Node.js SDK 的单元测试 |
| `mvn -B -ntp -f sdk/java/pom.xml verify -Djvm=<java>` | Java SDK 的单元测试；`-Djvm` 指定运行测试的 Java（CI 用 Java 8） |
| `cargo test --locked --manifest-path sdk/rust/Cargo.toml` | Rust SDK 的单元测试（CI 用 Rust 1.87，另用 stable 跑 `cargo fmt` 和 `cargo clippy`） |
| `node scripts/sdk-e2e.mjs` | 端到端测试：各 SDK 的示例后端接入真实的 Lithe Rust Core。Node.js 和 Java 跑 `sdk/e2e` 里同一套构建后端场景，Rust 跑插件后端场景（`plugin_sdk/backend.rs`），测试代替宿主应答凭据请求 |
| `node scripts/sdk-e2e.mjs --ref preview` | 同上，但用主仓库 `preview` 分支的最新 Core |

端到端测试需要 Rust、Node.js 20.3+、Java 和 Maven。Rust Core 在主仓库，脚本会把锁文件锁定的那个主仓库 commit 稀疏检出到 `.artifacts/lithe`（只取 `rust/` 和 `shared/`），把 `sdk/e2e` 复制进它的 `rust/lithe-core/tests/` 再运行，所以 SDK 测的是和契约副本同一个版本的 Core。`LITHE_PLUGIN_SDK_JAVA_RUNTIME` 指定运行 Java 示例的 Java，`LITHE_PLUGIN_SDK_RUST_SAMPLE` 可以指定已编译好的 Rust 示例后端（否则脚本用 Cargo 编译）；已有主仓库本地克隆时可以加 `--source <克隆目录>`。CI 每次提交跑锁定版本，每周一再跑一次主仓库 `preview` 的最新版本，Core 改动影响 SDK 时会提前发现。

发版：先改版本号（Node.js 改 `sdk/node/package.json`；Java 运行 `mvn -f sdk/java/pom.xml versions:set -DnewVersion=<版本> -DgenerateBackupPoms=false`，三个 pom 一起改；Rust 改 `sdk/rust/Cargo.toml` 的 `workspace.package.version` 并更新 `Cargo.lock`），合并后推送对应标签 `sdk-node-v<版本>`、`sdk-java-v<版本>` 或 `sdk-rust-v<版本>`。发版工作流会检查标签和版本一致，测试通过后把包和 `.sha256` 发到本仓库的 Releases。

## 许可证

[Apache-2.0](LICENSE)
