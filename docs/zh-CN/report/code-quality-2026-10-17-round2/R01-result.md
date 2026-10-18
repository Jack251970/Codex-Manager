# R01：重试与超时组合验收

## 交付状态与范围

- 日期：2026-10-17。状态：**Q01+Q06 整合、补漏及本任务要求的范围验证完成，可供审查；尚未合入主仓库**。最终 frontend runtime 319/319、Web 40/40、独立 Tauri RPC 19/19 + 3/3，build/build:desktop、eslint、rustfmt、diff 检查通过。浏览器完整端到端与超时语义边界见末节。
- 本会话独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\round2-retry-timeout\Codex-Manager`。
- Git 基线及交付时 HEAD：`f14bd735dca9203d7430958480b82ba7713641a9`。
- 编辑前调用 `list_artifacts`，返回空；通过 `create_worktree(ref=上述 SHA, name="round2-retry-timeout", allowAsync=true)` 创建，operation `e87d520b-cd15-40c8-9476-9bd50909cbcc` completed 后读取根及 `apps/AGENTS.md`，核查初始 HEAD 一致、状态干净。
- 原仓库仅只读读取两份任务书；冻结输入只读，未读取第一轮来源或其他会话正在变化的工作树。所有代码、测试、报告、依赖和构建输出均在本工作区。
- 保留可审查的未暂存本地改动；未提交、推送、PR、合入主仓库、发布或变更版本。未连接真实账号、数据库、生产服务或上游。

## 冻结输入与整合

输入根：`C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round2-3e2657e8`。

| 输入 | manifest SHA256 | tracked.patch SHA256 | 文件核验 |
| --- | --- | --- | --- |
| Q01 | `3ACAA272269E242D3AF4793CB962B51C017E8FB0376D52DD26B39AA2FF95A35E` | `2F716421A9F75044BB464D3C62D2606B0FFF8D9029D09D6409F28FD1F352E041` | 11 tracked + 6 untracked 全部匹配 |
| Q06 | `21569D4FDC83DAED3C3D956733F10C7CF003F2B9DE935DE1A25FE6406C04FE63` | `89AB7B06FD2663B18CB9A942137E02981B28A2B654312EC1E78F7213F6B3D169` | 5 tracked + 2 untracked 全部匹配 |

两份 manifest 均为同一基线，无删除项。24 项输入文件对应 22 个唯一路径。完整逐文件输入与整合后 SHA256 记录：`target/r01-validation/input-verification.json`。

1. Q01 `git apply --check`、`git apply` 均退出 0，复制 manifest 中全部新增文件。
2. Q06 完整 `git apply --check` 退出 1，仅冲突于 `apps/src/lib/api/rpc-http.ts` 和 `apps/tests/rpc-http.test.mjs`，原始输出保留 `target/r01-validation/q06-full-check.log`。排除这两文件后 check/apply 均退出 0，全部新增文件同样导入。
3. 用 `git show <base>:<path>` 保存共同基线，逐处比较基线、当前 Q01 文件、Q06 `tracked/` 文件。RPC 入口保留 Q01 方法白名单、安全重试上限、未知结果标识、禁止重定向及完整 envelope 校验，同时接入 Q06 四参 reader；成功 JSON、错误文本/JSON 都在同一次尝试的 deadline 和父取消内读取。没有用第二份文件整份覆盖第一份。
4. RPC 测试保留 Q01 的共享策略加载及全部安全重试案例，接入 Q06 四参 fetcher、取消/迟到结果/错误体测试，再增加组合案例。Q06 的模拟查询名 `demo/read` 不在 Q01 白名单，相关只读用例改用经过审核的 `apikey/list`，没有把未知方法开放重试。
5. 第一轮 Q01/Q06 报告原样导入 `docs/zh-CN/report/code-quality-2026-10-17/`，仅作为来源证据；以下日志全部来自本组合工作区重新运行。

## 组合修复与新增增量

- 写入重试上限在传入调用者选项后强制应用，成功/错误 body 读取、deadline、用户取消均不能突破 `retries: 0`。307/308 不在 fetch 或代理内隐式回放。
- 已发送写入读取失败继续带 `rpc_result_unknown:`，并保留原始 `cause`；deadline 明确为 `TimeoutError`，用户取消为 `AbortError`。已取消、序列化失败、桌面明确连接前失败，与发送后未知结果分开处理。浏览器 fetch 无法证明网络 TypeError 发生在发送前，因此保守未知。
- 复核发现并修复两个排队窗口：立即取消时 fetch 尚未调用却误标 unknown；主线程同步繁忙超过 deadline 时，microtask 可能在 timer 之前发送过期写入。请求控制层现用仅内部记录的错误来源证明“本次所有尝试从未调用 fetch”，RPC 保留其确定失败；实际 fetch 前再次检查单调时钟。两类回归均先失败后通过，发送数严格为 0。没有根据网络异常文案猜未发送。
- 完整 HTTP 200 RPC 错误/业务错误在读取、校验完成后解包，保持确定业务错误，不标记未知，也不自动重试。非 2xx 仍按 Q01 保守未知；未擅自把代理/网关错误当成服务端业务未执行的证明。
- 错误体读取失败不吞取消、不尝试第二次消费 body；登录跳转和业务解包只发生在受控请求完成并复查父取消之后。迟到 reader/IPC 结果不能写消费方缓存或触发后续写请求。
- Q06 的 `runWithAbortSignal` 延后到 microtask 调用 IPC，暴露 Q01 旧测试的时序假设。改为明确等待 IPC 启动后取消，并新增调度前取消对照与迟到成功不得写缓存断言，未用 sleep 猜调用先后。
- 新增 Node 真实 HTTP：写入提交后 body 停滞/取消，200、401 各覆盖；只读 body deadline 后恢复，确认前一次 signal abort 在第二次 send 之前。另有受控 stream 检查 cancel 已调用、前次 signal 已中止后才开始下一次；不等待不合作 cancel promise 无限结算。
- 新增 Rust：桌面创建/resetCredit 在仅发 headers 后超时、连接释放、主址与 fallback 无重放；Web 代理提交后断连、测试注入 body timeout、取消 proxy future 的连接释放对照。

相对两个输入快照的额外修改仅为以下 9 个已导入文件及本报告：

| 文件 | 额外变化 |
| --- | --- |
| `apps/src/lib/api/rpc-http.ts` | 三方整合生命周期、策略与解码后取消检查 |
| `apps/src/lib/api/rpc-retry-policy.ts` | unknown 错误保留 cause 及取消/超时类型 |
| `apps/src/lib/api/transport.ts` | 桌面启动后取消 unknown 保留 cause |
| `apps/src/lib/utils/request.ts` | deadline 与用户取消区分、明确未发送标识、实际发送前 deadline 检查 |
| `apps/tests/rpc-http.test.mjs` | 双方用例合并，新增 8 个组合回归 |
| `apps/tests/request-utils.test.mjs` | 4 项 deadline 断言明确要求 TimeoutError，其余关键断言保持 |
| `apps/tests/transport-retry.test.mjs` | 启动握手、调度前取消及迟到 IPC 结果断言 |
| `apps/src-tauri/src/rpc_client/transport_retry_tests.rs` | 创建/resetCredit headers-only 对照 |
| `crates/web/src/service_gateway_tests.rs` | 3 个代理异常与资源释放回归 |

新测试进入既有入口：Node `node --test` 自动发现现有文件；Rust 已注册 `rpc_client::transport_retry_tests` 和 Web `service_gateway_tests`。未复制平行生产实现，未修改 CI/发布流程。额外基线对照脚本和原始日志仅保留在忽略的 `target/r01-validation/`，不属于产品增量。

## 实际发送次数及证据层次

| 场景 | 入口/条件 | 发送或提交次数 | 结果 |
| --- | --- | --- | --- |
| 固定基线创建提交后丢响应 | 基线源码 + Node 真实 HTTP，retries=3 | 4 | 对照确认旧重放缺陷 |
| 组合创建提交后丢响应 | 生产 postJsonRpc + fetchWithRetry + Node HTTP；调用者 retries=3/自定义策略 | 1 | unknown |
| 创建后仅响应 headers，body 停滞 | Node HTTP 200/401，调用者 retries=3 | 每例 1 | TimeoutError + unknown，无登录跳转 |
| headers 后用户取消 | Node HTTP 200/401，调用者 retries=3 | 每例 1 | AbortError + unknown，无登录跳转 |
| resetCredit / 未知写入丢响应 | Node HTTP，调用者 retries=3 | 每例 1 | unknown |
| 无效 JSON/envelope、截断体、HTTP 503 | Node HTTP 写入 | 每例 1 | unknown |
| 浏览器请求 307/308 | Node fetch 模拟浏览器语义，真实 HTTP | 原请求 1 / 重定向 0 | unknown |
| 只读/已审核幂等写瞬时失败 | Node HTTP，先断连再503再成功 | 每方法 3 | 恢复成功 |
| 只读持续503 | Node HTTP，默认 retries=3 | 4 | 有限失败 |
| 只读成功体停滞后恢复 | Node HTTP；retries=1 | 2 | abort-first 位于第二次 send 前 |
| 只读不合作 reader 超时后恢复 | 受控 Response；retries=1 | 2 | 前次 abort 与 cancel 已发生 |
| 完整业务错误 | 生产解包器 + 完整 Response | 1 | 原业务错误，无 unknown |
| 发送前取消/序列化失败 | 生产入口 + 发送断言 | 0 | 原明确失败 |
| 排队取消/主线程阻塞至deadline后 | 生产 postJsonRpc/fetchWithRetry；retries=3 | 每例 0 | 确定 AbortError / TimeoutError，无 unknown |
| 桌面外层写入/未知命令 | 生产 transport/runWithControl + mock IPC；retries=3 | 每例 IPC 1 | 不重放、unknown 不触发别名回退 |
| 桌面外层只读/幂等写 | 同上，首次临时失败 | 每例 IPC 2 | 恢复成功 |
| 桌面外层调度前取消 / 启动后取消并迟到成功 | 同上 | IPC 0 / 1 | 明确取消 / unknown；迟到值不写缓存 |
| 桌面内层创建/resetCredit headers-only | 真实 TCP mock，含 fallback 与 retries=99 参数 | 每方法主址 1 / fallback 0 | unknown、连接释放及专用提示通过 |
| 桌面内层只读/幂等写丢响应 | 真实 TCP mock，两个有限尝试 | 每方法 2 | 第二次成功；持续失败对照恰好2次 |
| 桌面内层明确连接前失败后切换地址 | 真实 TCP mock | 失败地址业务发送0 / fallback业务发送1 | 写入可安全回退，不标unknown |
| Web 代理丢响应/headers-body超时/取消future | 实际 rpc_proxy + mock HTTP 上游 | 每例 1 | 502/502/取消；释放连接 |
| Web 代理307/308 | 实际 rpc_proxy + mock HTTP 上游 | 原请求 1 / 重定向 0 | 原样返回3xx，不回放POST |

桌面内外层分别验证：mock IPC 调用数不是实际前端→Tauri IPC→Rust TCP 的整条联动发送数。Web 代理测试调用实际 handler 和真实上游 HTTP，但未运行浏览器到 Web 监听端口的完整部署链路。

## 实际验证命令与日志

所有命令在本工作区根执行。日志根绝对路径：`C:\Users\qxnm\.codex\worktrees\round2-retry-timeout\Codex-Manager\target\r01-validation`。测试失败记录均保留。Rust 显式设置 `CARGO_TARGET_DIR=<本工作区>/target`，每次 `-j 2`，Web 与桌面串行；每次桌面命令显式设置临时 mock `CODEXMANAGER_RPC_TOKEN`、`CODEXMANAGER_DB_PATH=<本工作区>/target/r01-validation/desktop-temporary.sqlite`，最终 Web 复验同样指向本工作区临时数据库路径。没有读真实凭据或用户库。前端构建设置 `NODE_OPTIONS=--max-old-space-size=4096`、`CIRCLE_NODE_TOTAL=2`、`RAYON_NUM_THREADS=2`、`UV_THREADPOOL_SIZE=2`，实际使用 1 worker。

| 实际命令 | 退出码 / 数量 | 日志 |
| --- | --- | --- |
| `pnpm.cmd -C apps install --frozen-lockfile --network-concurrency 4 --child-concurrency 2` | 0；lockfile 未修改 | `install.log` |
| `node --test --test-name-pattern='committed write with HTTP\|read-only stalled body' apps/tests/rpc-http.test.mjs`（整合前） | 1；0 passed / 5 failed / 0 skipped | `overlap-before.log` |
| 同上（RPC/错误分类整合后） | 0；5 passed / 0 failed / 0 skipped | `overlap-after.log` |
| `node --test --test-concurrency=2 apps/tests/rpc-http.test.mjs apps/tests/request-utils.test.mjs apps/tests/transport-retry.test.mjs apps/tests/transport-runtime.test.mjs`（首次） | 1；62 passed / 7 failed / 0 skipped | `targeted-first.log` |
| 同上（修复测试契约与时序后） | 0；70 passed / 0 failed / 0 skipped | `targeted-final.log` |
| `node target/r01-validation/baseline-control.mjs` | 0；1 passed / 0 failed；实际提交4次 | `baseline-control.log` |
| `pnpm.cmd -C apps run test:runtime --test-concurrency=2`（初次完整） | 0；316 passed / 0 failed / 0 skipped | `runtime.log` |
| 同上（追加真实 HTTP 只读释放顺序后最终完整） | 0；317 passed / 0 failed / 0 skipped | `runtime-final.log` |
| `node --test --test-name-pattern='write rejected before actual fetch' apps/tests/rpc-http.test.mjs`（发送前窗口修复前 / 后） | 1：0 passed / 2 failed；0：2 passed / 0 failed；均0 skipped | `presend-before.log` / `presend-after.log` |
| `pnpm.cmd -C apps run test:runtime --test-concurrency=2`（所有增量后的最终完整） | 0；319 passed / 0 failed / 0 skipped | `runtime-presend-final.log` |
| `pnpm.cmd -C apps run build` | 0；类型检查、19/19 静态页面 | `build.log` |
| `pnpm.cmd -C apps run build:desktop` | 0；类型检查、19/19 静态页面 | `build-desktop.log` |
| `pnpm.cmd -C apps run build`（发送前窗口修复后） | 0；类型检查、19/19 静态页面 | `build-final.log` |
| `pnpm.cmd -C apps run build:desktop`（发送前窗口修复后） | 0；类型检查、19/19 静态页面 | `build-desktop-final.log` |
| `pnpm.cmd -C apps exec eslint src/lib/api/rpc-http.ts src/lib/api/rpc-retry-policy.ts src/lib/api/transport.ts src/lib/api/transport-errors.ts src/lib/api/transport-runtime.ts src/lib/utils/request.ts tests/rpc-http.test.mjs tests/request-utils.test.mjs tests/transport-retry.test.mjs tests/transport-errors.test.mjs tests/transport-runtime.test.mjs` | 0 | `eslint.log` |
| 同上（发送前窗口修复后） | 0 | `eslint-final.log` |
| `cargo test -p codexmanager-web -j 2 -- --test-threads=2` | 0；40 passed / 0 failed / 0 ignored | `web.log` |
| 同上（最终前端静态导出后复验） | 0；40 passed / 0 failed / 0 ignored | `web-final.log` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 rpc_client:: -- --test-threads=2`（首次） | 101；18 passed / 1 failed / 0 ignored / 70 filtered | `desktop-rpc.log` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 non_idempotent_writes_do_not_replay_or_keep_socket_after_headers_only_timeout -- --test-threads=2`（修复mock模式后） | 0；1 passed / 0 failed / 0 ignored / 88 filtered | `desktop-headers-only.log` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 rpc_client:: -- --test-threads=2`（最终） | 0；19 passed / 0 failed / 0 ignored / 70 filtered | `desktop-rpc-final.log` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 rpc_call_ -- --test-threads=2` | 0；3 passed / 0 failed / 0 ignored / 86 filtered | `desktop-rpc-call.log` |
| `rustfmt --edition 2021 --check --config skip_children=true apps/src-tauri/src/rpc_client/mod.rs apps/src-tauri/src/rpc_client/retry_policy.rs apps/src-tauri/src/rpc_client/transport.rs apps/src-tauri/src/rpc_client/transport_retry_tests.rs apps/src-tauri/src/tests/lib_tests.rs crates/web/src/main.rs crates/web/src/service_gateway.rs crates/web/src/service_gateway_tests.rs` | 0 | `rustfmt.log` |
| 同上（Windows mock 模式修复后） | 0 | `rustfmt-final.log` |
| `git diff --check` | 0；仅Windows行尾提示 | `diff-check.log` |

首次 5 项失败证明组合接口/控制缺口：旧三参 RPC 不执行 reader，无法满足 body 控制和分类，新只读释放顺序用例触发外部测试界限。修复后通过。首次定向 7 项失败为 6 项 deadline 仍断言 AbortError，加上旧桌面“启动后取消”测试实际在 microtask 调用前取消；改为更精确的 TimeoutError 和握手/0次发送对照，没有放宽 deadline、发送数或关键业务断言。没有新增 skip/ignore，没有无上限重试。

桌面首次唯一失败是新增 mock 夹具的 Windows socket 模式：非阻塞 listener 接受的 stream 继承非阻塞模式，观察 EOF 时立即返回 `WouldBlock`，没有等待预设 read timeout。修复为接受后显式 `set_nonblocking(false)`，保持 250ms 产品测试 timeout、2秒观察界限、提交数/连接释放断言不变。先定向通过，再完整 RPC 19项通过。不是把 WouldBlock 当成功，也未扩大超时或删除失败日志。首次 Web 冷编译 5分10秒，首次桌面冷编译 6分36秒；service 的既有 unused-import 警告没有修改，构建退出码按真实结果记录。

交付范围核对：23 个预期变更路径（输入并集22个 + 本报告），无额外产品文件；暂存区0项，HEAD仍为共同基线。依赖锁文件未改。`input-verification.json` 保留冻结输入重验、最终代码/测试 SHA 与变更路径集合。所有本任务启动的测试进程均已退出，本地 mock 连接/服务由测试释放，未留下运行中的服务。

## 未验证项与汇总边界

- 未运行完整浏览器 UI、桌面窗口或真实 Tauri IPC 端到端；没有把 Node fetch 或 handler 测试称为浏览器完整端到端。`pnpm.cmd -C apps run test:navigation` 未执行：本次不改页面导航/缓存，只改传输基础设施，已运行完整 runtime 与两种构建。
- Web 代理生产客户端目前没有 body 总 deadline；新增代理超时测试向客户端注入 250ms 测试 deadline，仅证明 body 错误映射、不重放与释放。取消测试直接 drop/abort Rust future，不证明真实浏览器断连一定传播取消到该 future。上述限制不能用 Q06 前端通过掩盖。
- 桌面 `TcpStream` timeout 是单次读写等待限制，resetCredit 保留 600 秒专用 IO 超时和状态核实提示；本轮不宣称 native 慢速持续字节具有绝对整包总 deadline，也不宣称前端取消能中止已开始的原生业务。
- reader 只读取/解码；同步 JSON 解析不能中途抢占，仅在解析完成后拒绝过期值。raw Response 模式把所有权交给流式调用者；两个既有 EventSource 消费方未改。
- runtime 缓存测试采用 Q06 的缩短 deadline 测试配置，验证迟到响应不能覆盖 fallback；不宣称实测了生产 2500ms × 3 次的等待时间。
- `cargo test --workspace -j 2 --no-fail-fast` 未执行：本任务仅修改前端、Web代理、桌面RPC，按根规则选择 Web 包及独立 Tauri RPC 范围；没有以根测试替代 Tauri，也不继承 Q03/R05 的全量结果。根 workspace 不含 Tauri。
- 旧 `scripts/tests/web_runtime_probe*.ps1`、`web_ui_smoke.ps1` 在基线已移除（`docs/zh-CN/TESTING.md`）；未把不存在的脚本计为通过。未执行生产验收脚本、真实上游充值或账号操作。
- Q08 清理拒绝的17个文件不在本轮范围；未读取/清理来源遗留产物。本任务 mock 和临时转译资源由测试关闭；依赖、构建输出和本轮日志保留。
- 这份组合仅包含 Q01+Q06，不等于五组独立结果形成统一验收版本。后续统一整合仍需保留 R04→R03、R05→R02 的明确交接与共享文件冲突处理，不能凭本报告宣告全部任务已合入或全量通过。
