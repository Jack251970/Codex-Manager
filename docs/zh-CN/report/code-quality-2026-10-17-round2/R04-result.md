# R04：非流式 Chat 工具调用保真修复（2026-10-17）

## 工作区、输入与交付边界

- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\round2-chat-tools\Codex-Manager`。
- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。创建前 `list_artifacts` 为空；`create_worktree(ref=<基线>, name=round2-chat-tools, allowAsync=true)` 的 operation `eafab3d1-624d-4c1f-b5d4-6f08609cc663` 已确认 completed。初始 HEAD 一致、状态干净，修改前已读工作区根 `AGENTS.md`。
- 原仓库两份任务书仅从指定绝对路径读取，原仓库、第一轮来源工作树及冻结输入均未改写；未读取其他第二轮正在变化的工作树。
- 只导入冻结 `Q02` 完整改动，没有导入 Q07 或其他 Qxx。没有暂存、提交、推送、PR、合入主仓库、改版本或发布；后续必须把本增量整合进 R03，再重新组合验收。
- HTTP 验证使用生产路由、临时 SQLite、合成账号/密钥及回环 mock 上游；未操作真实账号、真实数据库、生产服务或自动启动配置。

冻结根：`C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round2-3e2657e8\Q02`。逐项重新计算 SHA256 并对照 manifest；manifest 本身 SHA256 为 `A9B719E0AF7EFF325A0935B47AD68AB13BB970D217B39B8002D7E03B6326F634`，patch SHA256 为 `6732306B3A93644AA3823532CFA1BFB104CE04D3D7557779226B9A4ECDB8A601`。

| Q02 输入文件 | SHA256 |
| --- | --- |
| `.github/workflows/ci.yml` | `C6E119E0DFDE848322D796D98746EC03802A4BD674E568AAADD10EFEA8DA3D34` |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/chat_completions.rs` | `2F3A22B47872677D27ADC61B2C57B9A79A0C4B24D3C5A2F0946E6F31318DB403` |
| `crates/service/src/gateway/observability/http_bridge/stream_readers/chat_completions_tool_tests.rs` | `C449443EECCF9A53722CD9032A1F40F7E3DCE899EE8787C0DABCBC7D6815EEF5` |
| `docs/zh-CN/report/code-quality-2026-10-17/Q02-result.md` | `81B41A3812A2EE0B2B88F8486A01B2A04ECBC99FA9C86570A3AA25E53CE2BBCE` |
| `scripts/chat-tool-indices-sdk-check.py` | `B1647EF72F54DEA880015D61271798D758EF2256F595F4953A6D09E4FF887C06` |

导入时先 `git apply --check <冻结Q02>/tracked.patch`，再 `git apply`，两次退出 0；复制 manifest 全部 3 份 untracked 文件，没有删除项或内容冲突。Windows `core.autocrlf=true` 使 reader 应用 patch 后字节哈希不同；先核实 CRLF/LF 归一后的文本完全相等，再恢复冻结副本的原字节，未覆盖其他任务语义。最终除 CI 的本轮新增步骤外，上述 Q02 reader、测试、SDK 脚本和第一轮报告全部保持原 SHA。导入校验另存 `target/r04-evidence/input-verification.json`。

## 生产根因与修复

