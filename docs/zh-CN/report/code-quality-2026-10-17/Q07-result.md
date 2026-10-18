# Q07：SSE 字节预算与缓冲解析

## 状态与基线

- 状态：**本地实施与相关 service 回归完成，未合入**；未提交、推送、合并、创建 PR 或发布。
- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。
- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\quality-q07\Codex-Manager`。
- 修改前检查本会话附件为空，使用 `create_worktree` 指定上述 ref 创建并等待完成。根 `AGENTS.md` 已读取。原仓库总任务书只读。

## 复核与根因

1. 公共帧泵只有 128 帧 channel 上限，未完成行、未完成多行帧、排队帧均没有字节约束。
2. 每块从 pending 起点查找换行，再 drain 已解析前缀：长行反复扫描、多短行反复移动剩余数据。
3. 原生 Responses 额外使用 raw tee + EventSource sidecar，绕过公共帧泵；只修公共路径不足以限制这条解析路径。
4. Anthropic→Responses 把读取错误当作 EOF 完成，Gemini 在已有部分输出时可能把错误改成成功 STOP，Images 读取错误只写日志收集器。这些出口不能可靠向客户端报告预算失败。

## 改动

- 新增聚焦模块 `stream_readers/sse_frame_buffer.rs`，公共帧泵从 `common.rs` 移入。只扫描新输入，跨块保留未完成行，完成行移动为 String，无 drain 前缀和长行重扫。
- 单块输入也只扫描剩余单帧预算加 1 字节，超大 chunk 在扩展缓冲前被拒绝。兼容 LF、CR、CRLF 和跨块 UTF-8/CRLF；首行 BOM 仅在语义解析时忽略，原生输出保留原字节。
- 单帧线上字节上限 **128 MiB**，每帧最多 **65,536 行**；排队线上字节上限 **128 MiB**，同时保留 128 帧数量上限。生产者等待队列字节/帧许可后才继续解析和接收，向上游传递背压。
- 进程内所有生产帧泵共用 **512 MiB** 的缓冲存储记账预算。按 `2 × 线上字节 + 每行 (2 × size_of::<String>() + 8)` 保守计账，覆盖几何容量增长、行描述符和短行分配；当前 64 位平台每行 56 字节。未完成帧、排队帧和 reader 正在消费的最后一帧均持有许可。全局许可不足立即失败，避免多个未完成帧各持部分许可后互相等待。
- 超帧/行数、全局耗尽、非法 UTF-8 分别返回 `upstream_sse_frame_too_large`、`upstream_sse_buffer_exhausted`、`upstream_sse_invalid_utf8` 原因；先取消上游并释放失败帧，再等待向下游发送协议错误，不发送被截断的业务帧。
- 原生 Responses 也消费公共完整帧，同帧解析 usage 并原字节输出，取消后先停止网络输入，再排空已接收的完整帧收集 usage。保留原有 `response.failed` 规范化和终结去重。
- 公共帧泵记录每块字节到达时间。各 reader 判断空闲超时时读取这一时间，持续传入的大图片帧无需先完成才刷新活动。
- Anthropic→Responses、Gemini、Images 修正预算错误出口，防止静默 EOF 或伪成功终结；失败前 usage 和 partial output 保留，Anthropic→Responses 同时记录显式失败状态。Chat 工具索引逻辑没有改动；仅更新其空闲活动时间读取。
- CI 新增公共 reader、aggregate 帧解析与 `gateway_logs` 定向入口，包含新增 BOM 回归。

### 修改文件

- 新增帧泵与资源测试：`stream_readers/sse_frame_buffer.rs`、`sse_frame_buffer_tests.rs`、`common_protocol_tests.rs`。
- 公共入口与协议 reader：`common.rs`、`common_tests.rs`、`anthropic.rs`、`chat_completions.rs`、`gemini.rs`、`images.rs`、`openai_responses.rs`、`passthrough.rs`、`responses_from_anthropic.rs`、`responses_from_anthropic_tests.rs`，均位于 `crates/service/src/gateway/observability/http_bridge/stream_readers/`。
- 帧语义解析及 BOM 测试：`crates/service/src/gateway/observability/http_bridge/aggregate/sse_frame.rs`、`sse_frame_tests.rs`。
- 取消输入接口和测试专用 tee：`crates/service/src/gateway/upstream/response.rs`。
- CI 定向入口：`.github/workflows/ci.yml`；本报告为唯一新增的任务文档。

### 默认值依据与范围

现有图片测试只有短 base64 字段，没有真实大图片响应尺寸样本。128 MiB 是保守工程上限，可容纳约 96 MiB 二进制内容的 base64（需扣除 JSON/事件包络）；不是上游图片尺寸契约或生产实测结论。新增生成式 2 MiB、64 MiB 合法 base64 响应验证完整字段及尾部，无日志预览截断参与业务帧处理。没有新增环境变量或持久化设置。

512 MiB 约束的是公共 SSE 帧泵保留的存储记账，不是进程 RSS 上限；HTTP 客户端/`GatewayByteStream` 的底层 chunk channel、协议 JSON 树、累计输出文本/工具状态、下游输出缓冲及分配器开销不在这个预算内。极端并发下合法帧也可能收到明确的全局资源耗尽错误。

## 实际验证

所有命令均在上述 worktree 执行，Rust 使用本 worktree 默认 `target/` 且 `-j 2`。测试使用合成字节流、mock HTTP 上游、临时数据库及动态端口，不使用真实账号或生产数据库。

| 命令 | 实际结果 |
| --- | --- |
| `cargo test -p codexmanager-service --lib gateway::observability::http_bridge::stream_readers::common::tests -j 2 -- --test-threads=1` | 退出 0，但模块筛选路径错误，**0 测试**，不能作为验收通过证据。随后改用实际 `gateway::http_bridge` 路径。 |
| `cargo check -p codexmanager-service --lib -j 2` | 初次及最终检查均退出 0。最终日志为 `target/q07-check-final.log`；保留未改动文件 `gateway/mod.rs` 原有 unused imports 警告。 |
| `cargo test -p codexmanager-service --lib gateway::http_bridge::stream_readers -j 2 -- --test-threads=1`（第一次） | 退出 1，测试导入位置及移入子模块后的测试构造器可见性错误；已修正后重跑。 |
| 同上（第二次） | 81 项中 78 通过、3 失败；新字节活动时间覆盖测试 fixture 人工旧时间，多输出 keepalive。改为生产流提供 `Some(activity)`、仅接收测试帧的 fixture 提供 `None`，保留原断言，随后以 gateway 全范围重跑。 |
| `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --test-threads=1`（最终功能回归） | 退出 0，**937 passed，0 failed**，95.65s；日志 `target/q07-gateway-lib-final.log`。覆盖公共 pump/readers、aggregate、Claude 终结/超时 failover、非流式与 tools 等既有回归。 |
| `cargo test -p codexmanager-service --test gateway_logs -j 2 -- --test-threads=1` | 退出 0，**57 passed，0 failed**，168.36s；日志 `target/q07-gateway-http-final.log`。含 Claude 文本/tools 提前终结、Responses/Chat 流式与非流式、图片、超时及失败切换的 mock HTTP 回归。 |
| `cargo fmt --all -- --check` | 早期因本任务文件格式失败，修正后最终退出 0；日志 `target/q07-fmt-check.log`。 |
| `git diff --check` | 最终退出 0；仅工作区 LF→CRLF 提示。 |

最终功能回归后仅给已改为测试专用的 tee 所用 import 补充 `cfg(test)`，清理本次产生的非测试编译警告，并补齐 CI 的 aggregate 筛选；最终 `cargo check` 和格式检查在这两处调整后完成。

收尾核对：HEAD 仍为上述基线；变更仅为列出的 service/CI/本报告文件，无版本、前端、桌面、真实配置或原仓库任务书改动。没有本任务启动的持久服务进程，测试子进程已经退出。

### 资源预算行为证据

以下断言随公共帧泵测试执行，最终退出结果见上表：

- 长行逐字节输入与多短行输入中，`scanned_bytes` 等于输入总字节数。
- 对单个超大 chunk，扫描计数不超过当前剩余帧预算加 1；65,536 行上限限制短行描述符数量，实际 Vec/String 容量测试验证其不超过存储记账。
- UTF-8、CR/LF/CRLF 逐字节切分、CR-only 活连接终结、EOF 尾帧与 BOM 解析均有断言；原生 Responses 64 MiB base64 整帧逐字节相等，Responses/Chat/Images 三路 2 MiB base64 完整字段相等。
- 7 类 reader 的单帧超限及共享预算耗尽均报告协议错误，不输出伪成功终结；非法 UTF-8 在换行或 EOF 时报告错误并释放半帧。
- 小队列预算为单帧 18 字节时，只有 1 帧排队、1 帧等待字节许可，128 帧 channel 尚有 127 空位；证明实际触发的是字节背压。
- 两个并发泵共用 160 字节预算，32 字节未完成行计账 120 字节，只允许其中一个保留；另一个立即报告资源耗尽，最终所有许可归还，无互等。
- 超限在客户端读取 Error 前即触发上游取消、释放未完成帧；丢弃消费者同样释放队列及等待许可。
- 字节队列满和 128 帧 channel 满时，`stop_input` 都能先确认取消，再完整有序排空已接收帧；已消费活动帧在下一次接收时归还许可。

## 未验证项与整合风险

- 未执行真实上游账号/真实图片生成；合成 base64 只证明协议完整性与缓冲行为，不验证图像解码或提供商尺寸保证。
- 以下为未执行命令；本任务仅隔离修改 service SSE 链路，按任务约定采用相关 service 回归，前端、桌面、Web shell 无改动。局部通过不能替代最终组合版本验收，Tauri 也被根 workspace 排除：
  - `cargo test --workspace -j 2`：留待组合版本执行全 workspace 验收。
  - `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2`：本任务未改桌面 Rust。
  - `pnpm -C apps run build`、`pnpm -C apps run build:desktop`、`pnpm -C apps run test:runtime`：本任务未改前端或静态导出。
  - `cargo test -p codexmanager-web -j 2`：本任务未改 Web shell/transport；组合版本另验。
- Q02 与 Q07 同时涉及 `stream_readers/chat_completions.rs`。本任务只新增字节活动时间读取，不修改工具索引；应先整合 Q02，再整合 Q07，重跑公共 reader、HTTP bridge、upstream、gateway mock HTTP 回归。
- 保留最新 Claude 流终结、错误规范化与超时 failover；其最终实际回归结果见验证表。不得把独立工作区通过解释为已合入或组合版本通过。
- 本工作区保持可审查的未提交改动，不创建 PR、不修改版本号、不修改原仓库总文档。
