# Q04：设置读取不重置账号调度

## 状态与基线

- 状态：**本地实现与定向验证完成**；新增回归、设置集成与 HTTP 调度回归均通过。未提交、推送、合并、创建 PR 或发布。
- 日期：2026-10-17。
- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。
- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\quality-q04\Codex-Manager`。创建前 `list_artifacts` 返回空，按指定 ref 创建，初始工作区干净。
- 已只读读取原仓库任务书的执行约定、Q04 和交付与整合顺序；未改原仓库产品代码或总任务书。

## 根因与范围

1. `app_settings/api/current.rs` 的设置读取、作者内容读取和保存回读都会调用 `sync_runtime_settings_from_storage()`。
2. 未设置非空 `CODEXMANAGER_ROUTE_STRATEGY` 时，运行态同步会将数据库策略再次送入 gateway setter。该 setter 还会把规范化值镜像到进程环境；因此不能把问题描述为所有环境下每次读取必然重置。
3. 原 `route_hint::set_route_strategy()` 无条件清空数字游标、账号身份游标、P2C nonce 和维护计数。即使规范化后仍为 balanced，也会把 A、B 后的下一次选择重置为 A。
4. 另外，首次设置回读会保存 catalog 默认环境快照。后续读取补载缺失环境变量时会调用完整 runtime reload，空默认值仍可能保持缺失，从而反复走该路径。原 `route_hint::reload_from_env()` 也无条件清空游标；只修 setter 无法覆盖自然连续读取和保存回读。

本次评估后保留查询的现有同步调用：它还负责环境恢复、历史设置迁移及当前快照行为，直接移除会扩大成设置生命周期/写入边界重构，并与 Q03 重叠。本次集中修复路由策略的规范化幂等应用。设置 getter 仍有其他既有运行态副作用，不能称为完全无副作用查询。

## 修改文件与行为

- `crates/service/src/gateway/routing/route_hint.rs`
  - 提取 `apply_route_mode()`，setter 与环境 reload 共用。
  - 在同一个路由状态锁内比较并发布规范化模式；模式不变时保留三个调度 map 及维护计数，避免同模式并发 setter 重复清理。
  - 真正切换模式时仍清空上述状态。别名、大小写和两侧空白继续按既有规则标准化，无效输入继续报错且不改变状态。
  - 环境 reload 仍更新健康度开关/窗口、TTL、容量，继续执行原有 manual-preferred 缓存清理。没有修改 gateway 其他运行态的 reload 行为。
- `crates/service/src/gateway/routing/tests/route_hint_tests.rs`
  - 新增同规范化策略及全部别名不重置状态的回归，覆盖 setter 和环境 reload；同时验证无效策略不损坏状态。
  - 新增真实策略切换清理全部调度状态、切回 balanced 后从 A 开始的回归。
  - 仅将既有候选账号 fixture 暴露给同级测试模块复用。
- `crates/service/src/gateway/routing/tests/route_hint_settings_tests.rs`
  - 新增 4 项设置调用链回归，使用独立临时 SQLite，环境变量 RAII 恢复和进程级测试锁。
  - 三个健康账号 A、B 后，在没有策略环境变量时自然读取设置，继续 C；多次读取继续 A、B，作者内容读取后继续 C。
  - canonical/别名保存及回读保持连续分配，并断言数据库存储规范化策略。
  - 启动加载持久策略、重新同步同策略保持游标，以及显式环境变量优先于持久策略（双方向）。
- 本结果文件。

未改设置批量持久化/事务、API/RPC/前端、版本号、协议适配器、数据库 schema 或发布文件。

## 实际验证

所有命令的工作目录均为上述 worktree。Rust 构建使用本 worktree 的 `target`，`-j 2`；测试采用 `--test-threads=1`。测试进程设定 `CODEXMANAGER_STORAGE_BACKEND=sqlite`、独立 `CODEXMANAGER_DB_PATH` 和 `CODEXMANAGER_TEST_DB_DIR`，未使用真实账号/凭据/数据库或现有服务。设置新 fixture 清理自己创建的唯一临时目录，HTTP 回归仅使用测试服务器/mock 上游。

| 阶段 | 实际命令 | 退出码与结果 | 本地日志 |
| --- | --- | --- | --- |
| 修复前复现（仅加 setter 回归，产品代码仍为基线） | `cargo test --locked -p codexmanager-service --lib same_normalized_route_strategy_preserves_all_scheduling_state -j 2 -- --test-threads=1` | **101，预期失败**；1 项失败，同策略应用后 map 条目已消失（`no entry found for key`） | `target/q04-baseline.log` |
| 修复后 route_hint 全组 | `cargo test --locked -p codexmanager-service --lib route_hint -j 2 -- --test-threads=1` | **0；18 passed，0 failed，0 ignored** | `target/q04-route-hint.log` |
| 设置相关单元回归 | `cargo test --locked -p codexmanager-service --lib app_settings -j 2 -- --test-threads=1` | **0；13 passed，0 failed，0 ignored**（含 2 项新 route_hint 设置测试，与前一行有重叠） | `target/q04-app_settings.log` |
| 账号筛选/可用性 | `cargo test --locked -p codexmanager-service --lib gateway::selection::tests -j 2 -- --test-threads=1` | **0；11 passed，0 failed，0 ignored** | `target/q04-gateway-selection-tests.log` |
| 会话调度/绑定 | `cargo test --locked -p codexmanager-service --lib gateway::conversation_binding::tests -j 2 -- --test-threads=1` | **0；16 passed，0 failed，0 ignored** | `target/q04-gateway-conversation_binding-tests.log` |
| 既有 HTTP bridge 协议回归 | `cargo test --locked -p codexmanager-service --lib http_bridge -j 2 -- --test-threads=1` | **0；187 passed，0 failed，0 ignored**；涵盖 Responses/Chat、stream/non-stream、tools 等既有测试 | `target/q04-http_bridge.log` |
| 设置集成 | `cargo test --locked -p codexmanager-service --test app_settings -j 2 -- --test-threads=1` | **0；35 passed，0 failed，0 ignored** | `target/q04-app-settings-integration.log` |
| HTTP 调度/会话亲和性 | `cargo test --locked -p codexmanager-service --test gateway_logs prompt_cache -j 2 -- --test-threads=1` | **0；12 passed，0 failed，0 ignored**；包含并发冷启动、限流切换、streaming SSE 和匹配筛选的 Claude 用例 | `target/q04-prompt-cache-http.log` |
| Rust 格式检查 | `rustfmt --check --edition 2021 crates/service/src/gateway/routing/route_hint.rs crates/service/src/gateway/routing/tests/route_hint_tests.rs crates/service/src/gateway/routing/tests/route_hint_settings_tests.rs` | **0** | 无诊断输出 |
| diff 空白检查 | `git diff --check` | **0** | 仅 Git 换行符转换提示，无错误 |

编译中存在基线已有 unused/dead-code 等 warnings；修复前与修复后的 lib test 均报告 59 warnings，没有将 warnings 报成测试失败。完整命令输出仅保存在本地忽略目录 `target`，不会随源码自动提交。

新增回归已注册到既有 service `--lib` 测试入口，不需要新增发布/CI 工作流。

最终 HEAD 仍为基线；工作区仅包含上述 1 个产品文件、2 个测试文件和本结果文件的未提交改动。所有本次测试进程均已正常退出，没有启动长期运行的服务。

## 未执行项与证据边界

- 未执行 `cargo test --workspace -j 2` 或完整 service 全集：产品改动仅在 service 路由策略应用，优先按任务约定控制共享主机构建量，执行关联定向回归。组合版本仍需全量验收。
- 未执行 `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2`：未改 `apps/` 或桌面 Rust。根 workspace 排除了 Tauri，本文不以根测试代替桌面 manifest 验证。
- 未执行 `pnpm -C apps run build`、`pnpm -C apps run test:runtime`、`pnpm -C apps run build:desktop`、`cargo test -p codexmanager-web -j 2`：未改前端、桌面导出或 Web shell/transport。
- 新 Q04 连续分配验收是实际设置 API 函数 + 路由选择器 + 临时 SQLite 的进程内回归，没有声称浏览器或完整 HTTP 设置读取链路验收。HTTP 会话调度与协议回归是另列的既有测试。
- 未运行真实 MySQL/PostgreSQL、真实上游或真实生产环境；本修复位于存储无关的路由策略应用入口。

## 整合风险与顺序

- 按任务书先验收 Q04 的“相同规范化模式不重置”语义，再整合 Q03 的批量提交边界。与 Q03 的接点是 `app_settings/gateway.rs` 共享 setter、`api/current.rs` 保存/回读和 `runtime_sync.rs`；本任务不修改这些文件。
- Q03 若拆分校验/发布或移除保存后全量回读，应继续通过同一个幂等路由模式应用入口，不能把无条件清理重新引入 setter 或 reload。整合后至少重跑本文列出的 route_hint、app_settings 和相关调度回归，再按整合计划做 workspace/Tauri/Web/前端验收。
- 同模式 reload 现在保留游标，TTL/容量仍在访问或维护时按既有逻辑生效；真正策略切换仍重置。这是预期行为变化，不再把 reload 当作同策略轮询的重置接口。
- getter 引发的完整 gateway reload 仍可能清理 request gate、cooldown、route quality 等其他状态；本任务只保证账号调度游标语义，不承担 Q03 的整批设置副作用重构。
- 测试 fixture 恢复进程环境及 gateway/usage 配置，但现有 `ENV_OVERRIDE_BASELINE` 静态缓存没有独立清空接口；定向测试通过不能替代完整 lib 测试顺序隔离检查，整合全量验收时需继续观察。
- 未自行整合任何其他会话成果；本地通过不代表组合版本通过，也不代表已合入主仓库。