1. 账号路由的非原生 Chat：`local_validation/request.rs` 选择 `ChatCompletionsFromResponses`；`http_bridge/delivery.rs` 非流式分支先聚合上游 SSE（若有），再调用 `convert_success_body_for_adapter`。原 `body_conversion.rs::convert_responses_body_to_chat_completions` 只生成文本/reasoning/images/usage，没有 `message.tool_calls`，且固定 `finish_reason=stop`。实际 HTTP 负对照已证实，非未使用 helper 的静态推测。
2. 本轮在这个现有 converter 内按 `output` 顺序收集 `function_call`：优先有效非空 `call_id`，其次有效 `id`；名称原样保留，字符串参数原样保留，对象/数组等 JSON 值仅序列化一次。有工具时 `finish_reason=tool_calls`；纯工具 `content=null`，混合内容仍保留文本；无工具保持原来的 `stop` 和文本行为。usage 仍复用现有映射。畸形上游缺少必需 ID/名称不虚构新身份，不宣称已新增上游 schema 校验。
3. 原生 Chat 的非流式 SSE 合成另有同类丢失：`aggregate/sse_aggregate.rs` 原先只累计 role/content/finish_reason。新增独立 `chat_function_calls.rs` 保存每个 choice 内的原生工具 index，交错 delta 按各自 index 累计 ID、名称、参数，输出按原生 index 排序，纯工具内容为 null；保留上游 finish_reason 和 usage。原生 Chat JSON 仍走既有透传，不加入额外转换。
4. 复核又发现 Responses SSE 旧 `merge_tool_call_arguments` 会用字符串前后缀去重，吞掉合法重复 delta，并可能把随后 done 快照拼成两个 JSON。已通过真实 HTTP 新测试复现 `{"x":"aa"}` 与预期 `{"x":"aaaa"}` 差异；现在 delta 直接追加，非空 done 按完整快照处理（空 done 保持旧行为，不清除先前累计）。原生 Chat delta 同样直接追加，不能借用快照去重规则。

协议边界：Chat 的 Responses 适配与 Q02 stream reader 一样，只映射 `function_call`；不把 `custom_tool_call.input` 猜测转换成函数 arguments。custom-only 与 custom+function 均覆盖 JSON/SSE；原生 Responses 保留其 function/custom 输出。原生 Chat JSON 保留原格式；新增原生 Chat SSE 合成只验收函数工具，不宣称已新增自定义 Chat 工具流、旧 `function_call` 字段或其他工具类型支持。未改 incomplete/失败响应的协议契约。

## 负对照及失败证据

所有命令在本工作区根运行。公共环境为：`CARGO_TARGET_DIR=<本工作区>/target`、`TEMP=TMP=<本工作区>/target/r04-runtime-tmp`。Rust 均 `-j 2`，不共享其他工作区可写 target/out；本轮 Rust 构建/测试串行启动，SDK 与只读格式/哈希检查可独立执行，未使用旧来源日志替代。

| 实际命令 | 退出码与计数 | 证据 |
| --- | --- | --- |
| `cargo test -p codexmanager-service --test gateway_logs non_stream_chat_preserves_function_calls_from_responses -j 2 -- --nocapture` | 101；0 passed / 1 failed / 0 ignored | `target/r04-baseline-http.log`、`target/r04-baseline-http/`。首次测试配置只绑定聚合路由，目录模型把 key 有效策略归为聚合透传，实际上游仍是 messages、返回原 Responses；该失败属于 harness 路径错误，**不计为目标 converter 复现**。随后改用临时账号路由，并增加实际上游 input/messages 强断言。 |
| `cargo test -p codexmanager-service --test gateway_logs non_stream_native_chat_sse_preserves_interleaved_tool_deltas -j 2 -- --nocapture` | 101；0 / 1 / 0 | `target/r04-baseline-native-sse.log`、同名前缀 fixture 目录；真实原生 Chat 上游 messages、stream=false，实际结果缺少两项 tool_calls。 |
| `cargo test -p codexmanager-service --test gateway_logs chat_non_stream_tools:: -j 2 -- --nocapture`（产品修复前，`CM_REPAIR_FIXTURE_DIR=target/r04-baseline-account`） | 101；3 passed / 5 failed / 0 ignored | `target/r04-baseline-account.log`、`target/r04-baseline-account/`。已强断言账号链上游为 input、无 messages；实际 Responses→Chat 响应有文本/usage，却缺工具且 stop。原生 Chat JSON、Responses 与纯文本对照通过。 |
| `python -B scripts/chat-non-stream-tools-sdk-check.py target/r04-baseline-account --allow-partial` | 1；首份工具结果严格断言失败 | `target/r04-sdk-baseline.log`；官方 SDK 消费实际网关结果后同样没有 tool_calls。`--allow-partial` 仅用于负对照，最终验证不使用。 |
| `cargo test -p codexmanager-service --test gateway_logs chat_non_stream_tools:: -j 2 -- --nocapture`（首轮产品修复后，新增重复参数测试） | 101；8 passed / 1 failed / 0 ignored | `target/r04-http-final.log`、`target/r04-http-final/`。文件名含 final 但这次**没有通过**：重复参数为 aa，预期 aaaa；此失败促成第 4 项修复。该目录有 16 份结果，不作为最终完整 SDK 输入。 |

