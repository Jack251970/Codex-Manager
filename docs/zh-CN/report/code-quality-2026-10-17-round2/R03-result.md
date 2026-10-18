# R03：工具索引与 SSE 组合验收

组合增量和本轮发现的终结误判修复已完成，保留本地可审查修改。最终完整 gateway 回归 956/956、HTTP mock 回归 57/57、官方 SDK 14/14 fixture 通过。此前完整回归曾出现两项非确定性测试失败；加入不改变行为的阶段诊断后未复现，失败时根因尚未确证，历史失败及验收风险仍保留。R04 非流式 Chat 工具修复及全项目统一验收不在本次完成范围。

## 隔离、基线与冻结输入

- 工作区：`C:\Users\qxnm\.codex\worktrees\round2-tools-sse\Codex-Manager`。
- Git 基线：`f14bd735dca9203d7430958480b82ba7713641a9`。创建前 `list_artifacts` 为空；以 `create_worktree(ref=<基线>, name="round2-tools-sse", allowAsync=true)` 创建，operation `2ecf08c0-73d9-4be4-84aa-f73f8088474f` 返回 completed 后才编辑；初始 HEAD 正确、状态干净，已读根 AGENTS.md。
- 原仓库两份任务书仅按指定绝对路径读取，未改原仓库、第一轮来源工作树或冻结快照；没有读取其他任务不断变化的工作树。
- 输入根：`C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round2-3e2657e8`；已读取 Q02/Q07 的 manifest 与第一轮报告，逐个验证 patch、全部 tracked/untracked 文件 SHA256 及基线。无删除项。

| 输入 | manifest.json SHA256 | tracked.patch SHA256 | 完整文件校验 |
| --- | --- | --- | --- |
| Q02 | `A9B719E0AF7EFF325A0935B47AD68AB13BB970D217B39B8002D7E03B6326F634` | `6732306B3A93644AA3823532CFA1BFB104CE04D3D7557779226B9A4ECDB8A601` | 2 tracked + 3 untracked 均匹配 |
| Q07 | `7A22A1DC887708978A331D1EC5556F91DCEC9D1DC560AC833C6F8C8D4764F943` | `96F683CF3B14B45636B2C6B8335F714A1AA4048D8037CDC050B492F105A76A0A` | 14 tracked + 4 untracked 均匹配 |

## 导入与冲突处理

1. Q02 `git apply --check <Q02/tracked.patch>` 与 `git apply` 均退出 0；复制 manifest 全部 3 个 untracked 文件。
2. Q07 整体 `git apply --check <Q07/tracked.patch>` 退出 1，仅 CI 同位置新增步骤冲突，日志 `target/r03-evidence/q07-initial-check.log`。读取 `git show f14bd735:.github/workflows/ci.yml`、当前 Q02 版本和 Q07 `tracked/.github/workflows/ci.yml`，逐段保留双方新增步骤。
3. Q07 使用 `git apply --check --exclude=.github/workflows/ci.yml <Q07/tracked.patch>` 后 apply，均退出 0；复制 manifest 全部 4 个 untracked 文件。Chat reader 的 5 行活动时间改动在 Q02 完整映射上按上下文应用，没有用 Q07 整文件覆盖 Q02。
4. 保留 Q02 工具首次出现时分配连续序号、上游 output_index 身份映射、ID/name 延迟恢复和参数后缀补发；保留 Q07 帧/队列/共享预算、活动时间、上游取消和显式协议错误出口。
5. CI 同时保留工具索引定向检查、公共 reader/aggregate/gateway_logs 检查；工具步骤导出实际 reader SSE，新增官方 Python SDK 消费步骤。未更改发布工作流或版本。
6. 导入的 `code-quality-2026-10-17/Q02-result.md`、`Q07-result.md` 原样保留，仍是第一轮来源记录，不能替代本报告的组合验证。

## 本轮新增内容

