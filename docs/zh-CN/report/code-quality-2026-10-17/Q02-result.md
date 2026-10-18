# Q02：Chat 流式工具调用连续索引

## 状态与隔离范围

- 日期：2026-10-17。
- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。
- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\quality-q02\Codex-Manager`。
- 状态：**Q02 流式修复与范围内验证完成，可本地审查**；非流式 Chat tools 独立基线缺口见下文，不计为通过。未提交、推送、合并或发布。
- 创建前本会话 `list_artifacts` 返回空列表；以 `create_worktree(ref=<上述基线>, name=quality-q02)` 创建并等待 completed。开始修改前 HEAD 与基线一致，工作区干净，已阅读根 `AGENTS.md`。
- 原仓库仅用于只读获取未提交任务书；未修改原仓库总文档、产品代码、版本号、真实数据库、账号或生产服务。

## 根因与完整调用链

Chat 请求在 `gateway/local_validation/request.rs` 选择 `ChatCompletionsFromResponses` 适配器，`http_bridge/delivery.rs` 的流式分支构造 `ChatCompletionsFromResponsesSseReader`。该 reader 的 added、参数 delta/done、item.done 及 completed 补发，原来直接把 Responses 的 `output_index` 发为 Chat `tool_calls.index`。

Responses 的 output 同时包含 reasoning、message 和其他项目。工具位于 1、3、5 时，Chat 同样输出 1、3、5，使用数组下标聚合的客户端无法把首个工具追加到位置 0。官方 Python SDK 2.29.0 的 Chat stream helper 可稳定复现 `IndexError`。

补发还有两处相关缺口：已 added 的工具在 completed 被直接跳过，导致参数缺失；未收到 added、但已发出参数 delta/done 时，item.done 会重新发出完整参数。重复 added 还会重复拼接 ID、名称和参数。

## 改动

1. `crates/service/src/gateway/observability/http_bridge/stream_readers/chat_completions.rs`
   - 用以原始 `output_index` 为键的 `ChatToolCallState` 维护工具身份；首次遇到工具时分配 `0,1,…` 的 Chat index。非工具输出不占用 Chat 序号。
   - added、参数 delta/done、item.done、completed 共用该状态。completed 枚举位置仍然是上游身份键，不用连续 Chat index 查找上游项目。
   - 参数 delta 按原工具追加；快照只补发相对于已发送参数的缺失后缀，重复或旧快照不重新追加。名称与 call ID 各补发一次，支持参数先到、元数据后到。
   - 每个工具块都保留 `function` 对象（包括仅恢复 ID 的块），首块提供空 name/arguments 占位，兼容官方 SDK helper。
   - 不修改 Anthropic、图片、SSE pump、错误终止或 keepalive 实现。
2. `crates/service/src/gateway/observability/http_bridge/stream_readers/chat_completions_tool_tests.rs`
   - 新增连续数组客户端聚合断言，以及 reasoning/message 前置、混合输出、多工具参数交错、部分增量后 done/completed、done-only/completed-only、缺少 added、延迟元数据、ID-only 补发和重复快照回归。
   - 同一上游 wire 通过真实 Responses 非流式 SSE 聚合后，工具内容与流式 Chat 聚合结果比较；另比较 Chat 流式/非流式的文本和 reasoning。
   - 复用已有 `CM_REPAIR_FIXTURE_DIR` 测试导出机制，供官方 SDK 消费真实 reader 输出；不新增生产设置。
3. `.github/workflows/ci.yml`
   - 当前 CI 只筛选部分服务测试，补一个 `stream_readers::chat_completions` 定向回归步骤，不改发布流程。
4. `scripts/chat-tool-indices-sdk-check.py`
   - 使用官方 `client.chat.completions.stream()` 和 `httpx.MockTransport`，检查 reader 导出的工具 ID、名称、精确参数字符串、JSON 值与 finish reason；不访问真实服务或账号。

## 实际验证

所有命令均在本 worktree 根目录执行，Rust 使用本地 `target`，没有复用其他 worktree 的可写输出。构建并行度 `-j 2`，测试 `--test-threads=1`；临时 Python 依赖安装在 `target/q02-sdk-python`，不全局安装。

| 命令 / 阶段 | 实际结果 |
| --- | --- |
| 未修复实现：`cargo test -p codexmanager-service --lib tool_indices -j 2 -- --test-threads=1` | 退出 1；6 个有效用例失败，1 个可选导出测试空跑通过。失败包括工具下标 1/3/5 及重复 ID/name/arguments。此时只有测试接入，没有产品修复。 |
| 第一轮修复：相同 `tool_indices` 命令 | 退出 0；7/7（包含可选导出）。随后增加 ID-only 恢复和非流式对照，并再次执行最终回归，见下方。 |
| `cargo test -p codexmanager-service --lib chat_completions -j 2 -- --test-threads=1` | 退出 0；54/54；本轮含 ID-only 修复，最后新增的非流式对照尚未包含在此轮构建中。 |
| 最终代码：`cargo test -p codexmanager-service --lib http_bridge -j 2 -- --test-threads=1` | 退出 0；195/195；含全部 8 个新增测试、既有 Chat/Responses/Anthropic/image reader、流式/非流式转换、错误与 keepalive 回归。日志 `target/q02-http-bridge.log`。 |
| `cargo test -p codexmanager-service --lib gateway::upstream -j 2 -- --test-threads=1` | 退出 0；266/266；含 Chat 工具增量在上游终止前交付、Responses/WebSocket、取消、预检与协议适配回归。日志 `target/q02-upstream.log`。 |
| `cargo test -p codexmanager-service --lib gateway::request -j 2 -- --test-threads=1` | 退出 0；133/133；含 Chat tools 规范化、Responses function tools/tool_choice 和流式请求转换。日志 `target/q02-request.log`。 |
| `cargo test -p codexmanager-service --test gateway_logs hybrid_routing:: -j 2 -- --test-threads=1` | 退出 0；10/10；临时数据库 + mock HTTP 上游覆盖 Chat 工具流、Responses 流式/非流式路由与 tools 回退。日志 `target/q02-gateway-hybrid.log`。 |
| `cargo fmt --all -- --check` | 退出 0。 |
| `git diff --check` | 退出 0。 |
| 官方 SDK：基线 reader 导出 wire 普通模式 | 退出 1，`IndexError`；预期失败模式 `--expect-index-error` 退出 0。 |
| 官方 SDK：第一批修复 reader 导出 wire | 退出 0；12 个 fixture 全部通过，openai 2.29.0 / httpx 0.28.1。最终新增对照输出另行复核。 |
| 官方 SDK：正式脚本检查最终 reader 输出 | 退出 0；13/13 个 fixture，精确恢复所有工具元数据和参数，包含新增文本/reasoning 对照；日志 `target/q02-sdk-verified.log`。 |

最终代码相关 Rust 回归 195 + 266 + 133 + 10 项全部通过；这些筛选可能有部分重叠，不能据此推断全量 package/workspace 通过。新增测试 8 个，实际生成 13 个 SDK fixture。Rust lib test 编译报告 59 条 warning（首次基线构建也有相同数量），主要为 unused/dead_code；integration 的 lib 构建另有 1 条既有 unused_imports warning，没有为此扩大修改范围。

upstream/request/integration 回归的 `TEMP`/`TMP` 均显式设为本 worktree 的 `target/q02-runtime-tmp`；测试隔离环境并使用临时数据库/mock。测试进程最终退出 0；检查本 worktree 路径下无遗留 `gateway_logs` 进程。未启动持续运行的手工服务。保留 `target/` 下合成 fixture、日志、临时依赖和测试数据作为本地复核材料，无真实账号数据。

### SDK 与 fixture 复现命令

下面使用本次实际 Python 路径，环境变量只影响执行命令的进程。`CM_REPAIR_FIXTURE_DIR` 在最终 HTTP bridge 测试中设为 `target/q02-sdk-verified`，实际导出 13 对文件；完整日志和临时依赖均位于本 worktree 的忽略目录 `target/`，不作为提交内容。

```powershell
$q02Python = 'C:\Users\qxnm\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$q02SdkPath = Join-Path (Get-Location) 'target/q02-sdk-python'
$env:TEMP = Join-Path $q02SdkPath 'tmp'
$env:TMP = $env:TEMP
$env:PYTHONDONTWRITEBYTECODE = '1'
& $q02Python -m pip install --disable-pip-version-check --no-cache-dir --no-compile --target $q02SdkPath 'openai==2.29.0'
# 上述安装实际退出 0；版本 openai 2.29.0、httpx 0.28.1。