首个 harness 误配及后续真实产品失败均保留，没有通过删除日志、skip、扩大业务超时、弱化工具断言或无上限重试制造通过。HTTP fixture 单次上游、捕获次数断言为 1；原生 Chat 另断言 messages 存在、input 不存在和 stream=false；转换链强断言 input 数组与无 messages。

## 相对 Q02 快照新增的文件/差异

| 路径 | 本轮增量 |
| --- | --- |
| `crates/service/src/gateway/observability/http_bridge/body_conversion.rs` | 非流式 Responses 函数工具映射、参数保真、content/finish_reason。 |
| `crates/service/src/gateway/observability/http_bridge/aggregate/sse_aggregate.rs` | 原生 Chat SSE 汇总接入工具状态；Responses delta/done 语义区分。 |
| `crates/service/src/gateway/observability/http_bridge/aggregate/chat_function_calls.rs`（新） | 每 choice 的原生 Chat 函数工具状态，按 index 合并与排序。 |
| `crates/service/tests/gateway_logs.rs` | 仅注册新测试模块。 |
| `crates/service/tests/gateway_logs/chat_non_stream_tools.rs`（新） | 9 个真实 HTTP 回归，17 份可选 fixture；所有期望独立构造。 |
| `scripts/chat-non-stream-tools-sdk-check.py`（新） | 官方 SDK 消费实际 HTTP 结果回放；默认强制完整 17 case 集合、禁 SDK 重试、断言一次请求。 |
| `.github/workflows/ci.yml` | 保留 Q02 streaming step，再新增 `gateway_logs chat_non_stream_tools::` 的最小 CI 入口。 |
| 本 `R04-result.md`（新） | 独立记录第二轮结果，不改写第一轮 Q02 报告。 |

R03 汇总时只取这 8 个路径相对 Q02 的增量，不用本工作区整份旧 reader 覆盖 R03 的 Q02+Q07 reader。CI 只追加新 step，保留 R03/Q07 的其他步骤。共同基线不含任何 Qxx 未提交改动；仅创建基线工作区或只取 tracked.patch 都不能得到完整交付。

## 修复版本验证结果

以下都是本工作区新执行的结果；`gateway_logs` 首次完整运行失败，不以定向运行或 SDK 通过覆盖该事实。

