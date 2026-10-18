# S02：工具调用、SSE 与测试可靠性组合验收

当前状态：**S02 范围最终验收通过，等待父会话冻结交给 S03。** 本轮默认并发结果及 SDK 产物来自同一最终源码；不代表 workspace、前端或桌面统一验收通过。

## 工作区与授权边界

- 本会话独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\round3-tools-final\Codex-Manager`。
- 共同 HEAD：`f14bd735dca9203d7430958480b82ba7713641a9`。先 `list_artifacts` 得到空附件，再按明确 ref 创建 `round3-tools-final`；operation `043ebe3b-1a9a-4b6a-8594-ad87f69fbb3e` 返回 completed 后确认 HEAD 和初始干净状态，才编辑。
- 已读主工作区根 `AGENTS.md`、第三轮任务书及 R03/R04/R05 来源报告。主工作区、来源工作树、冻结输入保持只读；本轮不暂存、提交、推送、创建 PR、合入或发布。
- 父会话 `01a144dc-ec9f-74d3-bcc1-7cfe031e5c07` 明确通知“冻结输入已完成”后才导入。S01 完成后，按父会话授权接收重型时段再启动 Rust 验证；使用独立 target/TEMP、`-j 2` 和默认测试线程。
- 仅使用合成数据、临时库、回环 mock、假凭据。没有操作真实账号、用户库、生产服务、系统代理设置或自动启动配置。

## 固定输入与独立校验

输入根：`C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round3`。

`index.status=VERIFIED`，index SHA256：`6A728FFC70DF32DB3A46ED57B5E9119D5CDA213AD1086560E13B2CD174B520CE`。

| 来源 | manifest SHA256 | tracked patch SHA256 | 独立校验 |
| --- | --- | --- | --- |
| R03 | `86E2255B1080678A52F324BDABAFC2C09E19077964E3D0BED868D97D3D3FB13E` | `F62127583ED73A58B19DD66ABF100124393A4896A5AD4954D141CA6AB26C9E84` | 16 tracked + 8 untracked 全部匹配 |
| R04 | `A3D7453B9A0FF0A55D107823B296582E7C98400F9C68EDF73E5FD859076F69E0` | `71DB3E8FC5E259D3300C034042D3D56BEB7EFC2DF9EB586F3A4C22C5DE28EFA5` | 5 tracked + 7 untracked 全部匹配；不整份导入 |
| R05 | `F7CC8FC7AB102CD2EF3633AE1362D608682995361FECC18EDEE7E7D3FDB1308F` | `3117DD99DAE0AC99F4E0E8D7EDF6B31444137D5B3F6AFD43034A7428B1B9FCB6` | 45 tracked + 4 untracked 全部匹配；只导入指定两测试文件 |

三份快照共 85 份源码/测试/脚本/报告逐个校验大小与 SHA256，均无删除项。记录：`target/s02-evidence/input-verification.json`，实际导入命令及退出码：`import-commands.json`；导入脚本 `import-inputs.py` 本次退出 0。

R04 明确增量 `R04/explicit-increment/r04-over-q02-code.patch` 的 SHA256 为 `5738DB7A4AA646D7A256AFB4191821F01BCCCCC665489774A8B746AB2C7935E1`；其文件清单 SHA256 为 `C2C316FD31A70E2128D94A58096E8DE1F70A3CE336AFE2362D62234EF2368CFA`。二者均独立复核。

## 导入与三方处理

1. R03 完整 tracked patch `git apply --check` 和 apply 均退出 0，全部 8 份新增文件导入。初次完整快照可安全恢复冻结文件原字节，避免 Windows 换行差异；R03 的 24 路径完整保留。
2. 在 R03 之上检查 R04 相对 Q02 的明确增量，整体 check 退出 1，唯一冲突是 `.github/workflows/ci.yml:56`。排除 CI 后 check/apply 均退出 0；6 个非 CI 路径归一换行对照后与明确增量目标 SHA 完全匹配。R04 报告单独原字节导入。
3. CI 以共同基线、R03（Q02+Q07）及 R04（Q02+非流式增量）逐段比较，保留双方全部步骤。完整 YAML 解析发现 R04 原行内 `run` 的 `chat_non_stream_tools::` 尾冒号后接空格，源语法无效；组合版只给该命令加引号，命令内容不变。另新增非流式官方 SDK 消费及本轮局部隔离/多 choice 检查入口。
4. R04 增量没有 reader；没有复制 R04 的旧 `chat_completions.rs` 或旧工具测试覆盖 R03。R03 生产 reader 的 SHA256 仍为 `0BF089A5E7F55AB9C2330BE7CA0C6C02D01ACC87290CB3D7EA6BBF13A80B72F4`，保留 SSE 活动时间刷新及连续工具索引、ID/name/arguments 补发。
5. R05 只导入 `crates/service/tests/gateway_logs/support.rs`（源 SHA256 `FA9690D06DBFC4D09CFF5F4EE9CE3E9C3BB5F6A48A3DD368E2F4A47CEC490AE8`）及 `stream_timeout_failover.rs`（源 SHA256 `9C33FF36349E61986290694249D0322F1C7DBC8428E06FAFF54AD7D7A3338AB0`）。其产品/设置/存储/其余隔离增量由 S01 处理，没有越界引入。

## 本组合额外回归及夹具修复

没有新增产品代理策略或预算设置；产品修复来自原样保留的 R03/R04。

- 多 choice 原生 Chat SSE 聚合：新增 `collect_non_stream_chat_sse_keeps_tools_isolated_per_choice`。独立字面量期望检查 choice 1→0 乱序、每 choice 的 tool 1→0 排序、同 index 跨 choice 隔离、重复 `aa` 参数片段不丢、name 补发、混合文本/纯工具 null、各自 finish_reason、尾帧 usage 和 metadata。
- R03 原文本/reasoning 对等测试同时核对非流式/流式的准确工具数组、tool_calls finish_reason 和 usage，去掉已不成立的非流式缺口注释。未新增 fixture 数量或以实现输出自行生成期望。
- 流式 SDK 检查默认强制 `q02-0..13` 14 份 SSE 和全部配对期望文件，拒绝缺失、额外及孤立期望；保留原稀疏 index 的显式负控模式。非流式脚本仍默认强制原 17 完整 case，正式验收不用 `--allow-partial`。
- `gateway_logs` HTTP helper 仍使用原 2 秒读超时，读错误直接失败；只增加脱敏 phase 元数据及纯诊断回归，明确区分 no_response、partial_headers、partial/eof-delimited body、完整 framing 后等待 EOF。不把部分响应当成功，不打印响应内容或凭据。
- `crates/service/src/tests/loopback_proxy.rs` 是仅测试的局部 helper，借用原 `crate::test_env_guard` 锁。逆序保存/恢复大小写代理环境，设置回环 NO_PROXY，清除 CODEX 代理和 REQUEST_METHOD；进入和退出均用既有 reload 重建缓存，恢复后才释放原锁。未改变全局 test_env_guard 或产品代理逻辑。
- 新三项 helper 回归已实际覆盖实际 Matcher 布尔、沉默本地代理负控、默认新 client/缓存 client 直连正控、显式 Proxy::all 仍生效、退出后缓存恢复、panic/进入失败环境恢复。helper 位于 gateway 过滤器之外，必须单独执行，不能用 gateway 计数代替。
- reload 本身若 panic，故障注入仅证明环境恢复且不会因 unwind 中第二次 panic 而 abort；不能声称部分更新的缓存也必然恢复成功。正常 reload/测试 panic 后的缓存恢复由另外的回归验证。
- Cloudflare 原 2 秒 mock、403、仅 1 发、直接 Failover 保留；补明确 403 断言和局部 guard。取消测试保留原 5 秒总界限、100ms 禁止额外连接、Handled、首账号恰一条 499，并要求实际 `Ok(0)` EOF，读错误不得冒充 EOF。
- R05 两项 header timeout 保留 200ms、成功 200/准确 502 文本、1..2/2..5 次数、耗时、trace/wire SHA 和日志断言；其 mock actor 完成/异常回收及显式代理保护保持来源语义。

- 增补两项公开入口流式工具 HTTP 回归：`stream_http_native_responses_function_tools` 与 `stream_http_account_chat_function_tools`。两者真实回环 HTTP 经过生产路由，断言上游请求路径、input/tools/stream、单次捕获、三个交错工具、非工具 output index、ID/name 补发、参数快照、混合文本/reasoning、usage/终结。原生 Responses 逐字节保留原始 SSE；Chat 精确重建连续工具索引。R04 原 31,597 字节文件前缀及 17 根目录非流式 fixture 保留，新增流式产物单独写入 `stream-http/`。
- 官方 SDK 新脚本 `scripts/public-stream-tools-sdk-check.py` 严格消费上述两份公开流式 HTTP 产物，检查三个工具的准确内容、JSON 参数、文本、usage/finish/native completed output；reasoning、上游路由与原始终结由 Rust 断言，不冒充 SDK 独立检查。
- 首轮完整 gateway 暴露两项 exact-wire 测试隔离缺口后，只改 `openai_responses.rs` 的 `cfg(test)` 区域：局部 RAII 借原共享锁，保存环境 OsString 与缓存 bool，移除 enabled 覆盖后用既有 setter 临时禁用生成心跳；退出先恢复缓存、再恢复原环境、最后释放锁。避免全量 reload 改变无关运行态。读 fixture、split/CR-only/预算前缀用例保持所有原超时、原事件与完整字节断言。
- 新增四项回归：缓存/环境恢复（含无值、空值、开关覆盖）、panic 恢复、比较器拒绝额外注释/修改/截断、实际 `next_chunk` 开关成对验证。后者启用时精确输出合法注释且不误记首响应/终结，关闭时空输出，随后上游自带同样注释及 CRLF terminal 仍逐字节保留。64MiB 仍完整比较所有字节，失败仅输出长度、首差位置及短窗口，不 strip、不用 hash 替代相等断言。
- R03 原生 reader 生产前缀逐字节不变：12,169 字节，SHA256 `FF015ACC41A21CE74FD78451B8C6F042B2DA91EE6780DAD1BD8D5743B678AE5D`；整文件最终 SHA256 `A604CE03C393F5811A2501EDD7370DC803AF59A44BACBEA648C8F47DA8D5A40C`。

## 实际命令、退出码与日志

验证器 `target/s02-evidence/run-command.py` 每条命令仅执行一次，拒绝覆盖既有日志及非空 `RUST_TEST_THREADS`，记录真实子进程退出码、用时、测试计数和全新 fixture 的大小/SHA256。`CARGO_TARGET_DIR`、TEMP/TMP 均位于本工作区 target，Rust 编译并行度为 2；正式测试保持默认测试线程。以下日志均位于 `target/s02-evidence/`，全部命令参数另存 `validation-results.jsonl`。

| 记录 | 实际命令 | exit | 结果 passed/failed/ignored/filtered | 用时秒 | 日志 |
| --- | --- | ---: | --- | ---: | --- |
| `diagnostic1-proxy-controls` | `cargo test -p codexmanager-service --lib -j 2 loopback_proxy_test:: -- --nocapture` | 0 | 3/0/0/1922 | 329.844 | `diagnostic1-proxy-controls.log` |
| `diagnostic2-historical-lib` | `target/debug/deps/codexmanager_service-f7b3088b64652c11.exe --exact gateway::upstream::attempt_flow::postprocess::tests::chatgpt_cloudflare_challenge_directly_failovers_without_same_account_retry gateway::upstream::proxy_pipeline::candidate_executor::tests::cancelled_header_wait_logs_one_499_without_retrying_or_failing_over --nocapture` | 0 | 2/0/0/1923 | 1.203 | `diagnostic2-historical-lib.log` |
| `png-source-validation` | `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe -B target/s02-evidence/verify-png.py` | 1 | 非 Rust harness；见日志 | 0.094 | `png-source-validation.log` |
| `png-source-validation-utf8` | `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe -B target/s02-evidence/verify-png.py` | 0 | 非 Rust harness；见日志 | 0.063 | `png-source-validation-utf8.log` |
| `gateway-logs-compile` | `cargo test -p codexmanager-service --test gateway_logs -j 2 --no-run` | 0 | 非 Rust harness；见日志 | 84.015 | `gateway-logs-compile.log` |
| `diagnostic3-historical-http` | `target/debug/deps/gateway_logs-f263577868460677.exe --exact retry_logging::gateway_stateless_retry_strips_encrypted_content_on_invalid_encrypted_content stream_timeout_failover::gateway_silent_response_headers_fail_over_when_total_timeout_disabled stream_timeout_failover::gateway_single_silent_account_still_obeys_stream_header_timeout usage_limit_failover::gateway_low_quota_account_is_skipped_on_first_request usage_limit_failover::gateway_non_stream_responses_usage_limit_sse_fails_over_before_delivery usage_limit_failover::gateway_usage_limit_in_initial_sse_fails_over_before_delivery --nocapture` | 0 | 6/0/0/69 | 18.422 | `diagnostic3-historical-http.log` |
| `gateway-default-final` | `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --nocapture` | 101 | 955/2/0/968 | 185.906 | `gateway-default-final.log` |
| `native-keepalive-repair` | `cargo test -p codexmanager-service --lib openai_responses -j 2 -- --nocapture` | 0 | 44/0/0/1885 | 119.781 | `native-keepalive-repair.log` |
| `fmt-accepted` | `cargo fmt --all -- --check` | 0 | 非 Rust harness；见日志 | 5.359 | `fmt-accepted.log` |
| `gateway-default-accepted` | `cargo test -p codexmanager-service --lib gateway:: -j 2 -- --nocapture` | 0 | 961/0/0/968 | 68.781 | `gateway-default-accepted.log` |
| `sdk-stream-accepted` | `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe -B scripts/chat-tool-indices-sdk-check.py target/s02-accepted-stream` | 0 | 非 Rust harness；见日志 | 2.844 | `sdk-stream-accepted.log` |
| `static-audit-accepted` | `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe -B target/s02-evidence/static-audit.py` | 0 | 非 Rust harness；见日志 | 0.359 | `static-audit-accepted.log` |
| `diff-accepted` | `git diff --check` | 0 | 非 Rust harness；见日志 | 0.11 | `diff-accepted.log` |
| `gateway-logs-default-accepted` | `cargo test -p codexmanager-service --test gateway_logs -j 2 -- --nocapture` | 0 | 75/0/0/0 | 269.532 | `gateway-logs-default-accepted.log` |
| `sdk-nonstream-accepted` | `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe -B scripts/chat-non-stream-tools-sdk-check.py target/s02-accepted-http` | 0 | 非 Rust harness；见日志 | 2.032 | `sdk-nonstream-accepted.log` |
| `sdk-public-stream-accepted` | `C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe -B scripts/public-stream-tools-sdk-check.py target/s02-accepted-http/stream-http` | 0 | 非 Rust harness；见日志 | 2.125 | `sdk-public-stream-accepted.log` |

轻检查另包括本地依赖安装：OpenAI 2.29.0 / httpx 0.28.1 与 PyYAML 6.0.3，均 exit 0，仅在 target 内。早期完整 YAML 检查 exit 1 揭示冻结 R04 的尾冒号语法错误；另一次 CI 名称对照 exit 1 已恢复源步骤名称。相关失败日志均保留。PNG 校验首次因 Windows 默认 GBK 读取 UTF-8 源码 exit 1，显式 UTF-8 后 exit 0，验证 PNG signature/CRC/IHDR 与解码红色 RGBA 像素；不是实际上游图片生成。

最终 `static-audit-accepted` 完整解析 YAML、三份 SDK 脚本 AST，保留 R03/R04 全部 54 条历史 CI step 记录并检查源保留边界。原 CI 的定向串行命令保持来源内容；本次最终本地完整验证没有使用其串行参数。最终 fmt、diff 检查均 exit 0。33 份非文档变更输入在验收前后大小/SHA256 完全一致：`validation-source-accepted-before.json` 与 `validation-source-accepted-after.json`；测试期间没有继续改源码。

## 历史失败、有界诊断与本轮修复

R03 历史完整 **954/2**、后续诊断版 956/0 仍是来源证据；Cloudflare 当时为 2 秒 mock 无请求、取消为 5 秒流程超时，没有失败时完整阶段记录。R04 历史 **60/6**、六项各一次诊断 4 失败/2 通过、Q02 受控对照六项通过、恢复后 66/0 全部保留；对照不等于干净基线全量，也不证明原失败属于机器抖动。

S02 固定三轮新进程诊断已执行完毕，没有追加第四轮历史诊断或重复直到绿：

1. `loopback_proxy_test::` 三控/缓存/恢复 **3/0**：三项实际 Matcher 观察分别为首项 false/false、后两项 true/false（before/after），默认新 client 与缓存 client 回环直连，显式 Proxy::all 仍生效，正常退出及 panic 后环境/缓存恢复。reload 故障注入仅证明环境恢复及无双重 panic abort。
2. 同一 lib 进程两个历史目标 **2/0**，保留原 Cloudflare 2 秒/实际 403/单发/直接 Failover，以及取消 5 秒/真实 Ok(0) EOF/100ms 禁额外请求/Handled/首账号单条 499。
3. 同一 HTTP harness 进程六个历史目标 **6/0**，保留 retry encrypted_content、三项 usage_limit 及两项 stream header timeout 的全部业务、时限、次数、日志和 trace/wire SHA 断言。

三轮共 11 项通过，其中历史目标只有 8 项；没有相同原始故障环境下的完整 A/B，**不宣称已确证历史失败唯一根因**。只读独立审查：`independent-diagnostic-audit.md`。

本轮首个完整默认并发 gateway **955/2/0/968，exit 101** 仍保留于 `gateway-default-final.log`。两项为 split UTF-8/CRLF 与 64MiB base64 原字节测试；实际各多一个 14 字节合法 `: keep-alive\n\n` 前缀。长度分别 182/168 与 67,109,143/67,109,129；独立解析断言字符串后，剩余大图完整 SHA256 同为 `B09AB37283640FF9741E001B6550EE00403073BFBBE29BEEB6553ECA5FE61282`（大小写无关）。原巨型日志有 harness 输出交错，审查使用 JSON raw_decode 确定引号内断言边界，没有把引号外测试状态当载荷。

静态路径证据确认 exact-wire 夹具未持共享配置锁，而同进程其他 bridge 测试可锁内设置 5/10/15ms 心跳；reader 的超时出口允许完整帧到来前生成合法注释。没有失败时具体 setter/thread 捕获，**不声称识别了本次与哪一测试交错**。修复只隔离夹具未验证的生成心跳，真实 HTTP 心跳正反例和完整字节验证继续执行。修复后先定向验证，再在最终源码重跑完整 gateway 与 HTTP；详情见上表，不以首轮的其余通过抵消失败。

## 最终产物与 SDK 边界

最终 gateway 输出为全新 `target/s02-accepted-stream/`；最终 HTTP 输出为全新 `target/s02-accepted-http/`。失败完整运行的 `target/s02-final-stream/` 和修复定向输出 `target/s02-native-repair/` 保留，但不充当最终 SDK 输入。

- 流式：最终 reader 实际输出 `q02-0..13.sse` 和 14 份独立配对期望，官方 SDK 严格消费 **14** 份全部通过，未使用基线负控模式。
- 非流式：最终真实本地 HTTP 调用链输出完整原 **17** 份 case，官方 SDK 严格消费全部通过，不使用 `--allow-partial`。
- 公开流式 HTTP：`stream-http/` 下两份严格 case，官方 SDK 消费 **2** 份全部通过。

所有产物路径、文件名、大小、SHA256 存于 `validation-results.jsonl` 对应 freshFixtureRoot/fixtures。Rust 的 HTTP 检查实际经过本地生产网关路径；官方 SDK 检查使用 httpx MockTransport 回放这些结果，**SDK 没有直接连接 Rust 网关、真实账号或上游**。

## 最终边界与交接

- 本工作区 HEAD 保持 `f14bd735dca9203d7430958480b82ba7713641a9`；未暂存、提交、推送、PR、合入或发布。来源工作区与冻结输入只读。
- S02 范围没有剩余失败或忽略测试。定向过滤数量如上表，不代表完整 service/workspace 全量。最终源不因报告编辑改变。
- `cargo test --workspace -j 2 --no-fail-fast`、前端 runtime/build/build:desktop、独立 Tauri/Web shell 和 R01/S01/S02 统一版本验收由 S03 执行，本 S02 未执行这些命令，不能用来源或本轮范围结果替代。
- 未运行真实账号、真实数据库/上游图片生成、跨平台系统副作用或远端 CI；均为临时库、本地 mock、合成数据与假凭据。
- SSE 帧/共享存储预算约束指定帧存储记账，**不是进程 RSS 上限**；HTTP/chunk 队列、JSON 树、累计文本/工具状态、下游缓冲和分配器开销不在此预算内。
- 保留原协议边界：不新增 custom Chat/旧 function_call 契约或缺少必需上游字段的完整 schema 验证；原生 Responses 工具类型保留。
- 交接目录 `C:\Users\qxnm\.codex\worktrees\round3-tools-final\Codex-Manager`；本报告及 `target/s02-evidence/` 为可审查证据。最终进程检查另见 `process-inventory-final.json`，重型时段在确认进程结束后交还父会话，由父会话冻结交 S03。
