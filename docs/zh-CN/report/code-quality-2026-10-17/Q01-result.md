# Q01：RPC 写操作重试安全

## 交付状态与边界

- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。
- 本会话独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\quality-q01\Codex-Manager`。
- 状态：**本地修复及要求的隔离验证完成，可供整合审查，尚未合入主仓库**；未提交、推送、合并或创建 PR，未变更版本。
- 原仓库仅用于读取任务书。未修改原仓库总文档、产品代码、真实账号或生产数据库/服务。

## 根因及完整调用链核查

Web 的 `fetchWithRetry` 默认允许初次请求加三次重试，而原 `postJsonRpc` 直接转交调用方选项，创建密钥没有统一保护。Tauri 除 Rust 内部跨地址及第二轮重试外，前端外层 `runWithControl` 也接受调用方重试配置。原 Rust 仅保护 `account/usage/resetCredit/consume`，其他写入在发送后读取失败时仍会重放。

服务端当前两种分发都存在提交先于响应的窗口，不能假设丢响应代表未执行：

- 同步 SQLite：`crates/service/src/rpc_dispatch/apikey.rs` → `apikey/apikey_create.rs`，每次生成新 key/id，写入后才返回。
- HTTP 注入存储：`rpc_dispatch/storage_async.rs` → `apikey/injected.rs`，生成新 ID 后等待 `storage.create_api_key`，再返回。
- 异步 SQLite 适配器最终进入 `crates/core/src/storage/api_keys.rs` 的事务提交；SeaORM 路径在 `crates/storage-seaorm/src/api_key_details.rs` 的 `create_with_owner` 提交。
- `http/rpc_endpoint.rs` 等待分发返回后才序列化 HTTP 响应。以上属于源码核查，未运行真实 MySQL/PostgreSQL。
- Web 代理 `crates/web/src/service_gateway.rs` 只有一次显式 `send`，响应头/体读取失败返回 502。依赖默认协议安全重试不等于业务失败重试，但重定向需要额外禁止 POST 回放。

## 改动与策略

1. 新增 `apps/src/lib/api/rpc-retry-policy.json` 作为 Web 与桌面共同消费的精确白名单；TypeScript 导入，Rust `include_str!` 编译包含。分为 62 个只读方法、两个已核实幂等写入（`apikey/enable`、`apikey/disable`）。其他方法，包括未来新增方法，默认不可重放。新增方法须核查同步 SQLite 与异步存储实现。
2. 不按 get/read/set 等名称推断安全。`account/read`、`account/usage/resetCredits` 可刷新令牌，`initialize` 写事件，`account/login/status` 处理过期会话，`codexProfile/get` 可能迁移配置，均保守排除。`appSettings/get`、`authorContent/get` 在基线同步运行态且可迁移设置/清调度状态，也排除，不依赖 Q04 后续修复；`startup/snapshot` 的已核查链不调用这些 setter，保留只读。设置写入、钱包充值、创建任务/密钥、刷新/测速等同样不自动重放。
3. Web 在调用方选项之后强制非幂等 `retries: 0`；网络错误、HTTP 异常、损坏或缺失的 JSON/RPC 响应返回 `rpc_result_unknown:` 和“先刷新核实”提示。完整业务错误保持原消息；已取消和序列化失败这类明确发送前错误保持原类型。
4. Tauri 前端外层同样限制 `runWithControl`，避免重试整条 IPC 绕过 Rust；调用开始后的取消提示结果未知。未知命令也采用保守上限。
5. Rust 仅在明确连接前失败时继续跨 socket 回退；写入或读取错误、超时、HTTP/JSON 解析失败后，非幂等立即停止。只读和明确幂等写保留原有有限重试；resetCredit consume 的长超时及专用状态核实提示保留。
6. `rpc_result_unknown:` 不会被通用超时文案归一化吞掉，也不会被 `invokeFirst` 因嵌套“not found”误判为命令缺失并调用别名。明确命令缺失的安全回退保留。
7. RPC 浏览器请求设置 `redirect: "error"`，防止 307/308 在 fetch 内部重发 POST。Web Fetch API 无法证明 TypeError 是发送前失败，因此写入不按异常文字猜测安全性；桌面可精确识别连接失败并安全回退。
8. Web 壳的本地服务代理客户端同样设置 `redirect::Policy::none()`，禁止代理内隐式重发 POST；保留服务端返回的 3xx 状态。共享代理客户端还用于作者内容、事件流与网关固定地址转发，也停止自动跟随 3xx；正常服务路由没有重定向。客户端初始化失败时不回退到允许重定向的默认客户端。

没有新增服务端幂等键或修改密钥存储业务。现有 resetCredit operationId 机制保持不变。本任务没有改写 `utils/request.ts` 的超时生命周期。

## 实际验证

所有运行命令的工作目录均为本 worktree（前端命令注明 apps）。Rust 限制 `-j 2`，使用本 worktree 自己的输出目录；HTTP 测试仅监听临时 loopback 端口，使用内存模拟提交，退出时关闭。

| 命令 | 实际结果 |
| --- | --- |
| `node docs/zh-CN/report/code-quality-2026-10-17/validation/baseline-retry-probe.mjs` | 退出 0；只读提取固定基线的传输模块，mock 提交后断开，实际记录 4 次创建（首次 + 3 次重试），证实基线缺陷；修复后对应回归只记录 1 次 |
| `pnpm.cmd -C apps install --frozen-lockfile --network-concurrency 4 --child-concurrency 2` | 退出 0，锁文件未改变 |
| apps：`node --test --test-concurrency=2 tests/rpc-http.test.mjs tests/request-utils.test.mjs tests/transport-errors.test.mjs tests/reset-credit-client.test.mjs` | 退出 0，35/35（新增最终回归之前的定向验证） |
| apps：`node --test --test-concurrency=2 tests/transport-retry.test.mjs tests/rpc-http.test.mjs tests/transport-errors.test.mjs` | 首次退出 1：新测试加载器扩展名重复；修复后退出 0，30/30；不属于产品运行失败 |
| apps：`pnpm.cmd run test:runtime -- --test-concurrency=2` | 退出 1，pnpm 把 `--` 作为测试路径传递；更正为下一行 |
| apps：`pnpm.cmd run test:runtime --test-concurrency=2` | 退出 0，272/272，0 skipped（含最终重定向/序列化回归） |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 rpc_client:: -- --test-threads=2` | 退出 0，18 passed / 0 failed / 70 filtered；最终 62 项白名单后再次通过，含原 resetCredit 保护和隐藏写入分类断言 |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 rpc_call_ -- --test-threads=2` | 退出 0，3 passed / 0 failed / 85 filtered；最终策略后再次通过，慢响应、chunked、只读 socket fallback |
| `cargo test -p codexmanager-web -j 2 -- --test-threads=2` | 退出 0，37 passed / 0 failed / 0 ignored；最终静态 UI 构建后重跑同样 37/37，含代理 307/308 不重放、WebSocket 与嵌入 UI |
| `pnpm.cmd -C apps run build` | 退出 0，TypeScript 和静态导出 19/19；`NODE_OPTIONS=--max-old-space-size=4096`、`CIRCLE_NODE_TOTAL=2`，实际 1 worker |
| `pnpm.cmd -C apps run build:desktop` | 退出 0，TypeScript 和静态导出 19/19，同上构建限制 |
| `pnpm.cmd -C apps exec eslint src/lib/api/rpc-http.ts src/lib/api/rpc-retry-policy.ts src/lib/api/transport.ts src/lib/api/transport-errors.ts tests/rpc-http.test.mjs tests/transport-retry.test.mjs tests/transport-errors.test.mjs` | 首次退出 1，新测试变量名 `module` 触发 Next 规则；更名后退出 0 |
| `git diff --check` | 退出 0，仅 Windows LF/CRLF 提示 |
| `rustfmt --edition 2021 --check --config skip_children=true apps/src-tauri/src/rpc_client/mod.rs apps/src-tauri/src/rpc_client/retry_policy.rs apps/src-tauri/src/rpc_client/transport.rs apps/src-tauri/src/rpc_client/transport_retry_tests.rs apps/src-tauri/src/tests/lib_tests.rs crates/web/src/main.rs crates/web/src/service_gateway.rs crates/web/src/service_gateway_tests.rs` | 退出 0；独立复核曾裸调用 rustfmt 默认 edition 2015 而拒绝 async，使用 manifest 一致的 2021 后通过 |

原始最终运行日志位于同目录 `validation/`。桌面测试进程显式使用 mock RPC token 和本 worktree 内临时 DB 路径；不读取真实账号库。根 `.cargo/config.toml` 指定输出为本 worktree 的 `target/`。首次桌面定向冷编译约 10 分 33 秒，测试 0.57 秒；保留基线 service 的 unused-import 警告，没有将其当作失败。

资源清理：所有本任务 mock HTTP/TCP 服务和 Cargo 测试进程均已结束；Web 测试生成的临时 SQLite/WAL/SHM 及模型目录锁文件已精确清理，桌面临时 DB 未留下文件。依赖、构建产物和验证日志留在本 worktree 以供复核。

变更文件：`apps/src/lib/api/{rpc-retry-policy.json,rpc-retry-policy.ts,rpc-http.ts,transport.ts,transport-errors.ts}`；`apps/tests/{rpc-http.test.mjs,transport-retry.test.mjs,transport-errors.test.mjs}`；`apps/src-tauri/src/rpc_client/{mod.rs,retry_policy.rs,transport.rs,transport_retry_tests.rs}`；`apps/src-tauri/src/tests/lib_tests.rs`；`crates/web/src/{main.rs,service_gateway.rs,service_gateway_tests.rs}`；本报告和验证日志。

覆盖内容：模拟创建已提交后断连接；HTTP 503、截断响应体、无效 JSON/envelope；resetCredit consume、未知方法；调用方显式重试/shouldRetry 配置；只读及幂等写瞬时错误重试、次数有界；桌面连接前安全回退；外层 IPC 和别名回退；取消前后区分；浏览器及 Web 代理两层 307/308 不重放。模拟提交仅用于证明传输层请求次数，真实存储提交链另外以源码核查说明，不将其冒充真实业务端到端验证。

## 未验证项与整合风险

- 未连接真实账号、生产服务、MySQL 或 PostgreSQL；未执行真实上游扣减。服务端业务未修改。
- 未启动完整桌面窗口或进行交互式浏览器 UI 验收；native TCP 与 Web Node HTTP 回归覆盖传输行为。
- 没有发现适用于本任务的独立本地 RPC 重试 probe 脚本，使用新增 mock 回归；`scripts/production-acceptance.ps1` 属于生产配置验收，不在本任务授权范围，未执行。
- `cargo test --workspace -j 2` 未执行：core/service 业务未改，本次以 Web 包及独立 Tauri manifest 相关测试验证隔离改动；组合版本全量验收应在任务整合后另行执行，不能用根 workspace 代替 Tauri。
- Q01/Q06 都涉及 `rpc-http.ts`、`transport.ts` 及 request/runtime 测试；建议先整合 Q01，再整合 Q06，保留安全上限、未知结果 marker 与禁止重定向规则，并重跑完整 runtime/transport/native 回归。
- 完整响应体 deadline、取消监听、重试前释放响应体归 Q06；本任务仅在读取实际失败时分类未知结果，没有宣称修复 headers 后 body 永不完成的问题。
- 新增 RPC 方法默认为不可重放，可能降低未审核查询的自动恢复能力；应在核查副作用后加入共享白名单，不能通过调用方 `retries` 覆盖。
- 本独立工作区通过不代表与其他质量任务组合后通过，更不代表已合入主仓库。