| 实际命令 | 退出码；passed / failed / ignored | 日志与范围 |
| --- | --- | --- |
| `cargo test -p codexmanager-service --test gateway_logs -j 2 --no-fail-fast` | **101；60 / 6 / 0**，无筛选 | `target/r04-gateway-logs-verified.log`；默认测试并发。新增 9 项全通过并导出完整 17 份真实 HTTP 结果到 `target/r04-verified-http/`。完整测试未通过。 |
| `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --test-threads=1` | **0；915 / 0 / 0**，965 filtered out | `target/r04-service-gateway.log`；gateway 单元回归包含 reader、HTTP bridge、adapter、upstream、Claude/Gemini/图片等既有回归。`CM_REPAIR_FIXTURE_DIR=target/r04-verified-stream`。串行运行不替代默认并发的完整 gateway_logs 结果。 |
| `python -B scripts/chat-non-stream-tools-sdk-check.py target/r04-verified-http` | **0；17 / 0 / 0** | `target/r04-sdk-non-stream-verified.log`；默认完整 case 集合检查，无 `--allow-partial`。 |
| `python -B scripts/chat-tool-indices-sdk-check.py target/r04-verified-stream` | **0；13 / 0 / 0** | `target/r04-sdk-stream-verified.log`；Q02 脚本未改，读取本次实际组合 reader 导出的全部 13 份 SSE，官方 SDK helper 重建连续工具序号、ID、名称及精确参数。 |
| `cargo test -p codexmanager-service --test gateway_logs chat_non_stream_tools:: -j 2 -- --test-threads=1`（Q02 对照恢复后） | **0；9 / 0 / 0**，57 filtered out | `target/r04-ci-final.log`；新增 CI 最小入口，`CM_REPAIR_FIXTURE_DIR=target/r04-final-http`，全部 17 份结果重新导出。 |
| `python -B scripts/chat-non-stream-tools-sdk-check.py target/r04-final-http` | **0；17 / 0 / 0** | `target/r04-sdk-non-stream-final.log`；消费恢复后重新导出的完整结果。 |
| `cargo test -p codexmanager-service --test gateway_logs -j 2 --no-fail-fast`（Q02 对照恢复后第二次完整运行） | **0；66 / 0 / 0**，0 filtered out | `target/r04-gateway-logs-final-default.log`；默认并发，无 `--test-threads=1`，193.59 秒；`CM_REPAIR_FIXTURE_DIR=target/r04-final-default-http`。同一恢复后的 R04 产品源码，未增加超时、skip 或改断言。 |
| `python -B scripts/chat-non-stream-tools-sdk-check.py target/r04-final-default-http` | **0；17 / 0 / 0** | `target/r04-sdk-final-default.log`；消费最后一次完整默认并发运行导出的 17 份实际 HTTP 结果。 |

Python 实际解释器为 `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe`；`PYTHONPATH=<本工作区>/target/r04-sdk-python`，依赖 `openai==2.29.0`、`httpx==0.28.1`。依赖安装退出 0，日志 `target/r04-sdk-install.log`。两份 SDK 检查均使用 `httpx.MockTransport` 回放实际 Rust 网关产物；Rust 测试本身经真实 loopback HTTP 走生产网关调用链，**没有声称 SDK 直接连接 Rust 网关或真实上游**。

### 完整 gateway_logs 的 6 个失败与有界诊断

每项修复版本仅单独复跑 1 次，完整命令统一为 `cargo test -p codexmanager-service --test gateway_logs <下表测试全名> -j 2 -- --exact --nocapture`。每次实际执行 1 项（65 filtered out、0 ignored），未改任何超时、重试或断言；结果索引为 `target/r04-diagnostic-exits.json`，逐项日志前缀为 `target/r04-diagnostic-`，后接测试全名（`::` 换为 `-`）。

| 测试全名 | 完整运行失败 | 修复版本单次诊断 |
| --- | --- | --- |
| `retry_logging::gateway_stateless_retry_strips_encrypted_content_on_invalid_encrypted_content` | `support.rs:209` 下游客户端 2 秒读超时（OS 10060） | 101；0 / 1 / 0 |
| `stream_timeout_failover::gateway_silent_response_headers_fail_over_when_total_timeout_disabled` | `stream_timeout_failover.rs:236` 预期 200、实际 502，仍报 200ms 响应头超时 | 101；0 / 1 / 0 |
| `stream_timeout_failover::gateway_single_silent_account_still_obeys_stream_header_timeout` | `stream_timeout_failover.rs:321` mock 捕获请求为 `[]` | 101；0 / 1 / 0 |
| `usage_limit_failover::gateway_low_quota_account_is_skipped_on_first_request` | `support.rs:209` 下游客户端 2 秒读超时 | 101；0 / 1 / 0 |
| `usage_limit_failover::gateway_non_stream_responses_usage_limit_sse_fails_over_before_delivery` | `support.rs:209` 下游客户端 2 秒读超时 | 0；1 / 0 / 0 |
| `usage_limit_failover::gateway_usage_limit_in_initial_sse_fails_over_before_delivery` | `support.rs:209` 下游客户端 2 秒读超时 | 0；1 / 0 / 0 |

