# 插件接入统一日志

新版本插件使用 `litheLogging` 1 与 `lithe/log` 发送受限的操作事实，由宿主的共享事件目录、脱敏、队列和文件写入器落盘。三个 SDK 不再提供自由文本 `log` 接口，也没有 stderr 回退。插件无需实现 v2 文件格式、文件轮转、脱敏器或后台写入线程。

宿主必须在 initialize 中提供 `capabilities.experimental.litheLogging.versions: [1]`，SDK 回复 `capabilities.experimental.litheLogging.version: 1`。缺少协议时，新 SDK 拒绝初始化；不自动退回旧日志。初始化尚未协商时的失败只通过原有 RPC 错误与宿主进程生命周期记录，不能提前发送未协商的日志通知。

本文描述源码中的新 API。插件本轮将源码与真实哈希一起固定在可复现的构建输入中；正式发布 SDK 后也可以改为锁定真实 release tag 与 SHA-256。不要引用不存在的发布版本或开发机的绝对 path 依赖。

## 每种事实与级别

| `kind` | v2 目录事件 | 级别与用途 |
| --- | --- | --- |
| `operationFinished` | `plugins.operation.finished` | INFO，关键用户操作的真实成功终态 |
| `operationFailed` | `plugins.operation.failed` | ERROR，影响请求结果的失败；SDK 在拥有结果的边界发送一次 |
| `degraded` | `plugins.operation.fallback_selected` | WARN，已处理并选择回退的失败 |
| `recovered` | `plugins.operation.recovery_finished` | WARN，曾失败后的真实恢复 |

`operationFinished` 和 `operationFailed` 必须有单调时钟计算的 `durationMs`。失败、降级、恢复使用受控的 `code`；成功没有 `code`。可选 `count` 与 `durationMs` 都是 0…9007199254740991 的整数。API 不接受原始 message、stack、request、URL、SQL、命令输出或字段字典。

`operation` 只接受本 SDK 已注册的函数 ID 或封闭协议操作，超过 64 字节的已注册函数 ID 统一为 `function-call`。公开的协议操作包括 initialize/shutdown/notification/function-call/result-encoding、SDK 已知的 `lithe/build/*` 与 `lithe/vcs/*` 方法，以及快捷键导入请求 `lithe/keymap/import`（三种 SDK 自动上报其终态）；任意用户文本即使字符合法也不接受。`code` 是最长 64 字节的 ASCII 类别标识，不是错误正文。

通道由宿主从已校验的安装插件 ID 得到 `plugins/<plugin>/host`，插件不能更改通道或拼接路径。上下文诊断优先携带宿主原始 `operationID`，缺少合法值时沿用当前请求原始 JSON-RPC ID（安全字符串或整数的十进制字符串），wire 名为 `operationId`，只保留最长 128 字节的受控 ASCII 标识；无真实身份时省略。通知没有请求 ID，不生成回退身份。后台共享任务可以使用 backend/handle 的无 op 诊断，不能伪造一个操作 ID。

## 三个 SDK 的用法

```js
backend.function('project.refresh', async (params, context) => {
  // 只在决定继续执行的真实降级分支调用，不把异常正文传进日志。
  context.diagnostics.degraded('project.refresh', 'cache_unavailable', 1);
  return refreshWithoutCache(params);
});
// 高频读取或轮询不产生 INFO 成功噪声，失败仍自动记录。
backend.function('project.inspect', (params) => inspectProject(params));
backend.suppressSuccessDiagnostics('project.inspect');
backend.diagnostics.recovered('project.refresh', 'cache_restored');
```

```java
context.diagnostics().degraded("lithe/build/model", "snapshot_unavailable", 1L);
backend.diagnostics().recovered("lithe/build/model", "snapshot_restored");
backend.suppressSuccessDiagnostics("lithe/build/dependencies");
```

```rust
context.diagnostics().degraded("query.run", "rows_truncated", Some(1))?;
handle.diagnostics().recovered("query.run", "connection_restored", None)?;
backend.suppress_success_diagnostics("query.inspect")?;
```

`degraded`、`recovered` 的 operation 必须已注册，静态示例需要放到实际的注册完成之后。Rust 的返回错误只表达调用参数非法；协议写入复用现有 writer，诊断传输失败不会把已经成功的用户操作改成失败。没有额外 writer 或后台队列。

混合用户操作和高频读取的单个入口可以在当前调用上使用 `context.suppressSuccessDiagnostics()`（Rust 为 `context.suppress_success_diagnostics()`）；该决定只影响本次成功事实，不影响其他调用与失败。

SDK 在函数与受管 build/VCS 请求的结果边界自动发送一次终态，通知在 RPC 响应之前发送，宿主才能关联仍活动的请求。不要在处理函数里先发送失败再返回或抛出同一次失败。取消不产生失败事件，正常缺失的收尾对象不应产生 WARN。高频只读、预览、统计与轮询通过 `suppressSuccessDiagnostics` / `suppress_success_diagnostics` 关闭成功事实；失败与真正降级仍保留。

Rust SDK 用一次安装的 panic hook 包装器，仅在线程处于 SDK 拥有的 catch 边界时屏蔽原始 payload 输出，随后发送固定 `panic` 类别。其他线程和边界外的 panic 继续执行原 hook。不要把用户内容放进 panic，插件也不要在 SDK 启动后替换该 hook 来恢复正文输出。

## 上游诊断与文件位置

上游语言服务器的 stderr、`window/logMessage` 继续由宿主接收；它们不是插件自己的结构化日志入口。插件不得把源码、协议载荷、SQL、HTTP body 或构建输出复制进上游诊断，以为脱敏与截断能使其安全。控制台输出走已有 Build、Run、Test、VCS 或插件界面结果。

macOS 的日志窗口与导出读取 v2。Windows 的持久化仍是 v1，新的 Core 接收与目录事件不等于 Windows 文件窗口已经迁移；只有在对应目标平台执行工作流后才能声明该平台已验证。

只有上游工具确实只能写文件时，才使用 `${logDir}`。宿主显式提供当前会话的 `plugins/<id>/files/`，通过可写存储适配器创建，不能从插件安装路径或 app bundle 推导：

- 日志目录只属于当前会话，不保存数据库、索引、下载或工作区状态，不跨工作树共享。
- 缓存和状态继续使用 `${stateDir}`、`${workspaceStateDir}`。
- 日志存储不可用时，宿主不提供该变量。远程进程不能使用本机日志目录；引用 `${logDir}` 却没有绝对 `logDirectory` 的启动计划会失败。
- 不需要原生文件的插件不要增加这个启动依赖。先核对目标宿主和平台确实支持，再修改启动模板。
- 原生文件不自动变成 v2 时间线事件，不能再把整个文件打印进 stderr。
- app bundle、插件包、Windows 安装目录运行时只读；日志与锁文件不得改变签名或 Sparkle 增量更新基线。

## 最小验证

1. 用已有有界协议测试确认 logging v1 协商、四类事实、级别、原始操作关联、成功抑制、取消和非法参数拒绝。
2. 触发真实失败，确认原有用户错误码与提示保持不变，诊断只有固定类别，不含请求正文、原始异常或堆栈。
3. 使用真实 Core 查询 `diagnostics.drainCoreEvents`，核对目录事件与 `op`；不要把 v1 poll 中的旧 detail 当作新结构化记录。
4. 在目标宿主中安装并执行对应工作流，核对日志窗口。SDK 组件通过不能替代产品验收。
5. 原生日志位置有变化时比较工作流前后安装目录清单或哈希，确认发行资源不变；结束后清理本轮进程。
