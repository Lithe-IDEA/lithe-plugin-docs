# 从零构建一个完整生命周期的 LSP 插件 / Building an LSP plugin with a complete lifecycle

[中文](#中文) · [English](#english)

## 中文

这篇教程面向想给 Lithe 增加语言支持的插件作者。我们用可运行的
[Lua Support 模板](https://github.com/Lithe-IDEA/lithe-plugin-lua)走完选型、清单、语言服务器打包、
安装与激活、日常运行、停止、升级和卸载；构建、运行与测试的接入另见
[构建工具接入指南](build-tool-integration.md)。发布步骤见[发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)。

本文以主仓库最新 `preview` 的 **macOS 宿主**为基线：社区插件可以从插件市场源安装，也可以从磁盘安装。
Windows 宿主目前只能从磁盘安装声明式插件包，没有插件市场浏览器，不支持运行提供者（`executionProviders`），
也不向启动计划提供 `logDirectory`，所以用到 `${logDir}` 的启动在 Windows 上会被拒绝。
索引能描述 Windows/Linux 的包，不代表对应宿主已经实现相同的安装与运行流程。
Lua 模板目前只提供 macOS arm64、x64 两个包。各能力版本见[版本对照](../reference/capabilities-and-versions.md)。

### 1. 先确定插件与宿主各自负责什么

LSP（Language Server Protocol，语言服务器协议）让编辑器通过独立进程获得补全、诊断、
跳转等能力。Lithe 的声明式插件用 JSON 描述要启动的程序；宿主读取清单、发现运行时并管理
进程及 LSP 会话。你不需要编写宿主内的 Swift/JavaScript 插件入口。命令行要到启动时才能确定的服务器，
可以改由插件后端在每次启动时解析，见第 3 节的“由插件后端驱动的语言服务器”。

```text
开发与构建：固定上游版本 → 下载并校验 → 组装各平台插件包 → 发布 zip → 索引收录
用户安装：下载 zip（或从磁盘选择包）→ 校验摘要与清单 → 确认程序及权限 → 安装并记录完整性摘要
正常运行：启用 → 打开对应语言的文件 → 启动服务器 → 初始化 LSP → 编辑与诊断
结束使用：禁用/关闭工作区 → 宿主停止会话和进程；更新/卸载 → 替换或移除包
```

| 对象 | 插件作者负责 | Lithe 负责 |
| --- | --- | --- |
| 语言识别 | 文件扩展名、完整文件名、项目标记 | 匹配文件并选择提供者 |
| 语言服务器 | 上游选型、许可、固定版本、启动入口、正确参数 | 校验声明、按需启动、LSP 通信与停止 |
| 运行时 | 声明依赖，或在允许的 API 中随包提供 | 按声明发现、检查并启动运行时 |
| 运行与测试 | 声明命令、参数和测试范围 | 生成计划、执行与展示结果 |
| 可写数据 | 指定上游使用的日志、索引和状态路径 | 为 API 2 启动模板提供平台可写目录 |
| 发布与维护 | 各平台产物、版本升级、兼容与回退验证 | 下载校验、安装审查、插件启停 |

先选成熟且许可允许分发的语言服务器。项目导入、依赖解析、符号和模块模型应尽量由该语言的
成熟后端提供。不要为了“接上 LSP”再复制一套工程模型。LSP 支持的具体功能仍取决于服务器
能力与宿主支持；安装服务器不会自动带来构建、测试、DAP 调试或自定义界面。

### 2. 跑通原版模板，再改成自己的插件

本地需要 Git、Node.js 22+、`tar`、`zip` 和 `unzip`。打包工具使用 Node.js，
不代表目标语言服务器需要 Node.js；LuaLS 是平台相关的原生程序。

```bash
git clone https://github.com/Lithe-IDEA/lithe-plugin-lua.git
cd lithe-plugin-lua
node scripts/package.mjs
unzip -l dist/lua-support-0.3.1-darwin-arm64.zip
shasum -a 256 dist/lua-support-0.3.1-darwin-arm64.zip
```

这里的 `0.3.1` 是本文核对时的模板版本；上游更新后，产物名以模板清单和脚本输出为准。
脚本会重新生成 `dist/`，下载两个 macOS 架构的 LuaLS、核对固定 SHA-256，并输出每个 zip
的 SHA-256。Linux CI 可以组装 macOS 包，但这只证明打包成功，不能替代在目标 Mac 上运行。

模板源码与产物的关系：

```text
源码仓库
├── io.github.lithe-idea.lua-support/
│   ├── plugin.json
│   └── bin/lithe-lua-language-server    # 保留可执行权限的启动器
├── language-server.json                # 构建输入：版本、各平台 URL 与 SHA-256
├── scripts/package.mjs                 # 下载、校验、解包、组装 zip
└── .github/workflows/                  # PR 校验与标签发布

产物 lua-support-0.3.1-darwin-arm64.zip
└── io.github.lithe-idea.lua-support/
    ├── plugin.json
    ├── bin/lithe-lua-language-server
    └── language-server/                # 完整上游资源，不只是可执行文件
        ├── bin/lua-language-server
        └── …                          # 上游依赖、脚本和许可文件
```

模板清单是 API 2：语言服务器之外还声明了运行 `lua <当前文件>` 的执行提供者（`process.run` 权限和
`onRun:lua` 事件），启动器的第一个参数是宿主传入的 `${logDir}`。

派生自己的仓库后，同步修改以下位置：

1. 包目录名、`plugin.json` 的 `id`、显示名、版本与 `vendor`，使用你自己的稳定反向域名。
2. `scripts/package.mjs` 的 `PACKAGE_DIR`、`ASSET_PREFIX`，以及与新服务器对应的资源检查。
3. `language-server.json` 的版本、各平台 URL、摘要和许可信息。它是模板脚本自己的输入，
   不是宿主自动解释的统一下载 API；更换清单结构时也要修改打包脚本。
4. 启动器中的包内入口、参数和缓存命名空间。不能继续写入 Lua 模板的缓存目录。
5. CI 中读取清单的目录名、Release 说明、README、许可证与第三方声明。

不要只换包目录名：模板 CI 中也有原插件 ID，启动器中也有缓存路径。

### 3. 写一个最小、完整的 LSP 清单

下面是一个纯 LSP 示例。它沿用模板的目录布局和启动器，先只接入 LuaLS；如要运行 Lua 程序，
再按[构建工具指南](build-tool-integration.md)添加运行提供者。示例中的作者和 ID 要换成自己的。

```json
{
  "schemaVersion": 2,
  "id": "io.github.alice.lua-support",
  "displayName": "Lua Support",
  "version": "0.1.0",
  "pluginAPIVersion": 2,
  "vendor": {
    "id": "io.github.alice",
    "displayName": "Alice",
    "signatureRequirement": "none"
  },
  "permissions": ["languageServer.launch"],
  "activationEvents": ["onLanguage:lua"],
  "languages": [
    {
      "id": "lua",
      "displayName": "Lua",
      "fileExtensions": ["lua"],
      "projectFileNames": [".luarc.json", ".luarc.jsonc"]
    }
  ],
  "languageServers": [
    {
      "languageID": "lua",
      "languageIdentifier": "lua",
      "launch": {
        "kind": "packaged",
        "path": "bin/lithe-lua-language-server"
      },
      "arguments": ["${logDir}"],
      "validationArguments": []
    }
  ]
}
```

三个版本不要混淆：`schemaVersion: 2` 是清单格式，`pluginAPIVersion: 2` 是宿主需要支持的
插件 API 等级，`version: "0.1.0"` 是你的插件版本。上游 LuaLS 的版本另记在构建输入中。
这里用 API 2，是因为参数里的 `${logDir}` 是 API 2 的启动模板；只声明实际需要的最低 API。
API 1 的服务器参数是字面量，宿主原样传给进程、不展开模板，这时启动器要自己选择日志和缓存目录。

`languages[].id`、`languageServers[].languageID` 和 `onLanguage:<id>` 必须对应。
声明服务器需要 `languageServer.launch` 权限及激活事件；每个语言最多一个服务器。
事件集合必须与实际提供者精确对应，不能随意增加未使用的事件。权限是能力声明，
不是限制外部程序磁盘或网络访问的操作系统沙箱。
启用只表示允许激活，打开项目中相应语言的文档才触发服务器启动。
文件后缀不等于 LSP 的文档标识，必要时单独设置 `languageIdentifier`。

#### 选择启动方式

| 方式 | 适用场景 | 作者要做的事 |
| --- | --- | --- |
| API 1 `packaged` | 随包提供原生服务器或启动器 | 声明相对路径，打包全部资源并保留可执行权限 |
| API 1 `systemCommand` | 明确要求用户自行安装的服务器 | 声明裸命令候选和正确探测参数，记录用户安装要求 |
| API 2 `runtime` | 包内 JS/JAR 由 Node.js/Java 启动 | 声明包内入口及发现的运行时最低版本，或声明随包运行时 |
| `serverProcess` 3 `backend` | 命令行取决于启动时才能确定的内容（解析版本、引导工具、按项目派生选项） | 声明解析命令行的后端函数；见下文“由插件后端驱动的语言服务器” |

例如，下面是 `languageServers[]` 中一个服务器声明的替代方案，要求用户预先安装 `gopls`，
完整清单还必须声明 `go` 语言及相应权限、事件：

```json
{
  "languageID": "go",
  "launch": { "kind": "systemCommand", "commands": ["gopls"] },
  "arguments": ["serve"],
  "validationArguments": ["version"],
  "runtimeDependencies": [{ "tool": "go" }]
}
```

探测参数必须能自行以退出码 0 结束，不能用会持续运行的 LSP 启动参数。若上游没有合适探测
方式，可以像 Lua 模板一样用空数组，让 LSP 握手决定是否可用。通过探测仍不代表补全已经正常。

随包分发时优先使用固定、可复验的服务器。`systemCommand` 不会替用户安装工具，也不能作为
启动时下载服务器的变通方案。运行时依赖与主入口是两回事：例如随包服务器调用 Go 时，
`runtimeDependencies` 让宿主发现 Go 并传递环境；API 1 支持 `node`、`go`，API 2 增加 `java`。

#### 由插件后端驱动的语言服务器（`serverProcess` 3 与 `languageServers` 3）

上面几种方式都由清单直接写出命令行。命令行要到启动时才能确定（解析某个版本、引导一个工具、按项目派生选项），
或者插件需要在服务器运行期间向它发请求、接收它的通知时，可以让插件后端参与。进程、LSP 会话和停止仍归宿主；
后端只回答命令行，并通过宿主方法与**自己的**服务器交谈。

- **后端决定启动方式。** `launch` 写 `{"kind": "backend", "function": "launch.resolve", "timeoutMilliseconds": 30000}`。
  每次启动时，宿主调用这个插件函数，后端返回可执行文件（包内文件，或插件自己定义的工具）、参数、环境变量等，宿主按答案启动。
  这类服务器不再声明 `arguments`、`validationArguments`、`runtimeDependencies`、`initializationOptions` 和 `hostIntegration`。
  它要求 `pluginAPIVersion` 4、`backend` 2、`serverProcess` 3（同时列出 `requiresTools` 时为 4），并声明 `backend.launch` 权限。
- **会话字段（`languageServers` 3）。** 后端只能做服务器条目声明过的事：
  `requestMethods`（允许用 `languageServer/request`、`languageServer/notify` 发的 LSP 方法）、
  `commands`（允许执行的 `workspace/executeCommand` 命令，与 `requestMethods` 是否列出该方法一一对应）、
  `serializedCommands`（`commands` 的子集：同一命令同时只跑一个，后来的请求并入并得到同一个结果）、
  `forwardNotifications`（宿主转给后端的服务器通知）、
  `state`（按工作区的状态目录及 `retention` 的 `maxEntries`、`maxAgeDays`，宿主据此在后台回收旧目录）、
  `remote`（`supported` 或 `unsupported`，后者不在远程工作区启动）。
  这些声明由 Core 的会话策略强制执行：宿主不能放宽，后端也越不过去。
- **权限 `languageServer.request`。** 只有服务器列出 `requestMethods` 或 `forwardNotifications` 时才能声明，反过来也一样；需要 `backend` 2。
  它只授予本插件自己服务器的 `languageServer/*` 宿主方法和下面两个通知。

宿主方法和通知（前缀 `lithe/host/`；方法都要带 `context` 和 `serverID`，`serverID` 是本插件某个服务器的 `languageID`，
别的插件的服务器返回 `plugin.host.foreignContribution`）：

| 方法或通知 | API | 权限 | 用途 |
| --- | --- | --- | --- |
| `languageServer/request` | API-25 | `languageServer.request` | 向自己的服务器发 JSON-RPC 请求，返回服务器的 `result`；超时默认 60 秒、最长 300 秒 |
| `languageServer/notify` | API-25 | `languageServer.request` | 向自己的服务器发通知 |
| `languageServer/setStatus` | API-22、API-25 | `languageServer.request` | 在宿主状态界面显示插件自己的准备状态（`preparing`、`ready`、`degraded`、`failed`，可带消息和百分比），用于导入工程、建索引 |
| `languageServer/didChangeState`（通知） | API-25 | `languageServer.request` | 服务器状态变化（`starting`、`running`、`stopped`、`crashed`），带 `generation` |
| `languageServer/didReceiveNotification`（通知） | API-25 | `languageServer.request` | 转发服务器发出、`forwardNotifications` 列出的通知；过大时只送 `truncated: true` |
| `documents/registerContentProvider` | API-12 | `documents.contentProvider` | 把一个 URI scheme 绑定到后端函数，见下 |

服务器交出的文档没有对应文件（例如反编译的库类）时，用 `documents/registerContentProvider` 提供：用户打开该 scheme 的文档时，
宿主调用绑定的函数取只读内容（最多 1 MiB，宿主等 30 秒）。每个工作区窗口每个 scheme 只能有一个 provider，用 `subscriptions/dispose` 撤销。

边界：仅 macOS。后端决定启动方式的服务器不会在远程工作区启动（远程后端启动未支持）；Windows 宿主不提供 `backend`，也就没有这种启动方式。
官方 Node、Java、Rust SDK 还没有这些方法的封装，可以先用 SDK 的通用宿主请求，以完整方法名调用，
例如 Node SDK 的 `backend.host.request(method, params?)`，见 [Host services](../sdk/node/README.md#host-services)。

细节见[语言、语言服务器与运行/测试](../reference/domains/languages.md)、[编辑器、文档与预览](../reference/domains/editor.md)，
以及契约的 [Backend-resolved launch](../contract/v1/README.md#backend-resolved-launch-serverprocess-3)、
[Plugin-owned sessions](../contract/v1/README.md#plugin-owned-sessions-languageservers-3)、
[Language server host methods](../contract/v1/README.md#language-server-host-methods)、
[Content providers](../contract/v1/README.md#content-providers)。
示例包：[`languages/language-server-backend-launch.plugin.json`](../contract/v1/fixtures/languages/language-server-backend-launch.plugin.json)。

### 4. 在构建阶段准备完整语言服务器

沿用模板的处理顺序：

1. 固定上游发行版本和每个平台/架构的 HTTPS 归档地址，核实许可证及再分发条件。
2. 将归档 SHA-256 写入版本控制。摘要应通过上游校验资料或经过审查的下载确认，
   构建时不能现下载、现算一个摘要就当作预期值。
3. 下载后先核对摘要，再解压到临时组装目录。缓存命中也必须重新校验；不匹配立即失败。
4. 复制服务器所需的全部文件：二进制、脚本、资源、动态库、许可证等；检查声明入口存在。
5. 检查归档没有符号链接、越界路径和开发机绝对路径，保留启动器与二进制的可执行权限。
6. 按平台生成 zip，再计算最终 zip 的 SHA-256，交给市场索引固定。

上游归档摘要与最终 zip 摘要用途不同：前者保证构建输入，后者保证用户拿到同一个发布包。
更换语言服务器时不能只替换下载地址，必须同步检查解压布局、入口、依赖、探测方式及许可。
模板中的归档处理适配 LuaLS 的 tar.gz；其他格式需要在自己的打包脚本中明确实现。

社区插件使用自己的 ID 和 `vendor.signatureRequirement: "none"`，通过市场源的
摘要校验或从磁盘安装时的私有快照、用户确认和加载时内容复查安装。不要伪造 Lithe 发布者签名，
也不要使用保留的 `dev.lithe.*` 命名空间。官方包的发布者签名是另一条信任路径，签名必须在完整组装后完成。

### 5. 将可写状态与安装文件分开

Lithe 会在加载时复查社区包内容。服务器如果将日志、索引、锁或生成文件写回插件目录，
下次加载就可能失败；写入应用安装包还会破坏应用签名与增量更新基线。

| 数据 | 放在哪里 | 需要注意的生命周期 |
| --- | --- | --- |
| 服务器、运行时、默认配置、许可证 | 只读插件包 | 随插件版本安装和替换 |
| 上游原生日志 | API 2 的 `${logDir}`（宿主提供的本次会话日志目录） | 由宿主的日志存储管理，不写结构化事件，见[插件日志](plugin-logging.md) |
| 可重建缓存 | `${stateDir}`，或平台用户缓存目录 | 设置容量策略，记录清理方法 |
| 项目索引、锁、项目相关状态 | `${workspaceStateDir}` 等按工作区隔离的可写目录 | 不同项目、工作树不能共用同一份可变索引 |
| 用户源码与构建产物 | 用户工作区或构建工具指定目录 | 卸载插件不应误删用户工程 |

Lua 模板的启动器把宿主传入的 `${logDir}` 交给 LuaLS 的 `--logpath`，把生成的库定义放在
`~/Library/Caches/io.github.lithe-idea.lua-support/meta`（设置了 `XDG_CACHE_HOME` 时放在它下面），
再用 `exec` 将启动器替换为真实服务器。
不要把服务器放到后台、`nohup` 或自行创建常驻守护进程，否则宿主难以完整停止它。
标准输出只承载 LSP 的 JSON-RPC 帧，启动器提示与日志应写标准错误或日志文件。

模板中的自定义缓存路径属于启动器自己的约定，**不能假定宿主卸载时自动清理**。
需要说明残留内容及安全删除方法。若自己的服务器维护工作区索引，还要增加稳定的工作区隔离，
不要直接复用模板的单一缓存目录保存所有项目索引。

#### API 2：让宿主提供状态目录

也可以去掉启动器，把上面示例的 `languageServers` 替换为以下值，直接运行包内 LuaLS
并传入宿主管理的目录：

```json
[
  {
    "languageID": "lua",
    "languageIdentifier": "lua",
    "launch": {
      "kind": "packaged",
      "path": "language-server/bin/lua-language-server"
    },
    "arguments": [
      "--logpath=${logDir}",
      "--metapath=${workspaceStateDir}"
    ],
    "validationArguments": []
  }
]
```

`${logDir}` 是宿主显式提供的本次会话插件日志目录；宿主不提供 `logDirectory` 时
（目前的 Windows 宿主就是这样），用到它的启动以 `plugin.launch.invalidRequest` 被拒绝。
`${stateDir}` 是宿主为该插件服务器管理的可写状态目录，卸载插件时删除；`${workspaceStateDir}` 进一步
按工作区隔离。`${pluginDir}` 是只读包根路径；`${arch}` 的值是 `arm64` 或 `x86_64`，
与 Release 资产里的 `x64` 拼写不同。状态和日志模板不能放进 `validationArguments`。
不要将这些模板写进 API 1 的参数，宿主会按原样传给进程。

如果服务器需要 Node.js/Java，API 2 的 `runtime` 可免去启动器。以下是服务器声明中的
`launch` 片段，入口要替换成自己的真实资源，服务器专用参数仍放在 `arguments`：

```json
{
  "kind": "runtime",
  "tool": "node",
  "minimumVersion": "18.17",
  "entry": "language-server/server.js"
}
```

宿主负责发现并检查运行时；不要再在 `runtimeDependencies` 中重复声明同一个启动工具。
`runtime` 不声明 `validationArguments`，运行时探测由宿主执行。若改为 `packagedRuntime`，
该字段与 `minimumVersion` 互斥，插件要随包提供运行时、依赖和许可，并分别验证目标架构。

API 2 也支持 `initializationOptions` 和声明式 `settings`。只传上游真实支持的选项；
`${setting:<id>}` 只能引用已声明设置，值必须符合类型和范围。设置是插件级、跨工作区共享，
修改后宿主重启相应服务器。详细格式见[正式契约](../contract/v1/README.md#plugin-settings-api-2)。

### 6. 本地验证、安装与激活

先运行模板现有 CI 中的两层检查：`node scripts/package.mjs` 组装产物，随后用 Lithe Store
的 `readPackageManifests` 和 `checkManifest` 检查包路径、ID、版本和社区信任声明。
可参考模板的 [CI 工作流](https://github.com/Lithe-IDEA/lithe-plugin-lua/blob/main/.github/workflows/ci.yml)。
这些检查不会启动服务器，也不等同于完整的 Core 清单语义校验。
在插件仓库根目录可以按 CI 的方式本地检查产物；下面的相邻目录应使用一个尚未占用的名字：

```bash
git clone --depth 1 https://github.com/Lithe-IDEA/Lithe-store.git ../lithe-store-validation
node --input-type=module <<'JS'
import { readdirSync, readFileSync } from "node:fs";
import { readPackageManifests, checkManifest } from "../lithe-store-validation/scripts/validate-index.mjs";
const manifest = JSON.parse(readFileSync("io.github.lithe-idea.lua-support/plugin.json", "utf8"));
for (const name of readdirSync("dist")) {
  const { manifests, unsafe } = readPackageManifests("dist/" + name, manifest.id);
  const problems = [...unsafe, ...checkManifest(manifests, { id: manifest.id }, manifest)];
  if (problems.length) throw new Error(name + ": " + problems.join("; "));
  console.log("validated " + name);
}
JS
```

派生模板后，把这里读取 `plugin.json` 的目录名也换成自己的 ID。
如果错误来自跨引用、模板上下文或 API 等级，还需对照契约并通过目标宿主的安装校验。

macOS 宿主上有两条安装 `none` 社区包的路径，两条都会显示安装审查，由用户确认：

- **从磁盘安装**：在插件管理中选择“从磁盘安装插件…”，选中 `dist/` 里的 zip 或解压后的包目录。
  宿主校验清单后保存一份私有快照，并把它标为发布者与源码仓库都未经验证的社区插件。
  它适合开发中反复验证，但不检查索引条目和 Release 资产的摘要。
- **自己的插件源**：按[发布流程](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)在自己的公开 GitHub 仓库
  发布 zip，再准备仓库根目录的 `lithe-plugin-index.json`。索引格式与 Store 相同，
  下载 URL 指向该仓库的 Release 资产，摘要固定最终 zip；不必先申请进入公共 Store。
  在 Lithe 的“管理插件仓库…”中加入你的仓库 URL，即可通过自己的源安装，走的是用户真正会用的路径。

发布前至少用自己的插件源完整安装一次。开发构建会把未签名、但要求发布者签名的包按本地开发包处理，
这属于单独的信任路径，不能替代社区安装验收。

安装前先禁用与你的插件声明同一语言的其他插件，避免提供者冲突。然后：

1. 在插件市场选择你的源和版本（或从磁盘选择包），确认显示的程序、权限、来源都符合预期。
   语言服务器插件安装或更新后，按界面提示**重启 Lithe**，让新版本进入本次启动的插件图；
   宿主只对仅含调试适配器或语法的插件免重启（见第 7 节）。
2. 打开一个小项目中的 `.lua` 文件。应当出现补全、悬停、跳转和诊断；
   不要只检查进程“已运行”。
3. 保存修改，确认诊断更新；关闭并重开文件，确认没有重复服务器和重复诊断。
4. 从 Dock 启动 Lithe 再试一次，避免依赖终端启动时偶然继承的 PATH。
5. 若增加运行/测试，分别验证；LuaLS 已正常工作并不代表用户的 `lua` 已安装。

启动后的协议顺序也应能解释清楚：宿主发送 `initialize`，服务器返回能力后接收
`initialized`，随后按协商能力通过 `didOpen`、`didChange`、`didSave`、`didClose` 同步文档，
补全等请求和诊断沿同一会话传输。正常停止先 `shutdown`，再 `exit`；初始化未完成或关闭
超时时，宿主可以强停进程。当前 macOS 默认初始化/请求超时为 30 秒、关闭超时为 2 秒。
取消请求通过 `$/cancelRequest` 通知服务器，服务器仍可能完成计算，不能将取消当作进程已退出。

可用这个文件进行最小交互验收：

```lua
---@param name string
local function greet(name)
  return "Hello, " .. name
end

print(greet("Lithe"))
```

悬停查看 `greet` 的签名，跳到定义，再将调用参数改成数字，观察类型诊断。
具体提示随 LuaLS 版本和项目配置变化，验收时应记录所用版本和配置。

### 7. 把停止、失败和卸载也当作功能验收

声明式插件没有让作者注册任意 `activate()` / `deactivate()` 回调的机制。
宿主管理会话、进程和停用；作者负责让服务器保持前台运行、遵守 LSP 生命周期，
并把所有子进程和状态设计成可以停止、重启和隔离。在 macOS 上，Core 的强停只针对直接子进程，
不能假定会自动终止服务器任意派生的整棵进程树；Windows 宿主按进程树结束。下面是发布前需要执行的验收，
不是声明后即可免测的保证。

| 操作或故障 | 要检查的结果 |
| --- | --- |
| 仅安装/启用，不打开该语言 | 不因安装就提前启动语言服务器 |
| 编辑、保存、关闭/重开文件 | 文档同步和诊断正常，没有重复会话 |
| 快速切换两个项目/工作区 | 旧结果不串到新项目，索引与锁不互相占用 |
| 服务器异常退出或初始化失败 | 宿主能显示可定位的失败；恢复操作不会留下重复进程 |
| 缺少 Node.js/Go/Java 或版本不足 | 给出可操作的运行时错误，不在启动时自行下载 |
| 修改 API 2 设置 | 新会话使用新值，旧进程退出 |
| 禁用插件、关闭工作区、退出 Lithe | 相应会话与进程被停止，启动器/辅助子进程不残留 |
| 卸载与重新安装 | 卸载先停用，重启后删除包和宿主管理的插件状态；自定义缓存有明确说明 |
| 篡改安装包内一个文件后重新加载 | 内容复查拒绝加载，不能靠服务器自行修改清单“修复” |

已在本次启动中注册的插件可即时启用/禁用。安装、更新、回滚、恢复和卸载是否免重启由宿主判断：
只有贡献全部是调试适配器或语法、且没有后端的插件，才在运行中立即生效；语言服务器属于启动时构建的
模块图，所以 LSP 插件的这些文件/版本转换都按界面提示在下次启动完成，
卸载请求会先停用已注册插件。验收需要覆盖重启前和重启后的状态。

验收完成后关闭测试启动的应用、服务器和子进程。不要把关闭编辑器标签等同于卸载服务器，
也不要假定崩溃后一定会自动重试或所有平台都有相同恢复策略。发现生命周期问题时，
在插件仓库记录日志、宿主版本、包版本、架构和复现步骤；宿主进程管理问题再提交到 Lithe。

### 8. 发布、更新与回退演练

按[发布社区插件](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md)递增插件版本、发布正式 Release 并首次提交收录。
模板的 Release 工作流会校验 `v<版本>` 与清单一致；原版示例是
`node scripts/package.mjs --expect-tag v0.3.1`，派生后使用自己的版本。

每次更新服务器或运行时，都要更新固定输入和许可，重建对应平台包，走完交互及生命周期验收。
发布后的 zip 不可替换，同一版本不能重用；修复要发布新版本。新增可运行程序会触发新的
安装确认，具体规则见[版本追踪与更新](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md)。

发布前至少演练一次“旧版 → 新版 → 恢复旧版逻辑”：

- 保留旧版本资产和索引记录，确保旧宿主仍有兼容版本可装。
- 新旧服务器使用不兼容索引格式时，隔离状态或提供安全重建；不能让旧版读损坏状态。
- macOS 的回滚功能使用安装记录中保留的上一版本，重新验证该包；LSP 插件在重启后切换。
  它不提供任意历史版本选择，也不会恢复服务器索引或用户数据。应实际演练该路径。
- 当前版本有问题时，发布更高版本的修复包，必要时在新包里恢复上一个稳定服务器；
  同时申请撤回问题版本，不能只依赖用户机器恰好保留上一版本。
- 卸载重装也要验证。源码、用户构建产物与自定义缓存的处理必须说明，避免误删用户数据。

### 9. 按失败阶段排查

| 现象 | 优先检查 |
| --- | --- |
| 市场看不到或不可安装 | 源地址、当前平台资产、`pluginAPIVersion` 是否被宿主支持 |
| `plugin.archive.checksumMismatch` | 是否替换过 Release 资产；索引固定的是否为最终 zip 摘要 |
| `plugin.manifest.invalid` | 未知字段、schema/API 混淆、ID/引用、API 2 字段误放到 API 1 |
| `plugin.permission.undeclared` / `plugin.activation.invalid` | 提供者、权限与事件是否一一对应 |
| `plugin.launch.missing` | 相对路径、大小写、打包完整性与可执行权限 |
| `plugin.launch.invalidRequest` | 用到 `${logDir}` 或 `${workspaceStateDir}`，但宿主没有提供对应目录 |
| `plugin.runtime.missing` | 运行时发现、最低版本、Dock 启动环境 |
| 有进程但无法补全 | stdio 是否被日志污染、LSP 握手、文档语言标识与上游配置 |
| 用过一次后拒绝加载 | 服务器是否向包内写了日志、索引、锁或生成文件 |
| 禁用后仍有进程 | 启动器是否使用 `exec`，是否后台化或产生未清理辅助进程 |
| 能补全但无法构建/测试 | 是否实际声明并验证了独立的运行/测试或构建集成 |

字段全集、API 等级和稳定错误码以 Lithe 的
[插件契约](../contract/v1/README.md)及
[清单 Schema](../contract/v1/plugin-manifest.schema.json)为准。
契约副本固定在 Lithe 主仓库的某个提交（见 [`contract.lock.json`](../contract.lock.json)），本文的宿主现状按主仓库最新 `preview` 核对；
两者都可能包含还没进入 Lithe 稳定版的能力，发布前以目标宿主实际支持为准。

---

## English

This tutorial is for plugin authors who want to add language support to Lithe. Using the runnable
[Lua Support template](https://github.com/Lithe-IDEA/lithe-plugin-lua), it walks through choosing a server, the manifest, packaging the language server,
installation and activation, daily operation, shutdown, upgrade, and uninstall. For build, run, and test, see the
[build tool integration guide](build-tool-integration.md). For publishing, see [Publishing a community plugin](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md).

The baseline is the **macOS host** on the latest `preview` of the Lithe repository: a community plugin installs from a marketplace source or from disk.
The Windows host currently installs declarative packages from disk only, has no marketplace browser, does not support execution providers (`executionProviders`),
and supplies no `logDirectory` to launch plans, so a launch that uses `${logDir}` is rejected on Windows.
That an index can describe Windows or Linux packages does not mean the corresponding host implements the same install and run flow.
The Lua template ships only macOS arm64 and x64 packages. For capability versions, see the [version tables](../reference/capabilities-and-versions.md).

### 1. Decide what the plugin owns and what the host owns

LSP (Language Server Protocol) lets an editor get completion, diagnostics, navigation, and more
from a separate process. A Lithe declarative plugin describes the program to start in JSON; the host reads the manifest, discovers runtimes, and manages
the process and the LSP session. You do not write a Swift or JavaScript entry point inside the host. A server whose command line is only known at launch
can have the plugin backend resolve it at each start; see "Language servers driven by the plugin backend" in section 3.

```text
Develop and build: pin upstream version → download and verify → assemble per-platform packages → publish zips → index entry
User install:      download zip (or pick a package on disk) → verify checksum and manifest → confirm programs and permissions → install and record integrity digests
Normal operation:  enable → open a file in the language → start the server → initialize LSP → edit and diagnose
Ending use:        disable / close workspace → host stops sessions and processes; update / uninstall → replace or remove the package
```

| Concern | The plugin author owns | Lithe owns |
| --- | --- | --- |
| Language detection | File extensions, full file names, project markers | Matching files and choosing a provider |
| Language server | Choosing upstream, license, pinned version, entry point, correct arguments | Validating declarations, starting on demand, LSP communication and shutdown |
| Runtime | Declaring dependencies, or shipping one where the API allows | Discovering, checking, and starting runtimes as declared |
| Run and test | Declaring commands, arguments, and test scopes | Building plans, executing, and showing results |
| Writable data | The log, index, and state paths the upstream uses | Platform-writable directories for API 2 launch templates |
| Release and maintenance | Per-platform artifacts, version upgrades, compatibility and rollback checks | Download verification, install review, enabling and stopping plugins |

Pick a mature language server whose license allows redistribution. Project import, dependency resolution, and symbol and module models should come from
the language's mature backend wherever possible. Do not duplicate a project model just to "hook up LSP". What LSP features you get still depends on the server's
capabilities and on host support; installing a server does not bring build, test, DAP debugging, or custom UI.

### 2. Run the original template, then make it your own plugin

You need Git, Node.js 22+, `tar`, `zip`, and `unzip`. The packaging tool uses Node.js;
that does not mean the target language server needs Node.js. LuaLS is a platform-specific native program.

```bash
git clone https://github.com/Lithe-IDEA/lithe-plugin-lua.git
cd lithe-plugin-lua
node scripts/package.mjs
unzip -l dist/lua-support-0.3.1-darwin-arm64.zip
shasum -a 256 dist/lua-support-0.3.1-darwin-arm64.zip
```

`0.3.1` is the template version when this page was checked; after upstream updates, take the artifact names from the template manifest and the script output.
The script regenerates `dist/`, downloads LuaLS for both macOS architectures, checks the pinned SHA-256 values, and prints the SHA-256 of each zip.
Linux CI can assemble the macOS packages, but that only proves packaging works; it does not replace running on a target Mac.

How the template source maps to the artifacts:

```text
Source repository
├── io.github.lithe-idea.lua-support/
│   ├── plugin.json
│   └── bin/lithe-lua-language-server    # launcher, keeps its executable bit
├── language-server.json                # build input: version, per-platform URLs and SHA-256
├── scripts/package.mjs                 # download, verify, unpack, assemble zips
└── .github/workflows/                  # PR checks and tag releases

Artifact lua-support-0.3.1-darwin-arm64.zip
└── io.github.lithe-idea.lua-support/
    ├── plugin.json
    ├── bin/lithe-lua-language-server
    └── language-server/                # the complete upstream resources, not just the executable
        ├── bin/lua-language-server
        └── …                          # upstream dependencies, scripts, and license files
```

The template manifest is API 2: besides the language server it declares an execution provider that runs `lua <current file>` (the `process.run` permission and
the `onRun:lua` event), and the launcher's first argument is the `${logDir}` the host passes in.

After forking, change all of these together:

1. The package directory name and, in `plugin.json`, the `id`, display name, version, and `vendor`, using your own stable reverse domain.
2. `PACKAGE_DIR` and `ASSET_PREFIX` in `scripts/package.mjs`, and the resource checks for your server.
3. The version, per-platform URLs, digests, and license information in `language-server.json`. It is the template script's own input,
   not a unified download API the host interprets; if you change its structure, change the packaging script too.
4. The in-package entry, arguments, and cache namespace in the launcher. Do not keep writing into the Lua template's cache directory.
5. The directory name CI reads the manifest from, the Release notes, the README, the license, and third-party notices.

Do not change only the package directory name: the template's CI also contains the original plugin ID, and the launcher contains its cache path.

### 3. Write a minimal, complete LSP manifest

Below is an LSP-only example. It keeps the template's layout and launcher and wires up only LuaLS; to run Lua programs,
add an execution provider following the [build tool guide](build-tool-integration.md). Replace the author and ID with your own.

```json
{
  "schemaVersion": 2,
  "id": "io.github.alice.lua-support",
  "displayName": "Lua Support",
  "version": "0.1.0",
  "pluginAPIVersion": 2,
  "vendor": {
    "id": "io.github.alice",
    "displayName": "Alice",
    "signatureRequirement": "none"
  },
  "permissions": ["languageServer.launch"],
  "activationEvents": ["onLanguage:lua"],
  "languages": [
    {
      "id": "lua",
      "displayName": "Lua",
      "fileExtensions": ["lua"],
      "projectFileNames": [".luarc.json", ".luarc.jsonc"]
    }
  ],
  "languageServers": [
    {
      "languageID": "lua",
      "languageIdentifier": "lua",
      "launch": {
        "kind": "packaged",
        "path": "bin/lithe-lua-language-server"
      },
      "arguments": ["${logDir}"],
      "validationArguments": []
    }
  ]
}
```

Keep the three versions apart: `schemaVersion: 2` is the manifest format, `pluginAPIVersion: 2` is the plugin API level the host must support,
and `version: "0.1.0"` is your plugin's version. The upstream LuaLS version lives in the build input.
The example uses API 2 because `${logDir}` in its arguments is an API 2 launch template; declare only the lowest API you actually need.
In API 1, server arguments are literals that the host passes unchanged without expanding templates, so the launcher must pick its own log and cache directories.

`languages[].id`, `languageServers[].languageID`, and `onLanguage:<id>` must match.
A server needs the `languageServer.launch` permission and an activation event; each language has at most one server.
The event set must match the actual providers exactly; do not add unused events. Permissions declare capabilities;
they are not an operating-system sandbox that limits an external program's disk or network access.
Enabling only allows activation; opening a document of the language in a project starts the server.
A file extension is not the LSP document identifier; set `languageIdentifier` separately when needed.

#### Choosing a launch kind

| Kind | When to use it | What the author does |
| --- | --- | --- |
| API 1 `packaged` | A native server or launcher shipped in the package | Declare the relative path, package every resource, and keep the executable bit |
| API 1 `systemCommand` | A server the user is explicitly expected to install | Declare bare command candidates and a correct probe, and document the install requirement |
| API 2 `runtime` | A packaged JS file or JAR started by Node.js or Java | Declare the packaged entry and the discovered runtime's minimum version, or declare a packaged runtime |
| `serverProcess` 3 `backend` | A command line that is only known at launch (resolving a version, bootstrapping a tool, deriving options from the project) | Declare the backend function that resolves the command line; see "Language servers driven by the plugin backend" below |

For example, this is an alternative server declaration in `languageServers[]` that expects the user to have installed `gopls`;
the full manifest must also declare the `go` language and the matching permission and event:

```json
{
  "languageID": "go",
  "launch": { "kind": "systemCommand", "commands": ["gopls"] },
  "arguments": ["serve"],
  "validationArguments": ["version"],
  "runtimeDependencies": [{ "tool": "go" }]
}
```

Probe arguments must exit with code 0 on their own; never use the long-running LSP start arguments. If the upstream has no suitable probe,
use an empty array as the Lua template does, and let the LSP handshake decide. Passing the probe still does not mean completion works.

When shipping a server, prefer a pinned, reproducible one. `systemCommand` does not install tools for the user, and it is not a workaround for
downloading a server at launch. Runtime dependencies are separate from the main entry: for example, when a packaged server calls Go,
`runtimeDependencies` makes the host discover Go and pass the environment. API 1 supports `node` and `go`; API 2 adds `java`.

#### Language servers driven by the plugin backend (`serverProcess` 3 and `languageServers` 3)

The kinds above all write the command line into the manifest. When the command line is only known at launch (resolving a version, bootstrapping a tool,
deriving options from the project), or when the plugin needs to send requests to the running server and receive its notifications, the plugin backend can take part.
The process, the LSP session, and stopping still belong to the host; the backend only answers the command line and talks to **its own** server
through host methods.

- **The backend decides how to launch.** Write `launch` as `{"kind": "backend", "function": "launch.resolve", "timeoutMilliseconds": 30000}`.
  At each start the host calls that plugin function, the backend returns the executable (a file in the package, or a tool the plugin defines), the arguments, the environment, and so on,
  and the host starts the server from the answer. Such a server no longer declares `arguments`, `validationArguments`, `runtimeDependencies`, `initializationOptions`, or `hostIntegration`.
  It requires `pluginAPIVersion` 4, `backend` 2, and `serverProcess` 3 (4 when the manifest also lists `requiresTools`), and declares the `backend.launch` permission.
- **Session fields (`languageServers` 3).** The backend may do only what the server entry declares:
  `requestMethods` (the LSP methods `languageServer/request` and `languageServer/notify` may send),
  `commands` (the `workspace/executeCommand` commands it may run; declared exactly when `requestMethods` lists that method),
  `serializedCommands` (a subset of `commands`: one runs at a time, and a later request joins it and gets the same result),
  `forwardNotifications` (server notifications the host forwards to the backend),
  `state` (per-workspace state directories with a `retention` of `maxEntries` and `maxAgeDays`, which the host uses to prune old directories in the background),
  and `remote` (`supported` or `unsupported`; the latter is not started for a remote workspace).
  Core's session policy enforces these declarations: the host cannot widen them and the backend cannot step outside them.
- **The `languageServer.request` permission.** It can be declared only when a server lists `requestMethods` or `forwardNotifications`, and the other way round; it needs `backend` 2.
  It grants only the `languageServer/*` host methods and the two notifications below, for the plugin's own servers.

Host methods and notifications (prefix `lithe/host/`; every method takes `context` and `serverID`, the `languageID` of one of the plugin's servers;
a server of another plugin returns `plugin.host.foreignContribution`):

| Method or notification | API | Permission | Purpose |
| --- | --- | --- | --- |
| `languageServer/request` | API-25 | `languageServer.request` | Send a JSON-RPC request to your own server and get the server's `result`; the timeout defaults to 60 s and is at most 300 s |
| `languageServer/notify` | API-25 | `languageServer.request` | Send a notification to your own server |
| `languageServer/setStatus` | API-22, API-25 | `languageServer.request` | Show the plugin's own preparation state in the host's status UI (`preparing`, `ready`, `degraded`, `failed`, with an optional message and percentage), for importing a project or indexing |
| `languageServer/didChangeState` (notification) | API-25 | `languageServer.request` | The server's state changed (`starting`, `running`, `stopped`, `crashed`), with a `generation` |
| `languageServer/didReceiveNotification` (notification) | API-25 | `languageServer.request` | A notification the server sent and `forwardNotifications` lists; only `truncated: true` is delivered when it is too large |
| `documents/registerContentProvider` | API-12 | `documents.contentProvider` | Bind a URI scheme to a backend function; see below |

When the server hands out documents that have no file (for example the decompiled source of a library class), serve them with `documents/registerContentProvider`: when the user opens a document of that scheme,
the host calls the bound function for its read-only content (at most 1 MiB; the host waits 30 seconds). Each workspace window can have one provider per scheme; revoke it with `subscriptions/dispose`.

Limits: macOS only. A server whose launch the backend resolves does not start in a remote workspace (remote backend launch is not supported); the Windows host does not offer `backend`, so it has no such launch kind.
The official Node, Java, and Rust SDKs do not wrap these methods yet; until they do, call them with the SDK's generic host request using the full method name,
for example `backend.host.request(method, params?)` in the Node SDK, see [Host services](../sdk/node/README.md#host-services).

For details, see [Languages, language servers, and run and test](../reference/domains/languages.md) and [Editor, documents, and previews](../reference/domains/editor.md),
and these contract sections: [Backend-resolved launch](../contract/v1/README.md#backend-resolved-launch-serverprocess-3),
[Plugin-owned sessions](../contract/v1/README.md#plugin-owned-sessions-languageservers-3),
[Language server host methods](../contract/v1/README.md#language-server-host-methods), and
[Content providers](../contract/v1/README.md#content-providers).
Example package: [`languages/language-server-backend-launch.plugin.json`](../contract/v1/fixtures/languages/language-server-backend-launch.plugin.json).

### 4. Prepare the complete language server at build time

Follow the template's order:

1. Pin the upstream release version and the HTTPS archive URL for each platform and architecture, and check the license and redistribution terms.
2. Commit the archive SHA-256 values. Confirm them from the upstream's checksum data or from a reviewed download;
   never download at build time and treat a freshly computed digest as the expected value.
3. Verify the digest after downloading, then unpack into a temporary assembly directory. Re-verify on a cache hit too; fail immediately on a mismatch.
4. Copy every file the server needs: binaries, scripts, resources, dynamic libraries, licenses; check that the declared entry exists.
5. Check that the archive has no symbolic links, paths escaping the root, or absolute paths from the build machine, and keep the executable bit on the launcher and binaries.
6. Produce one zip per platform, then compute the SHA-256 of each final zip for the marketplace index to pin.

The upstream archive digest and the final zip digest serve different purposes: the first guarantees the build input, the second that users get the same released package.
Switching language servers is more than swapping the download URL; recheck the unpacked layout, entry, dependencies, probe, and license.
The template's archive handling fits LuaLS's tar.gz; other formats need explicit handling in your own packaging script.

A community plugin uses its own ID and `vendor.signatureRequirement: "none"`, and installs through a marketplace source's
checksum or, from disk, a private snapshot, plus user confirmation and a content recheck at load. Do not forge a Lithe publisher signature,
and do not use the reserved `dev.lithe.*` namespace. Official packages' publisher signatures are a separate trust path, and signing must happen after full assembly.

### 5. Keep writable state apart from installed files

Lithe rechecks a community package's contents at load. If the server writes logs, indexes, locks, or generated files back into the plugin directory,
the next load may fail; writing into the application bundle also breaks the app signature and the delta-update baseline.

| Data | Where it goes | Lifecycle to consider |
| --- | --- | --- |
| Server, runtime, default configuration, licenses | The read-only plugin package | Installed and replaced with the plugin version |
| Native upstream logs | API 2 `${logDir}` (the current-session log directory the host supplies) | Managed by the host's log storage; not structured events, see [plugin logging](plugin-logging.md) |
| Rebuildable caches | `${stateDir}`, or the platform user cache directory | Set a size policy and document how to clean up |
| Project indexes, locks, project state | A per-workspace writable directory such as `${workspaceStateDir}` | Different projects and worktrees must not share one mutable index |
| User source and build output | The user's workspace or the directory the build tool chooses | Uninstalling the plugin must not delete user projects |

The Lua template's launcher passes the host's `${logDir}` to LuaLS's `--logpath`, keeps generated library definitions in
`~/Library/Caches/io.github.lithe-idea.lua-support/meta` (under `XDG_CACHE_HOME` when set),
and then uses `exec` to replace itself with the real server.
Do not background the server, use `nohup`, or create your own daemon, or the host cannot stop it completely.
Standard output carries only LSP JSON-RPC frames; launcher messages and logs go to standard error or a log file.

The template's custom cache path is the launcher's own convention, and **you cannot assume the host cleans it up on uninstall**.
Document what remains and how to delete it safely. If your server keeps workspace indexes, add stable per-workspace isolation;
do not reuse the template's single cache directory for every project's index.

#### API 2: let the host provide state directories

You can also drop the launcher: replace `languageServers` in the example above with the following to run the packaged LuaLS directly
with host-managed directories:

```json
[
  {
    "languageID": "lua",
    "languageIdentifier": "lua",
    "launch": {
      "kind": "packaged",
      "path": "language-server/bin/lua-language-server"
    },
    "arguments": [
      "--logpath=${logDir}",
      "--metapath=${workspaceStateDir}"
    ],
    "validationArguments": []
  }
]
```

`${logDir}` is the current-session plugin log directory the host supplies explicitly; when the host supplies no `logDirectory`
(as the Windows host does today), a launch that uses it is rejected with `plugin.launch.invalidRequest`.
`${stateDir}` is a writable state directory the host keeps for this plugin's server and removes on uninstall; `${workspaceStateDir}` is further
isolated per workspace. `${pluginDir}` is the read-only package root; `${arch}` is `arm64` or `x86_64`,
spelled differently from the `x64` in Release asset names. State and log templates are not allowed in `validationArguments`.
Do not put these templates in API 1 arguments; the host passes them to the process unchanged.

If the server needs Node.js or Java, an API 2 `runtime` launch removes the need for a launcher. This is the `launch` fragment of a server declaration;
replace the entry with your real resource, and keep server-specific arguments in `arguments`:

```json
{
  "kind": "runtime",
  "tool": "node",
  "minimumVersion": "18.17",
  "entry": "language-server/server.js"
}
```

The host discovers and checks the runtime; do not also declare the same launch tool in `runtimeDependencies`.
A `runtime` launch declares no `validationArguments`; the host probes the runtime. With `packagedRuntime` instead,
which excludes `minimumVersion`, the plugin ships the runtime, its dependencies, and licenses, and must be verified on each target architecture.

API 2 also supports `initializationOptions` and declarative `settings`. Pass only options the upstream really supports;
`${setting:<id>}` may reference only declared settings, and values must match their type and range. Settings are per plugin and shared across workspaces;
after a change, the host restarts the affected servers. For the format, see the [contract](../contract/v1/README.md#plugin-settings-api-2).

### 6. Verify locally, install, and activate

First run the two checks in the template's CI: `node scripts/package.mjs` assembles the artifacts, then Lithe Store's
`readPackageManifests` and `checkManifest` check package paths, the ID, the version, and the community trust declaration.
See the template's [CI workflow](https://github.com/Lithe-IDEA/lithe-plugin-lua/blob/main/.github/workflows/ci.yml).
These checks do not start the server and are not the full Core semantic manifest validation.
From the plugin repository root you can check the artifacts the same way CI does; pick an unused name for the sibling directory:

```bash
git clone --depth 1 https://github.com/Lithe-IDEA/Lithe-store.git ../lithe-store-validation
node --input-type=module <<'JS'
import { readdirSync, readFileSync } from "node:fs";
import { readPackageManifests, checkManifest } from "../lithe-store-validation/scripts/validate-index.mjs";
const manifest = JSON.parse(readFileSync("io.github.lithe-idea.lua-support/plugin.json", "utf8"));
for (const name of readdirSync("dist")) {
  const { manifests, unsafe } = readPackageManifests("dist/" + name, manifest.id);
  const problems = [...unsafe, ...checkManifest(manifests, { id: manifest.id }, manifest)];
  if (problems.length) throw new Error(name + ": " + problems.join("; "));
  console.log("validated " + name);
}
JS
```

After forking, change the directory this reads `plugin.json` from to your own ID.
If an error comes from cross-references, template contexts, or the API level, also check the contract and pass the target host's install validation.

The macOS host has two paths for installing a `none` community package, and both show an install review for the user to confirm:

- **Install from disk**: in plugin management choose "Install Plugin from Disk…" and pick a zip from `dist/` or an unpacked package directory.
  The host validates the manifest, keeps a private snapshot, and marks it as a community plugin whose publisher and source repository are unverified.
  It suits repeated checks during development, but it does not check an index entry or the Release asset's checksum.
- **Your own plugin source**: following the [publishing guide](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md), publish the zips in your own public GitHub repository,
  then add a `lithe-plugin-index.json` at the repository root. The index format is the same as the Store's;
  the download URLs point at that repository's Release assets, and the digests pin the final zips. You do not need to apply to the public Store first.
  Add your repository URL in Lithe's "Manage Plugin Repositories…" to install from your source, which is the path users will actually take.

Before publishing, install from your own plugin source at least once end to end. Development builds treat an unsigned package that requires a publisher signature
as a local development package; that is a separate trust path and does not replace community install acceptance.

Before installing, disable other plugins that declare the same language as yours, to avoid provider conflicts. Then:

1. In the plugin marketplace, choose your source and version (or pick the package on disk), and confirm the programs, permissions, and source shown are what you expect.
   After installing or updating a language server plugin, **restart Lithe** when prompted so the new version joins this launch's plugin graph;
   the host skips the restart only for plugins with nothing but debug adapters or grammars (see section 7).
2. Open a `.lua` file in a small project. You should see completion, hover, go to definition, and diagnostics;
   do not just check that the process "is running".
3. Save a change and confirm diagnostics update; close and reopen the file, and confirm there is no duplicate server and no duplicate diagnostics.
4. Launch Lithe from the Dock and try again, so you do not depend on a PATH inherited by chance from a terminal launch.
5. If you add run or test, verify them separately; LuaLS working does not mean the user's `lua` is installed.

You should also be able to explain the protocol order after start: the host sends `initialize`, the server returns its capabilities and then receives
`initialized`, documents sync through `didOpen`, `didChange`, `didSave`, and `didClose` per the negotiated capabilities,
and completion requests and diagnostics travel over the same session. A normal stop is `shutdown`, then `exit`; when initialization does not finish or shutdown
times out, the host may force-stop the process. The current macOS defaults are 30 seconds for initialization and requests, and 2 seconds for shutdown.
Cancellation reaches the server as `$/cancelRequest`; the server may still finish the computation, so do not treat cancellation as the process having exited.

Use this file for a minimal interactive check:

```lua
---@param name string
local function greet(name)
  return "Hello, " .. name
end

print(greet("Lithe"))
```

Hover `greet` to see its signature, go to its definition, then change the call's argument to a number and watch for the type diagnostic.
The exact messages vary with the LuaLS version and project configuration, so record both when you check.

### 7. Treat stop, failure, and uninstall as acceptance criteria

Declarative plugins have no mechanism for authors to register arbitrary `activate()` / `deactivate()` callbacks.
The host manages sessions, processes, and deactivation; the author keeps the server in the foreground, follows the LSP lifecycle,
and designs every child process and all state to be stoppable, restartable, and isolated. On macOS, Core's force-stop targets the direct child only,
so you cannot assume it ends an arbitrary process tree the server spawns; the Windows host ends the process tree. The following are checks to run before publishing,
not guarantees you get by declaring something.

| Action or failure | What to check |
| --- | --- |
| Install or enable only, without opening the language | The language server does not start just because it was installed |
| Edit, save, close and reopen files | Document sync and diagnostics work, with no duplicate sessions |
| Switch quickly between two projects or workspaces | Old results do not leak into the new project; indexes and locks do not collide |
| The server exits unexpectedly or fails to initialize | The host shows a failure you can trace; recovery leaves no duplicate processes |
| Node.js, Go, or Java missing or too old | An actionable runtime error, with no download at launch |
| Change an API 2 setting | New sessions use the new value, and the old process exits |
| Disable the plugin, close the workspace, quit Lithe | The sessions and processes stop, with no launcher or helper child left behind |
| Uninstall and reinstall | Uninstall deactivates first; after restart the package and host-managed plugin state are removed; custom caches are documented |
| Tamper with one file in the installed package and reload | The content recheck refuses to load it; the server must not "repair" the manifest itself |

A plugin registered in this launch can be enabled and disabled immediately. Whether install, update, rollback, restore, and uninstall skip the restart is the host's decision:
only a plugin whose contributions are all debug adapters or grammars, and which has no backend, takes effect while Lithe runs. Language servers belong to the module graph
built at launch, so for an LSP plugin these file and version transitions finish at the next launch when prompted,
and an uninstall request deactivates a registered plugin first. Acceptance must cover the state before and after the restart.

When the checks are done, close the app, servers, and child processes you started for testing. Do not treat closing an editor tab as uninstalling the server,
and do not assume a crash always triggers an automatic retry or that every platform recovers the same way. When you find a lifecycle problem,
record the logs, host version, package version, architecture, and reproduction steps in the plugin repository; file host process-management problems with Lithe.

### 8. Rehearse publishing, updates, and rollback

Following [Publishing a community plugin](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/publishing.md), bump the plugin version, publish a full Release, and submit the first index entry.
The template's Release workflow checks that `v<version>` matches the manifest; for the original it is
`node scripts/package.mjs --expect-tag v0.3.1`. Use your own version after forking.

Every time you update the server or runtime, update the pinned inputs and licenses, rebuild the affected platform packages, and run the interactive and lifecycle checks again.
A published zip cannot be replaced, and a version cannot be reused; ship fixes as a new version. Adding a runnable program triggers a new
install confirmation; see [Version tracking and updates](https://github.com/Lithe-IDEA/Lithe-store/blob/main/docs/versioning-and-updates.md).

Before publishing, rehearse "old version → new version → back to the old logic" at least once:

- Keep the old version's assets and index records, so older hosts still have a compatible version to install.
- When the old and new servers use incompatible index formats, isolate the state or rebuild it safely; never let the old version read corrupted state.
- macOS rollback uses the previous version kept in the installation record and verifies that package again; for an LSP plugin the switch happens after restart.
  It offers no choice of arbitrary past versions, and it does not restore server indexes or user data. Rehearse this path for real.
- When the current version is broken, publish a higher fix version, restoring the last stable server in the new package if needed,
  and ask for the broken version to be withdrawn; do not rely on users' machines happening to keep the previous version.
- Verify uninstall and reinstall too. Document what happens to source, user build output, and custom caches, so user data is not deleted by mistake.

### 9. Troubleshoot by failure stage

| Symptom | Check first |
| --- | --- |
| Not visible in the marketplace, or not installable | Source URL, an asset for the current platform, whether the host supports the `pluginAPIVersion` |
| `plugin.archive.checksumMismatch` | Whether a Release asset was replaced; whether the index pins the final zip's digest |
| `plugin.manifest.invalid` | Unknown fields, schema/API confusion, IDs and references, API 2 fields in an API 1 manifest |
| `plugin.permission.undeclared` / `plugin.activation.invalid` | Whether providers, permissions, and events match one to one |
| `plugin.launch.missing` | Relative path, letter case, packaging completeness, and the executable bit |
| `plugin.launch.invalidRequest` | A launch uses `${logDir}` or `${workspaceStateDir}`, but the host did not supply that directory |
| `plugin.runtime.missing` | Runtime discovery, the minimum version, the Dock launch environment |
| A process runs but there is no completion | Whether logs pollute stdio, the LSP handshake, the document language identifier, upstream configuration |
| Refuses to load after one use | Whether the server wrote logs, indexes, locks, or generated files into the package |
| Processes remain after disabling | Whether the launcher uses `exec`, backgrounds itself, or leaves helper processes behind |
| Completion works but build or test does not | Whether a separate run, test, or build integration was actually declared and verified |

For the full field set, API levels, and stable error codes, go by Lithe's
[plugin contract](../contract/v1/README.md) and the
[manifest schema](../contract/v1/plugin-manifest.schema.json).
The contract copy is pinned to a commit of the Lithe repository (see [`contract.lock.json`](../contract.lock.json)), and the host status on this page is checked against the latest `preview` of that repository;
both may include capabilities not yet in a stable Lithe release, so check what your target host supports before publishing.