另对完整运行 PID 24596 的指定 8 份本工作区临时 SQLite 以 `mode=ro` 读取请求日志及发送尝试，未读取他人资源；证据为 `target/r04-evidence/r04-failure-analysis.json`。6 个失败全部 original/adapted path 为 `/v1/responses`、adapter 为 `Passthrough`，`first_response_ms=null`，不直接进入本次 Responses→Chat 成功转换。4 个客户端读超时最终记录 499 / downstream request cancelled，分别在 1948–1986ms 结束且仅 1 次发送尝试；low-quota 已正确选择 `acc_healthy`，另外三个尚未进入下一候选/成功交付。双账号 header timeout 已选 PRIMARY→SECONDARY，但次账号仍头超时；单账号有发送尝试登记而 mock 捕获为空。

这些记录能定位失败发生于响应转换前，不能证明 mock 是否收齐 HTTP、不能把“登记发送”当作对端已收到，也不能据此归因为机器负载。`ENV_LOCK` 会使完整测试排队，`running over 60 seconds` 本身不证明并发环境冲突。single-header 的同名问题也已在任务书 R05 登记，但本轮没有读取或借用其运行中工作树的结论。

### Q02 产品源码对照

为检验上述失败是否也发生于修复前，等本轮所有 build/test 结束后，在同一工作区精确备份两个 R04 产品文件及 `gateway_logs.rs` 的字节，然后仅将这三处临时替换为基线 `git show f14bd735:<路径>` 的内容。Q02 reader 和 Q02 测试保持导入版本，R04 helper/测试文件临时不被模块引用；这是 **Q02 产品源码与原测试注册的受控对照**，不是另一个干净 worktree 或完整 Q02 全量验收。

实际编排命令为 `python -B target/r04-q02-control.py`（退出 0，表示编排和恢复成功）。它顺序执行上表 6 个完整 `cargo test ... <测试全名> -j 2 -- --exact --nocapture`，每项仅 1 遍，全部 **exit 0；1 passed / 0 failed / 0 ignored / 56 filtered out**。逐项日志前缀 `target/r04-q02-control-`；完整命令、每次退出码、切换前后 SHA256 和恢复结果为 `target/r04-evidence/q02-control-results.json`，编排日志 `target/r04-q02-control.log`。`try/finally` 已逐字节恢复 R04 原文件，`restored=true`；未使用 reset/stash/checkout，也未改动任何其他工作区。

Q02 这一遍未复现 6 个失败，**不能称这些失败已被证明为基线失败或已知机器抖动**。R04 前一遍单项诊断为 4 失败 / 2 通过，对照时刻与执行上下文不同；尚不能确定共享客户端、网络、mock 观测时序或其他间接因素中的具体根因。恢复后的 R04 又执行了且仅执行了 1 次完整默认并发运行，结果为 **exit 0；66 passed / 0 failed / 0 ignored**。首次 60/6、定向结果、Q02 对照与最后 66/0 均独立保留；最后一遍通过不改写首次失败，也不证明此前失败的具体原因已被修复。

只读追查 R04 fixture 的环境和运行态后，未发现新增的上游 URL、HTTP client、候选账号或 runtime settings 残留：EnvGuard 保存原 OsString，DB_PATH 同时恢复 TEST_DB_DIR，在外层 ENV_LOCK 释放前析构；R04 seed 不写 app_settings；启动时重新加载环境并清理客户端/路由缓存，存储池按数据库完整路径分桶。独立进程单项诊断根本未执行 R04 fixture，所以它的环境清理也不能解释该单项差异。

同时发现两处既有的同进程测试隔离边界：payload 后台队列的 job 无 DB 身份，writer 执行时才读取当前 DB，而 `server.join()` 不等待该队列；启动 reconcile 也未 join，延迟执行时可能读取后续 DB。两者尚无本轮因果复现，不能当作六项失败的已定根因，也不能解释每项独立进程的差异。本轮未跨范围修改这些机制；失败原因仍列为未解决，不能声称已排除一切间接回归。