- Chat 组合测试：7 字节分块、跨 UTF-8/CRLF、超过 64 KiB 的参数、reasoning/message/其他输出夹杂、三个工具交错及 done/completed 元数据和参数补发。实际 reader 输出继续进入 SDK fixture 导出。
- 部分工具输出后分别触发单帧超限与共享预算耗尽：检查唯一显式错误、无成功 finish_reason/[DONE]、部分 ID/name/arguments 保留，以及有限期限内许可全部归还。
- 图片协议测试新增可解码的 1×1 RGBA PNG base64，分别验证 Responses/Chat/Images 的完整字段；原有约 2 MiB、64 MiB 合成 base64 大帧断言继续执行。合成大帧不等同于真实图片尺寸或提供商保证。
- 组合审查发现并实际复现基线仍存在的终结误判：普通 delta 文本中的 `response.completed` 等子串会被当成已送达终结，随后真实帧超限错误被吞掉。修复公共检测为有界控制行识别，跨 read 保留前缀及长行截断状态；native reader 复用已有顶层 JSON 事件类型判断，避免重复解析大图片。cleanup 只收 usage，不能把未送达的排队终结算作已交付。
- 新 native 回归覆盖 5 个终结词 × 3 种帧形式（普通事件/data-only/头与 JSON 不一致），并保留嵌套 `type`；检查每次真实帧超限均输出一个 response.failed 和完整预算原因、释放全部许可。公共检测新增小块/长行/无空格控制字段回归。
- 第二次独立复审继续发现 native 未解析帧回退仍会把空终结头或损坏 JSON 头算作交付；新增真实超限 red 测试。修复回退为零复制检查整个 frame 的 data 载荷：必须恰好有一个非空值且严格为 `[DONE]`；空头、坏 JSON、重复、混合、大小写错误都不能证明交付。独立 `[DONE]`、BOM/CRLF、空 data 行及 `[DONE]` 混其他 data 的正反例均补入，正常大图片仍仅用一次 JSON 语义解析。

## 验证记录

所有命令在本工作区执行，Rust `-j 2`、独立 `target/`，未共用其他 worktree 的可写输出。`TEMP`/`TMP` 指向本工作区 `target/r03-runtime-tmp`。测试仅使用合成输入、临时数据库、动态本地端口与 mock 上游。没有操作真实账号、生产服务或真实数据库。

以下同时保留修复前 red、完整回归失败及最终通过记录，不以定向通过替换历史失败证据。

