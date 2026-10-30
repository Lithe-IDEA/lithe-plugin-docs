# 原生构建、测试与运行准备（开发中）

插件可以只提供工作区后端，不创建网页。用户从原生工具窗口执行构建、发现与运行测试、准备应用启动；宿主继续管理工作区、取消、输出和应用进程。

本文对应尚未发布的 Node/Java SDK 源码，以及 Lithe #1495 的宿主改动。发布的 SDK 0.1.0 资产不包含这些新入口。宿主改动合并后，按仓库规则同步 `contract/v1/`，再一起发布、锁定新 SDK；不要手改契约副本或向旧版本宿主隐瞒 `requires.buildProviders`。

## 选择入口

| 工作 | Node SDK | Java SDK | 最低 `buildProviders` / 进程协议 |
| --- | --- | --- | --- |
| 执行有界任务 | `backend.build.execute` | `onExecute` | 2 |
| 运行精确测试 | `backend.build.test` | `onTest` | 3 |
| 发现可运行的用例 | `backend.build.discoverTests` | `onDiscoverTests` | 3 |
| 准备应用启动 | `backend.build.prepareRun` | `onPrepareRun` | 4 |
| 准备原生程序调试 | `backend.build.prepareDebug` | `onPrepareDebug` | 5 |
| 读取原生配置 | `backend.build.configuration` | `onConfiguration` | 7 |

安装时检查清单的能力版本，启动时 SDK 从宿主提供的版本中选最高的共同版本。没有对应协议时，新方法在进入处理器之前被拒绝；不会改发私有函数或展开成另一条宿主命令。Node 用 `backend.buildVersion`、Java 用 `backend.negotiated().buildVersion()` 查看结果。

处理器必须在 `listen` 前注册。模型、依赖和测试发现回复由 SDK 补齐对应 `format` 和 `version: 1`；执行回复是 `{ exitCode, output }`，启动准备回复是完整的可移植计划。Core 继续检查选择器、工作区路径和大小限制。

## 所有权与清理

插件从其选定的上游引擎读取当前工程、测试和运行事实。发现结果中的每个用例都必须能通过同一套件、用例、工程与目标选择器精确执行；处理器重新核对选择，不把失效用例当成“运行全部”。不要在宿主或 SDK 里再扫描 Java 注解、拼 classpath 或解释 Maven 模型。

构建与测试处理器使用调用上下文的 `run` 启动工具，取消或调用结束会结束它拥有的工具树。工具正常返回非零码仍是执行结果；异常属于协议/准备失败。不要启动脱离调用的进程，或用一条私有函数代替这些领域入口。

运行准备先等待上游同步和必要构建，再返回 `executable`、原始 `arguments` 数组、工作区相对 `workingDirectory` 和 `environment`。`executable.kind` 只能是 `command`（宿主解析的裸命令名）或 `toolchain`（宿主工具链标识）。处理器不启动长期运行的应用；宿主校验后启动、停止并显示应用输出。构建失败、取消或旧结果失效时不能启动应用。

执行/测试请求的后端拒绝由 Core 报告为 `plugin.build.failed`。运行准备的 RPC 错误保留通用 LSP `serverError` 分类，不是带退出码的执行结果，也不使用私有函数的 `pluginErrorCode`。宿主在两种失败下都不能启动应用或改走旧命令。

执行、测试、运行与调试准备上下文携带 Core 分配的 `operationID`，进度自动带上它。Node 使用 `context.progress({ message, percentage, output })`；Java 使用 `context.progress(message, percentage, output)`。消息上限 4096 UTF-8 字节，单次输出上限 64 KiB，百分比限制在 0–100；取消或结束后的报告被丢弃。模型、依赖和发现不提供执行进度。

状态写入宿主提供的可写目录；安装后的插件包只读。`prepareRun` 的结果不能变成跨重启缓存的 classpath 或命令：宿主保存入口选择器，重启重新准备。

## 可运行示例

两套示例使用相同的虚构构建引擎，工程事实来自工作区的 `lithe-sample.json`。这是协议教学用的引擎，不是 Maven/Java 迁移成果。

- [Node 示例](../sdk/node/examples/sample-native-build/backend/main.mjs)及其[清单](../sdk/node/examples/sample-native-build/plugin.json)。开发时从源码引入 SDK；分发时将 `src/` 放入 `backend/vendor/`，把导入改为 `./vendor/index.mjs`。端到端测试按这种结构打包验证。
- [Java 示例](../sdk/java/examples/sample-native-build/src/main/java/org/example/samplenativebuild/SampleNativeBuild.java)及其[清单](../sdk/java/examples/sample-native-build/plugin.json)。运行 `mvn -B -ntp -f sdk/java/pom.xml package`，将 `sample-native-build/target/sample-native-build.jar` 放到包内 `backend/`。SDK 已 shade 进同一 JAR。