$env:CM_REPAIR_FIXTURE_DIR = Join-Path (Get-Location) 'target/q02-sdk-verified'
cargo test -p codexmanager-service --lib http_bridge -j 2 -- --test-threads=1
# 实际退出 0，195/195。

$env:PYTHONPATH = $q02SdkPath
& $q02Python -B scripts/chat-tool-indices-sdk-check.py target/q02-sdk-verified
# 实际退出 0，13/13。
& $q02Python -B scripts/chat-tool-indices-sdk-check.py target/q02-baseline-sdk --expect-index-error
# 实际退出 0，确认未修复 reader 输出触发 SDK IndexError。
```

`target/q02-baseline-sdk/q02-tools.sse` 是在产品实现未修改时由新增测试导出的真实 reader wire；旧可选导出测试随后合并为所有 fixture 自动导出，基线文件仅作本地负对照证据，不要求最终代码重新产生错误输出。普通模式的预期失败曾实际退出 1；正式脚本的预期失败模式再次确认异常，日志 `target/q02-sdk-baseline-check.log`。

## 非流式对照与范围限制

- 已覆盖：同 wire 的 Responses 非流式工具输出与 Chat stream 聚合结果；Chat 文本和 reasoning 的流式/非流式对照；既有 tools 和两种 OpenAI 风格端点相关回归。
- **基线独立缺口**：`http_bridge/body_conversion.rs::convert_responses_body_to_chat_completions` 当前没有生成 `message.tool_calls`，finish_reason 仍是 stop。因此，非流式 Chat 的工具调用与流式 Chat 的完整一致性不能报通过。本任务未改该 converter，也没有添加把缺失行为固化为正确结果的断言。
- 非流式 Chat 的实际链为 `delivery.rs` 非流式分支 → `collect_non_stream_json_from_sse_bytes` → `convert_success_body_for_adapter` → 上述 converter。聚合层能保留工具，丢失发生在后续 Chat JSON 转换。这是独立基线问题，不是 Q07 的 SSE 字节预算任务。
- 没有向真实上游、真实账号或生产服务发送请求；SDK 验证的输入是本次 Rust reader 实际生成的 SSE，HTTP 传输完全 mock。

## 未执行项与整合风险

- `cargo test --workspace -j 2`：未执行；本次改动局限 Chat reader，按根规则选用相关服务 package 回归。局部通过不等于组合版本验收。
- `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2`：未执行；无桌面 Rust 改动，且 Tauri 被根 workspace 排除。未来桌面整合验证必须单独运行此 manifest，不能用根 workspace 结果代替。
- 前端/Web 构建、生产运行验证及真实上游 SDK 端到端：未执行，无对应改动或生产授权。
- GitHub Actions：仅补充定向步骤，未推送/远程执行。对应本地 reader 测试已由最终 HTTP bridge 回归覆盖；远程 CI 不能报通过。
- Q02 与 Q07 都涉及流读取，建议按任务书先 Q02 后 Q07，解决 reader 重叠后重跑公共 reader、HTTP bridge、upstream 和两端点/tools 回归，保留基线已有 Claude 流适配修复。
- Chat 序号按工具首次出现顺序分配且不重新编号；快照必须是已发参数的延长或重复。对已发参数的矛盾重写无法通过 Chat 追加协议撤回，本次不扩展为新的协议错误机制。
- 本工作区仅提供可审查本地改动；未自行合入任何其他任务成果。
- 最终 HEAD 仍为固定基线；改动仅为上述 4 个实现/测试/CI/SDK 文件与本报告，共 5 个文件。未暂存或创建提交。