| 命令 | 退出码 / 结果 | 日志 |
| --- | --- | --- |
| `cargo test -p codexmanager-service --lib gateway::http_bridge::stream_readers::common -j 2 -- --test-threads=1` | 101，新增组合测试引用模块路径错误 E0433，尚未运行用例；改为已有 common::SseFrameLimits re-export | `target/r03-evidence/common-initial.log` |
| Python `-B -m pip install --disable-pip-version-check --no-cache-dir --no-compile --target target/r03-sdk-python openai==2.29.0` | 0；openai 2.29.0 / httpx 0.28.1，仅安装到本工作区 target | `target/r03-evidence/sdk-install.log` |
| `cargo test -p codexmanager-service --lib native_delta_terminal_words_do_not_hide_a_later_frame_budget_error -j 2 -- --test-threads=1`（修复前） | 101；0 passed / 1 failed / 0 ignored；普通正文 response.completed 导致真实超限 response.failed 数为 0 | `target/r03-evidence/native-terminal-red.log` |
| `cargo test -p codexmanager-service --lib terminal_delivery_ -j 2 -- --test-threads=1`（修复前） | 101；3 passed / 2 failed / 0 ignored；普通正文与长行尾部误标 | `target/r03-evidence/common-terminal-red.log` |
| `target/debug/deps/codexmanager_service-f7b3088b64652c11.exe tool_indices --test-threads=1`（修复前二进制定向诊断） | 0；10 passed / 0 failed / 0 ignored；导出 14 个实际 reader fixture | `target/r03-evidence/tools-diagnostic.log` |
| Python `-B scripts/chat-tool-indices-sdk-check.py target/r03-sdk-diagnostic`（同批诊断） | 0；14/14 个 fixture 全部恢复精确 ID/name/arguments/JSON/finish_reason，mock HTTP | `target/r03-evidence/sdk-diagnostic.log` |
| Python `-B target/r03-evidence/verify-png.py` | 0；1/1 PNG 签名、全部 CRC、元数据和红色像素解码校验 | `target/r03-evidence/png-validity.log` |
| `cargo test -p codexmanager-service --lib native_delta_terminal_words_do_not_hide_a_later_frame_budget_error -j 2 -- --test-threads=1`（正文误判修复后） | 0；1 passed / 0 failed / 0 ignored，含15场景；此时尚未修复空/坏JSON头回退 | `target/r03-evidence/native-terminal-green.log` |
| `cargo test -p codexmanager-service --lib gateway::http_bridge::stream_readers::common::tests -j 2 -- --test-threads=1` | 0；13 passed / 0 failed / 0 ignored | `target/r03-evidence/common-terminal-green.log` |
| `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --test-threads=1`（中间版本） | 0；952 passed / 0 failed / 0 ignored；98.58s，尚未包含最后空/坏JSON头回退修复，不当作最终通过 | `target/r03-evidence/gateway-lib-before-control-fix.log` |
| `cargo test -p codexmanager-service --lib terminal_header_does_not_hide_a_later_frame_budget_error -j 2 -- --test-threads=1`（最后回退修复前） | 101；0 passed / 2 failed / 0 ignored；空头和坏JSON均吞掉真实超限response.failed | `target/r03-evidence/native-control-red.log` |
| `cargo test -p codexmanager-service --lib stream_readers::openai_responses -j 2 -- --test-threads=1`（最终代码） | 0；16 passed / 0 failed / 0 ignored；0.68s，包含全部控制帧修复和64MiB base64 | `target/r03-evidence/native-control-green.log` |
| Python `-B scripts/chat-tool-indices-sdk-check.py target/r03-sdk-accepted`（最终组合 reader 导出） | 0；14/14 fixture，openai 2.29.0 / httpx 0.28.1；包括原有13份和新增大参数分块场景，精确验证ID/name/arguments/JSON/finish_reason | `target/r03-evidence/sdk-final.log` |
| `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --test-threads=1`（控制帧最终修复后，fixture诊断前） | 101；954 passed / 2 failed / 0 ignored；171.87s。失败：`chatgpt_cloudflare_challenge_directly_failovers_without_same_account_retry`、`cancelled_header_wait_logs_one_499_without_retrying_or_failing_over`；不能记全范围通过 | `target/r03-evidence/gateway-lib-final.log` |
| 同一组合测试二进制对上述两项分别 `--exact --test-threads=1`，固定3次各独立进程 | Cloudflare：退出码101/0/101（2/3失败，mock recv 2s内无请求）；取消：0/0/0。没有改断言/超时，定向通过不替代全范围失败 | `target/r03-evidence/failure-isolation-summary.json`，6份`failure-isolation-<轮>-<项>.log` |
| `cargo test -p codexmanager-service --test gateway_logs -j 2 -- --test-threads=1`（最终产品代码） | 0；57 passed / 0 failed / 0 ignored；197.48s | `target/r03-evidence/gateway-http-final.log` |
| `cargo test -p codexmanager-service --lib chatgpt_cloudflare_challenge_directly_failovers_without_same_account_retry -j 2 -- --test-threads=1 --nocapture`（仅加计时） | 0；1 passed / 0 failed / 0 ignored；client构造0ms，mock收到请求8ms，发送/运行时9ms，实际status403 | `target/r03-evidence/failure-phase-postprocess.log` |
| `cargo test -p codexmanager-service --lib cancelled_header_wait_logs_one_499_without_retrying_or_failing_over -j 2 -- --test-threads=1 --nocapture`（仅加阶段） | 0；1 passed / 0 failed / 0 ignored；accept/read/cancel10ms，outcome/EOF38ms，禁止额外连接断言143ms完成 | `target/r03-evidence/failure-phase-cancel.log` |
| 阶段诊断二进制对两项 `--exact --test-threads=1 --nocapture`，另固定3轮 | 0；6次各1 passed / 0 failed / 0 ignored。Cloudflare实际mock403，mock等待5–858ms；取消每次看到outcome+EOF，最后阶段127–141ms。没有无限重试 | `target/r03-evidence/phase-isolation-summary.json`，6份`phase-isolation-<轮>-<项>.log` |
| `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --test-threads=1 --nocapture`（最终代码及阶段诊断，最后一轮） | 0；956 passed / 0 failed / 0 ignored；102.02s。Cloudflare mock收请求3ms、发送/运行时5ms、status403；取消13ms发信号、17ms观察outcome和EOF、128ms完成禁止额外连接断言 | `target/r03-evidence/gateway-lib-with-phases.log` |
| Python `-B scripts/chat-tool-indices-sdk-check.py target/r03-sdk-delivery`（最后一轮实际 reader 导出） | 0；14/14 fixture；官方SDK恢复全部精确ID/name/arguments及JSON，HTTP完全mock | `target/r03-evidence/sdk-delivery.log` |
| `cargo fmt --all -- --check`、`git diff --check` | 均0；格式及差异检查通过 | `target/r03-evidence/fmt-final.log`、`diff-check-final.log` |