示例工程包含构建命令、精确用例及应用参数，例如：

```json
{
  "projects": [{
    "name": "app", "path": "app",
    "build": ["node", "-e", "console.log('built')"],
    "tests": [{ "suite": "Suite", "case": "exact case",
      "command": ["node", "-e", "console.log('passed')", "--"] }],
    "run": { "command": "node",
      "arguments": ["-e", "console.log(process.argv.slice(1))", "--"] }
  }]
}
```

模型的路径相对构建实例，后续请求的工程/目标选择器相对整个工作区；示例显式转换这两种表示，不把嵌套实例当成根工程。

`app` 目录需存在，测试工具需可执行。示例重新读取配置、核对工程/目标和测试，并解析真实路径以拒绝工作区外目录；实际插件应调用其上游工程引擎，而非复制这份虚构引擎。

## 验证

```sh
node --test sdk/node/test/*.test.mjs
npm ci --prefix sdk/node --ignore-scripts
npm run --prefix sdk/node test:types
mvn -B -ntp -f sdk/java/pom.xml verify
node --test scripts/*.test.mjs

# 锁定契约对应的兼容场景。
node scripts/sdk-e2e.mjs

# 宿主分支已提交，但尚未合并时。
node scripts/sdk-e2e.mjs --source /path/to/Lithe-IDEA --ref branch --build-workflows

# 显式使用尚未提交的宿主源码；不可和 --ref 混用。
node scripts/sdk-e2e.mjs --source /path/to/Lithe-IDEA --working-tree --build-workflows
```

`--working-tree` 只把 `rust/`、`shared/` 的暂存、未暂存和未跟踪源码复制到一次性 Core 检出，不修改源仓库，也不复制忽略的构建缓存。该模式报告基线 commit 加工作区改动，不能把输出里的基线 SHA 当成完整已提交源码。

`--build-workflows` 显式增加两套新协议场景，供合并前验证；契约镜像同步到包含 `buildProviders: 5` 后，默认验证也自动包含它们。每周的 preview 检查显式运行新场景。新的测试通过真实 Core 规划与解码，覆盖构建非零退出码、精确筛选、运行前构建、准备计划、进度、取消后的工具清理、并行工作区隔离，以及包含 SDK/JAR 的完整包文件不变。它不代替原生界面、真实 Maven/JDT 工程或 Windows 实机验证。

若复用预构建 Java 示例，分别设置 `LITHE_PLUGIN_SDK_JAVA_SAMPLE` 与 `LITHE_PLUGIN_SDK_JAVA_NATIVE_SAMPLE`。编译使用 JDK 21，产物面向 Java 8；可用 `LITHE_PLUGIN_SDK_JAVA_RUNTIME` 指定实际运行时。

调试适配器、长期服务和跨插件协作有各自的宿主契约；这些处理器不开放跨包私有函数或反向 `lithe/*` 调用。


## 原生程序调试准备

运行声明加 `debug: {"adapter":"native"}`，同一插件提供 `native` 适配器；manifest 要求 `buildProviders: 5` 与 `debugAdapters: 2`。适配器声明 launch/executable，使用 program、cwd、args、env 的目标绑定。不要用跨包 ID，也不要在后端返回 DAP 进程、端口或任意调试参数。

`prepareDebug` 先完成构建，再返回 `{program, arguments, workingDirectory, environment}`。程序是当前实例内的已有文件，路径从工作区起算；宿主校验路径和可执行权限，再复用现有 DebugAdapter Registry。取消、错误和过期结果都不能启动调试。Node 和 Java 的 sample-native-build 从虚构引擎的 `run.debugProgram` 读取程序，参数使用当前请求的 userArguments；示例的 `example-dap` 是占位适配器命令，真实插件应打包符合其许可的成熟适配器。

端到端示例验证两个 SDK 的真实进程、运行前构建、原生调试准备前构建、Core 路径解码、进度及失败不返回目标。它不启动真实调试器，也不代表 Java/JDT、跨插件协作或 Windows 已完成。

## 原生配置读取与确认保存（协议 7，开发中）