## 未执行与协议支持边界

- `cargo test --workspace -j 2 --no-fail-fast`：未执行。本增量局限 service gateway 的响应适配/聚合，已选 service gateway 单元与完整 `gateway_logs` 范围；不能据此声称 workspace、storage、Web 或其他组合全部通过。
- `cargo test --manifest-path apps/src-tauri/Cargo.toml`、`pnpm -C apps run build`、`pnpm -C apps run build:desktop`、`pnpm -C apps run test:runtime`、`cargo test -p codexmanager-web`：未执行；没有修改 apps、桌面 RPC、前端或 Web shell/transport。
- 远端 GitHub Actions、真实账号上游、联网 SDK 直连网关、浏览器完整端到端未执行。新增 CI 步骤仅在本地验证相同入口，不代表远端 CI 已完成。
- 原有 incomplete/失败响应、缺少工具 ID/名称等畸形上游输入的 schema 校验未扩展；native Chat `custom`/旧 `function_call` 契约未新增。Responses 参数空 done 保留旧累计行为；未对该语义作新的完整支持承诺。
- Q07 不在本工作区。R04 尚须在明确的后续交接点把上述增量整合到 R03（Q02+Q07），处理共享 SSE/CI 文件后重新执行组合回归；本轮结果不能替代最终组合验收。

## 增量交接与现场检查

除本报告外的 7 个代码/测试/CI/脚本路径，另导出为 `target/r04-evidence/r04-over-q02-code.patch`，SHA256 为 `5738DB7A4AA646D7A256AFB4191821F01BCCCCC665489774A8B746AB2C7935E1`。旧侧来自冻结 Q02 的 CI，其余旧侧来自共同基线；新增文件用 `/dev/null`。该补丁不包含原 Q02 reader、Q02 新文件或本报告，避免汇总时覆盖 R03 的组合 reader；逐文件 SHA256 为 `target/r04-evidence/r04-increment-files.json`。`target/` 为本地证据，不是 Git 提交。

- `cargo fmt --all -- --check`：初次、修复后及 Q02 对照恢复后均退出 0；日志分别为 `target/r04-fmt-initial.log`、`target/r04-fmt-final.log`、`target/r04-fmt-restored.log`。
- `git diff --check`：退出 0；日志 `target/r04-diff-check.log`、`target/r04-diff-check-restored.log`、`target/r04-diff-check-final.log`。仅有 Q02 原 reader 的 LF→CRLF 提示，没有重新改写该文件。
- `python -B target/r04-final-verification.py`：退出 0，证据为 `target/r04-evidence/final-verification.json` 与 `target/r04-final-verification.log`；HEAD 仍为基线、暂存区为空，Q02 的 4 份原文件及 7 份 R04 增量 SHA 全部符合既有记录，Q02 对照还原后 3 个文件逐字节相同，增量补丁 SHA 一致且 `git apply --check --reverse target/r04-evidence/r04-over-q02-code.patch` 退出 0。
- 最后检查本工作区可执行文件及关联 cargo/rustc/测试进程，匹配数为 0；`target/r04-evidence/final-process-check.json` 记录检查时间和结果。本轮工具会话均已结束，无本任务启动的测试服务遗留；保留合成临时 DB/fixture 作为失败和复现证据。

## 最终状态

R04 目标缺陷已在本工作区修复并形成可审查的本地改动：新增 CI 入口 9/9、非流式官方 SDK 17/17、Q02 流式官方 SDK 13/13、service gateway 单元 915/915；最后一次完整默认并发 gateway_logs 为 66/66。首次 6 个失败的证据与原因未定位的边界仍保留，没有将多次结果合并冒充一次全部通过。

未暂存、提交、推送、创建 PR、合入主仓库或发布；共同 HEAD 和 Q02 原报告均保留。交付只代表 Q02+R04 本地验证，后续仍须将本增量整合进 R03 的 Q02+Q07 再验收。