Python 实际路径：`C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe`。SDK 命令设置 `PYTHONPATH=<本工作区>\target\r03-sdk-python`、`PYTHONDONTWRITEBYTECODE=1`。导出环境变量 `CM_REPAIR_FIXTURE_DIR` 为日志批次对应的绝对目录。

两项全范围失败的源码与调用链已核对：均处在发送/等响应头阶段，尚未进入SSE reader；相关生产路径与fixture在导入版本中无语义修改。Cloudflare失败是mock `recv_timeout(2s)` 为None，非Failover业务断言；取消失败是包围整个流程的5s超时，原日志未定位阶段。独立进程总时延最高14秒，主机Windows系统代理启用（只记录布尔，未打印代理值），HTTP(S)/ALL/NO_PROXY及CODEX代理环境变量未配置。准确依赖reqwest 0.12.28/hyper-util 0.1.19会读Windows系统代理；但现有证据不能断言当时实际走了代理、必然是冷初始化或干净基线也失败。

随后仅给这两个测试加阶段计时，不改变产品、顺序、等待或断言。一次定向+固定3轮均未复现：Cloudflare确收到mock403；取消在发送取消信号后观察到Handled/499与TCP EOF。增加诊断后的最后一轮完整 gateway 956/956 通过，同样收到了阶段证据，之后未继续重跑。**原失败根因未被失败时阶段数据确证，仍作为非确定性验收风险保留**；最后一轮通过不能改写之前的954/2记录，也不能证明不稳定性已消除。保留2s/5s/100ms、次数、499/Failover全部原保证；不以no_proxy/预热/扩大时限掩盖。

## 额外增量文件

相对完整 Q02→Q07 导入，本轮新增语义集中于以下文件（不是另外导入 R04）：