构建系统声明 `configuration: {"source":"provider"}`，要求 `buildProviders: 7`，并使用自己的工作区 Provider 进程。宿主以同一组工具选择、选项和环境发送 `lithe/build/configuration`；这是读取，不传编辑值或完整原生文件。Node 注册 `backend.build.configuration(handler)`，Java 注册 `onConfiguration(handler)`。SDK 补齐 `format: "litheBuildConfiguration"`、`version: 1`；旧协商版本在进入处理器前拒绝。

处理器从选中的成熟上游获得实际来源与生效值，返回 `sources`。每个来源包含 `id`、`scope`（installation/user/workspace）、文件 URI、原始字节的 SHA-256 revision（缺失文件为 missing）、writable、XML 根名称与命名空间，以及非敏感字段。字段区分 value（原值，未设置为 null）、effectiveValue 和 effectiveSource，并声明 path（根以下的直接 XML 元素路径）；stringList 还声明 itemElement。参考契约合并并同步后的 `contract/v1/fixtures/build-configuration.json`。当前镜像在宿主合并前保持原样，不能把待合并字段写进旧契约副本。

例如 Maven 插件观察其所选 Maven 的原始 `SettingsBuildingRequest` 与 `SettingsBuildingResult`，在 reactor 导入前取得源文件与合并结果；会话执行请求提供最终命令行覆盖值。不要在宿主、SDK 或插件里重写 Maven 配置优先级，不要把会话 SettingsAdapter 当成未覆盖的原始设置。配置探针需在返回前确认实际来源和哈希仍匹配，文件更改时明确失败。

宿主在通用设置页显示来源、原值、继承/覆盖和生效值。输入只改草稿；保存明确列出的目标来源后，平台适配器以原始快照进行条件替换，再调用同一 Provider 读取实际生效值。取消、打开、失焦和离页不写配置。安装来源始终只读；多个作用域选择同一文件时，只能显示完全一致的只读别名。保存失效或冲突保留草稿，不把提交值当成生效值。

完整 XML 可能含凭据，只能在宿主文件 adapter 与 Core 的纯编辑命令间流动。不要通过配置回复、日志、插件函数、网页或模型共享传整份文件。宿主仅编辑获准的非敏感节点并保留其他内容、注释和权限；Provider 只读真实事实，不接收或代写宿主确认过的原生文档。探针目录、输出和带源哈希的模型缓存属于隔离的工作区状态，调用结束清理，不能写包或跨工作树复用。

Maven 的真实集成场景覆盖损坏 POM 前读取、用户/安装来源、自定义路径、命令行覆盖、敏感内容保留、Core 编辑后回读、旧版本拒绝、清理与包清单不变。SDK 单元测试分别验证 1–7 协商与配置格式；宿主合并前用 `node scripts/sdk-e2e.mjs --source <Lithe工作树> --working-tree --build-workflows --native-configuration` 验证 Node/Java 的端到端示例及协议 7 配置投影、Core 内部纯 XML 编辑、敏感内容保留与读取无写入。源码通过不表示已发版；Java SDK/JAR 的正式版本与哈希仍需发布流程固定。

`--native-configuration` 在宿主合并前显式启用协议 7 场景；契约镜像同步到包含 7 的 preview 提交后默认启用。Node/Java 的无页面样例仍以其虚构引擎 JSON 供应非敏感配置投影，不增加 XML 解析或凭据传输。

## Maven framework model properties

Maven's backend publishes `run.modelVersion: "1"` on each canonical model
project. For non-aggregator projects it may also publish `run.provider` with
`spring-boot.maven`, `quarkus.maven`, or `micronaut.maven`. The selection comes
from Maven's effective applied build plugins, including active Profiles and
inheritance; plugin management alone is not application. These are scalar model
properties, not a new SDK RPC or launch recipe. The host forwards the current
project name, reactor and selection to Run generation; Core validates and
projects existing service identities. A host requiring this model rejects an
older Maven plugin without the version property and asks for an update rather
than parsing POMs itself. Existing framework launch options remain in their
current Run configuration layer until launch planning is migrated.

The optional scalar project property `java.minimumVersion` contains a normalized
Java feature version from the configured compiler execution model. Maven's
backend calculates its own lifecycle plan without executing goals, evaluates
compiler parameters with Maven's evaluator, and accounts for main/test and
custom executions. Missing means unknown (including an unavailable descriptor
or unresolved expression). Hosts must not fill the gap by parsing POM text.
The highest known floor applies to project toolchain selection; platform Java
version preferences must not lower it.