| 文件 | 本轮增量 |
| --- | --- |
| `.github/workflows/ci.yml` | 手工合并双方检查，SDK导出及官方SDK消费步骤 |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/chat_completions_tool_tests.rs` | 大参数/分块/工具恢复和部分工具后预算错误组合回归、共享导出helper |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/common_protocol_tests.rs` | PNG有效图片协议回归，复用图片字段完整性断言 |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/common.rs` | 修正终结控制行探测、256字节有界前缀与长行状态 |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/common_tests.rs` | 正文/长行/无空格/跨块终结边界回归 |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/openai_responses.rs` | 顶层事件语义、native预算错误与控制帧边界回归 |
| `crates/service/src/gateway/upstream/attempt_flow/postprocess_tests.rs` | 仅目标Cloudflare测试的client/mock/send分段计时和HTTP状态诊断，未改业务行为 |
| `crates/service/src/gateway/upstream/proxy_pipeline/candidate_executor_tests.rs` | 仅目标取消测试的阶段计时和超时现场，原5s/100ms及全部断言保留 |
| `docs/zh-CN/report/code-quality-2026-10-17-round2/R03-result.md` | 唯一新增R03报告 |

其它 manifest 中的来源文件完整保留；`chat_completions.rs` 相对 Q02 只增加 Q07 的活动时间读取，`scripts/chat-tool-indices-sdk-check.py` 保持 Q02 原版。日志/fixture/临时依赖及 PNG 校验辅助脚本位于忽略目录 `target/r03-*`，不作为产品源码。

导入对照记录为 `target/r03-evidence/imported-file-comparison.json`；patch 应用受工作区 LF/CRLF 影响，原始文件字节 SHA 可能不同，另做统一换行后的文本对照。两份第一轮报告和 SDK 脚本与冻结文件的字节 SHA 仍完全匹配。除本节列出增量和预期的 Chat 双方组合文件外，其余来源内容对照一致。

## 终态审查

- HEAD 仍为 `f14bd735dca9203d7430958480b82ba7713641a9`，暂存区为空；16 个 tracked 修改、8 个 untracked 文件，均属于冻结输入或本轮增量，没有删除项或范围外文件。
- 全部24个文件的路径、字节数、SHA256与冻结来源对照保存在 `target/r03-evidence/final-file-inventory.json`；终态核验记录在 `target/r03-evidence/final-audit.json`。重新核对冻结 manifest、patch 及其全部来源文件，Q02/Q07 报告与 SDK 脚本仍保持原始字节 SHA。
- 最终 `git diff --check` 退出0；仅有既存换行规范提示。未遗留可执行文件路径位于本工作区内的测试或服务进程，记录为 `target/r03-evidence/worktree-processes-final.json`。

## 未执行项

- `cargo test --workspace -j 2 --no-fail-fast`：未执行。本轮改动限定 service gateway/SSE 及其 CI/测试，按根 AGENTS 采用相关 package 范围；全 workspace、R02/R05 存储/设置并发验收仍需最终统一版本执行，不能用本范围代替。
- `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2`：未执行，无桌面代码改动；Tauri 不在根 workspace 中，本报告没有桌面通过结论。
- `pnpm -C apps run build`、`pnpm -C apps run build:desktop`、`pnpm -C apps run test:runtime`、`cargo test -p codexmanager-web -j 2`：未执行，无前端/静态导出/Web shell/transport 改动，不作这些范围的完成声明。
- 真实上游、真实图片生成、真实账号数据库及浏览器完整端到端未执行；PNG 是已解码验证的本地固定样本，大 base64 是合成载荷。
- GitHub Actions 未远程执行（未推送）；本地运行对应 Rust 回归与官方 SDK。CI 只补最小定向入口，没有宣称远程 CI 已通过。

## 预算边界与后续汇总

- 单帧/排队线上字节各 128 MiB、队列通道容量 128 帧、单帧最多 65,536 行；进程内公共帧泵共享 512 MiB 存储记账。通道容量不代表总持有帧数上限，另有正在消费及解析、待入队的帧。按 `2 × 线上字节 + 每行 (2 × size_of::<String>() + 8)` 计账；64 位平台每行 56 字节。未完成帧、排队帧和正在消费的末帧持有许可。
- 这不是进程 RSS 上限。底层 HTTP/chunk 队列、协议 JSON 树、累计文本/工具状态、下游缓冲及分配器开销不在此预算内。全局预算不足时，即使单帧合法也可能明确失败；超限不会截断 JSON/base64 后当作成功。
- 非流式 Chat 的 `message.tool_calls` 缺口归 R04，本工作区不自动包含其独立修复；其他非流式/流式、tools 和两个 OpenAI 端点断言照常执行。
- R04 的最终增量仍须与本 Q02+Q07+R03 组合做三方整合、重新回归；R02/R05 等独立通过不能证明统一版本全部通过。
- 未授权且未执行暂存、提交、推送、PR、合入或发布。本地改动供审查。
