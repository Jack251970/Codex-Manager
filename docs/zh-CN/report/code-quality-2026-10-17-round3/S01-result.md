# S01：设置、调度与全量失败修复组合（2026-10-17）

状态：S01 组合本地验收完成。最终完整 workspace 为 **2743 passed / 0 failed / 19 ignored**；独立 Web **38/0**、Tauri 设置 **6/0**、前端运行时 **250/0**、静态导出均通过。12 项数据库特性排除及其他未验证边界单列，不计通过。未将来源通过数作为本组合验收结果。

## 工作区与授权边界

- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\round3-settings-final\Codex-Manager`。创建 operation `ee2b17a6-ece3-45f9-922b-3e6c94a905ec` 已为 `completed`，初始 HEAD 为 `f14bd735dca9203d7430958480b82ba7713641a9`，初始 `git status --porcelain=v1` 为空。
- 已读根 `AGENTS.md`、`apps/AGENTS.md` 及第三轮整合任务书。主工作区和来源工作树只读；本轮没有暂存、提交、推送、创建 PR、合入或发布。
- 父会话确认冻结后才读入快照，并明确将首个重型验证时段分配给本会话。Rust 统一使用本工作区 `target`、`-j 2`；本轮本地测试命令不设 `RUST_TEST_THREADS`，不传 `--test-threads=1`（来源 CI 的原有窄检查入口保留其配置）。
- 执行器只给子进程设置本工作区 `artifacts/round3-s01/temporary-bootstrap.sqlite`、`test-databases` 和 SQLite 后端，清除继承的 `CODEXMANAGER_*` 设置，使用本地假 RPC token；各回归继续使用自有临时库和本地 mock。没有操作真实账号、用户库、外部数据库、生产服务或系统自动启动配置。

## 冻结输入、校验与整合

冻结根：`C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round3`，`index.status=VERIFIED`。

| 输入 | SHA256 |
| --- | --- |
| `index.json` | `6A728FFC70DF32DB3A46ED57B5E9119D5CDA213AD1086560E13B2CD174B520CE` |
| R02 `manifest.json` | `9D4C127C83D3E04568B6AEA03F084ACDCB90B416B7CF885DF7256D81CCCB84D3` |
| R02 `tracked.patch` | `87A406D92A1C8EB787A2573B07F2232F51B39B5D28A4FACB94E66C386B909C82` |
| R05 `manifest.json` | `F7CC8FC7AB102CD2EF3633AE1362D608682995361FECC18EDEE7E7D3FDB1308F` |
| R05 `tracked.patch`（仅校验、对照） | `3117DD99DAE0AC99F4E0E8D7EDF6B31444137D5B3F6AFD43034A7428B1B9FCB6` |
| R05 `increment/r05-over-q03.patch` | `4867EFFE9B41145A68A76688298B0316ED574E8178B26E9EEC7C1A1D5DE6AD1B` |

`artifacts/round3-s01/import-verification.json` 记录逐文件核验与合并方法；`import.cjs` 保留实际导入程序。R02/R05 合计 93 个 tracked/untracked 文件及两份完整 patch 全部逐项通过 SHA256 核验，R05 增量的冻结 Q03 旧侧也核验原始 hash；基线旧侧读取固定 Git blob 并比对规范 hash。

1. 按 manifest 完整导入 R02 的 39 tracked、5 untracked（含新增测试和来源报告），导入后逐文件 hash 精确匹配。
2. 仅应用 R05 相对 Q03 的 10 路径增量，不整份覆盖 R02 共享文件。文本只做 CRLF→LF 归一化。每路径比较旧 Q03、当前 R02、新 R05；旧侧相同的文件合入增量，新增源码/报告一并导入。
3. `.github/workflows/ci.yml` 通过三方合并干净完成；随后按检查语义人工和 YAML 解析核对。完整 `gateway::route_hint::` 范围、原设置/事务/RPC/Tauri 入口与 R05 超时/WAL 入口都保留，没有退回单个 normalize 测试。另加入本轮固定时间路径并发和 network 权限夹具回归入口。
4. `rpc.rs` 的 R02 当前内容与冻结 Q03 旧侧规范文本相同，故 R05 增量干净应用；R02/Q03 的新增 RPC 原子性断言保留。三方中间文件保存于 `artifacts/round3-s01/merges/`，没有未解决冲突。

R02、R05、Q03、Q04 来源报告原文保留在 `docs/zh-CN/report/` 的原轮次目录，仍属于来源证据。历史日志仍保留在对应来源工作区的 `target/r02-evidence/`、`target/r05-evidence/`，没有覆写或计为本轮执行。`source-log-preservation-check.json` 在本轮只读核验 54 份来源日志的存在、大小和 SHA256。

## 保留的组合语义与新增回归

- 设置 patch 完成全字段准备和纯校验后才批量持久化；SQLite、SeaORM 路径保持事务提交/回滚，只改传入字段；提交后才发布运行态。进程内写锁保护不同字段并发 patch 和旧单字段入口，保持密码/权限/分发/计费约束。
- `normalize_route_strategy()` 只解析和归一化；`apply_route_mode()` 在调度状态锁内比较规范模式，同策略及别名保留三类 map、维护计数和游标，真实变化才清理。R02 preserving reload 保留无关环境 patch、GET 补载和空白策略环境不临时重置模式的行为。
- R05 SQLite 运行时冷初始化在同一共享锁内构建并发布，构建失败保持空槽；WAL 备份仍使用 SQLite Backup API 的一致读视图。mock 记录区分阶段和完成边界，回环系统代理豁免与显式代理保护、auth 私库/RAII、header setter 和 RPC one-shot 收尾均保留。
- 新增 `committed_settings_wal_backup_and_rejected_patch_preserve_balanced_sequence`：沿用独立子进程/临时库，固定旧读事务并确认提交值只在 WAL，实际提交多字段和策略别名后做 Backup；拒绝非法混合 patch 时比较完整持久行及时间戳、运行态和环境。再推进 live 库、从备份恢复独立库，比较旧快照、最新 live 值、旧读视图、公共 Storage 读值和 integrity。各阶段严格验证 A/B 后继续 C，不把自然回绕当成重置。

数据库原子性和进程内设置写锁不等于数据库、逐字段运行态和 OS 副作用的全局事务。普通网关读者不持设置写锁；提交后发布/响应/系统操作失败仍保留明确“已保存、需重试”语义，不回滚已经提交的数据或已完成副作用。真实策略切换也不保证所有在途请求同时切换。

## 本轮新增失败、诊断和修复

首次 `cargo test -p codexmanager-storage-seaorm --lib -j 2 transfer::preflight_tests::` 为 **exit 101，3 passed / 1 failed**。失败为 `dry_run_missing_and_malformed_schema_remains_unmodified`，发生于 `transfer.rs` 的 `legacy_fixture()` 调用 `legacy.init()`，错误 `no such column: workspace_name`；尚未执行 export 或目标 preflight，不能归因于 WAL 复制。原日志 `logs/wal-restore.log` 完整保留。

`preflight-failure-diagnosis.json` 保留现场文件、hash、schema 和受控证据：四例预期保留四份 source，实际 PID 15972 只有三份，三个只读主文件的迁移表均记录 142 项，包含 013/014，最终列形状不再有 workspace_name。按不可变只读模式核对前后 hash 一致。旧 helper 仅使用 PID 与系统时钟纳秒值，缺少唯一序号和排他预留；固定时间负控生成相同路径。两个独立连接使用实际 001/013/014 SQL，模拟另一初始化完成 014 后旧缓存重放 013，精确复现相同缺列错误。受控库保留在 `preflight-013-014-controlled.sqlite`。

上述证据支持临时 fixture 路径别名导致迁移交错；原日志没有逐例 source 路径，无法回溯具体碰撞测试对和当时调度。报告不把受控交错冒充原始事件的完整追踪。

最小修复仅调整 `transfer.rs` 的测试 helper：增加进程内 `AtomicU64` 序号，`create_new` 一次性排他预留 source；archive 保持由 export 原有 `create_new` 创建。新增 8 个调用者使用相同固定时间、Barrier 并发预留的回归，断言 source/archive 共 16 个路径唯一、source 为空且再次排他创建失败、archive 不存在，最后只清理本例拥有的空 source。没有重试、串行化或放宽超时/次数/业务断言，原失败库和日志均保留。

另有一次 CI 证据脚本模块解析错误（未找到 pnpm 嵌套 `js-yaml`），未启动产品测试；原日志 `ci-validation-module-resolution-error.log` 保留。改为实际已安装依赖路径后 YAML 校验通过，没有改依赖清单或锁文件。

首次完整 `cargo test --workspace -j 2 --no-fail-fast` 从 UTC `2026-10-17T13:35:47.254Z` 运行到 `13:55:18.612Z`，完整执行 26 个 harness 后 **exit 101，2741 passed / 1 failed / 19 ignored / 0 filtered**。唯一失败为 `rpc_dispatch::network_async::tests::network_methods_preserve_member_denials_before_side_effects`，发生在 fixture 的 `initialize_storage().unwrap()`；权限 actor、20 个成员拒绝方法、两项管理员例外及允许的 account/proxy/test 断言均尚未开始。原 `logs/workspace.log` 和独立 `workspace-summary.json` 保留；没有因该失败提前中止其他目标。19 项 ignored 名称及理由与源码一致，12 项 cfg 排除均未注册。

`network-fixture-failure-diagnosis.json` 保留原库及 lock 的路径、时间、payload 和只读检查。lock 写入 `pid=9640 started_at=1792244910`；原主文件在 immutable 只读检查前后 SHA256 均为 `1E938D68E63D5CB52CA49A4FD1E400E04CB962206C14887367B475CCA5F35852`，已存在 142 项迁移和初始化入口检查的 5 个 markers，但 `web.auth.mode` 行数为 0。原测试先全局发布随机 u64 库路径，再迁移和配置；`test_env_guard` 不能阻止未持该测试锁的后台读取进入初始化。配置协调后台任务存在可达调用链，但日志未记录成功抢锁者的栈，不能确认原 caller，也不能仅凭错误文案认定另一个进程。此问题不同于上面的固定时钟路径碰撞。

修复限定在测试夹具：私有初始化、smoke check 和 auth 配置全部完成后才发布路径，保留产品迁移锁、重试、超时和权限规则。新增受控回归使用隔离子进程：持锁的未迁移库必须单次拒绝初始化；另一个私有准备完成的库在 sidecar 锁持续持有时发布，多个并发初始化观察者必须成功并读到 accounts。原成员/管理员/允许方法断言保持原文。最终实际验证结果在下节登记。

## 来源历史与未验证边界

- R02 原全量 **2730/1/19** 的唯一失败是单沉默账号超时测试读取到空 mock 记录；另有早期两项 C→A 产品失败及两项 Windows 临时目录清理失败，来源报告和日志均保留。
- R05 保留基线有限对照失败、Q03 一轮 13 项 raw client timeout、空 mock 记录、三项冷初始化 Tokio panic、主动中止全量，以及首次全量 **2724/3/19** 的 auth/header/RPC 隔离失败；其最终 **2728/0/19** 不代替本组合。
- 真实 MySQL/PostgreSQL、真实上游/生产账号、实时联网技能/价格目录、跨平台 OS 自动启动、图形浏览器完整端到端及远端 CI 未验收。桌面系统副作用只用模拟闭包；静态导出和 Web 本地 mock 不等于完整部署验收。
- 本轮仅 S01，不包含 R01 或 S02 的最终统一源码。S03 必须按冻结交付再次做统一版本验收。

## 默认忽略与特性排除

当前源码共 31 个 ignore 属性，其中默认编入 19 项，另有 mysql/postgres 特性关闭而排除的 12 项。以下为源码核对；完整运行后的注册和 ignored 对照另见最终结果核对。逐项条件、隔离 URL 变量、源码行号及 hash 在 `artifacts/round3-s01/ignored-cfg-review.json`。

| 默认编入但忽略的确切名称 | 原始理由 |
| --- | --- |
| `account::remote_storage_tests::mysql_remote_account_facade_is_authoritative_and_transactional` | `requires isolated MySQL URL` |
| `account::remote_storage_tests::postgres_remote_account_facade_is_authoritative_and_transactional` | `requires isolated PostgreSQL URL` |
| `auth::app_manager::seaorm::tests::mysql_remote_auth_wallet_and_revocation` | `requires isolated MySQL URL` |
| `auth::app_manager::seaorm::tests::postgres_remote_auth_wallet_and_revocation` | `requires isolated PostgreSQL URL` |
| `codex_skill_repositories::tests::live_builtin_repositories_scan_and_package` | `live GitHub repository probe` |
| `codex_skill_repositories::tests::live_registry_search_and_install` | `live skills.sh search and install probe` |
| `codex_skill_repositories::tests::live_repository_refresh_persists_and_installs` | `live repository persistence and install probe` |
| `models_v2::pricing_sync::tests::live_sources_parse_and_update_an_isolated_catalog` | `requires live network access to external price catalogs` |
| `mysql_remote_auth_permission_listener` | `requires isolated MySQL URL` |
| `postgres_remote_auth_permission_listener` | `requires isolated PostgreSQL URL` |
| `remote_database_drives_real_gateway_and_durable_usage` | `requires an explicit isolated gateway backend: sqlite, mysql or postgres` |
| `account_tokens::cas_tests::mysql_token_compare_and_swap` | `requires isolated MySQL URL` |
| `account_tokens::cas_tests::postgres_token_compare_and_swap` | `requires isolated PostgreSQL URL` |
| `accounts_domain_tests::mysql_account_provider_domains` | `requires isolated MySQL URL` |
| `accounts_domain_tests::postgres_account_provider_domains` | `requires isolated PostgreSQL URL` |
| `billing::tests::mysql_domains_preserve_permissions_and_concurrent_ledger` | `requires isolated MySQL URL` |
| `billing::tests::postgres_domains_preserve_permissions_and_concurrent_ledger` | `requires isolated PostgreSQL URL` |
| `tests::mysql_real_database_health_check` | `requires a user-provided isolated MySQL test database` |
| `tests::postgres_real_database_health_check` | `requires a user-provided isolated PostgreSQL test database` |

| 默认未编入的确切名称 | 所需特性 |
| --- | --- |
| `accounts_warmups::tests::mysql_reset_cycle_eligibility_and_concurrent_claim` | `mysql` |
| `accounts_warmups::tests::postgres_reset_cycle_eligibility_and_concurrent_claim` | `postgres` |
| `model_catalog::managed::tests::mysql_managed_model_aggregate` | `mysql` |
| `model_catalog::managed::tests::postgres_managed_model_aggregate` | `postgres` |
| `reset_credit_operations::tests::mysql_reset_credit_operation_idempotency` | `mysql` |
| `reset_credit_operations::tests::postgres_reset_credit_operation_idempotency` | `postgres` |
| `transfer::preflight_tests::mysql_dry_run_is_read_only_and_retries_before_import` | `sqlite, mysql` |
| `transfer::preflight_tests::postgres_dry_run_is_read_only_and_retries_before_import` | `sqlite, postgres` |
| `transfer::tests::sqlite_snapshot_imports_into_mysql` | `sqlite, mysql` |
| `transfer::tests::sqlite_snapshot_imports_into_postgres` | `sqlite, postgres` |
| `usage_analytics::tests::mysql_usage_keeps_historical_owner_and_hourly_boundaries` | `mysql` |
| `usage_analytics::tests::postgres_usage_keeps_historical_owner_and_hourly_boundaries` | `postgres` |

12 项排除中 mysql/postgres 各 6 项，import/preflight 还继承 sqlite 模块条件。上述用例需显式启用对应 feature、选择 ignored 并提供专用隔离 URL；本轮没有执行，不计通过。

## 验证结果与最终状态

本轮定向、最终完整 workspace、独立 Web/Tauri、前端和源码稳定性验收均已完成。所有命令工作目录均为本工作区，日志名相对于 `artifacts/round3-s01/logs/`；实际 Rust 命令、开始/结束、退出码及逐 harness 统计见 `artifacts/round3-s01/results.jsonl`。定向筛选排除项不计通过，各筛选可能重叠，不把各行计数累加成一次全量。

| 已实际完成的 Rust 命令 | 退出码；passed / failed / ignored | 日志 |
| --- | --- | --- |
| `cargo test -p codexmanager-core --lib -j 2 storage::settings::tests` | 0；5 / 0 / 0 | `core-settings.log` |
| `cargo test -p codexmanager-storage-seaorm --lib -j 2 settings` | 0；7 / 0 / 0 | `seaorm-settings.log` |
| `cargo test -p rusqlite --lib -j 2` | 0；17 / 0 / 0，包含两项冷初始化回归 | `sqlite-shim.log` |
| `cargo test -p codexmanager-storage-seaorm --lib -j 2 transfer::preflight_tests::`（修复前） | **101；3 / 1 / 0**，失败和现场保留 | `wal-restore.log` |
| `cargo test -p codexmanager-storage-seaorm --lib -j 2 transfer::tests::concurrent_fixed_timestamp_paths_are_distinct_and_exclusively_reserved` | 0；1 / 0 / 0 | `transfer-paths-fixed.log` |
| `cargo test -p codexmanager-storage-seaorm --lib -j 2 transfer::preflight_tests::`（路径修复后） | 0；4 / 0 / 0 | `wal-restore-fixed.log` |
| `cargo test -p codexmanager-service --lib -j 2 route_hint` | 0；26 / 0 / 0，包含本轮 WAL/设置/游标组合回归 | `route-hint.log` |
| `cargo test -p codexmanager-service --lib -j 2 app_settings` | 0；20 / 0 / 0 | `settings-lib.log` |
| `cargo test -p codexmanager-service --lib -j 2 gateway::runtime_config::tests` | 0；66 / 0 / 0 | `runtime-config.log` |
| `cargo test -p codexmanager-service --test app_settings -j 2` | 0；35 / 0 / 0 | `app-settings.log` |
| `cargo test -p codexmanager-service --test app_settings_atomic -j 2` | 0；2 / 0 / 0 | `app-settings-atomic.log` |
| `cargo test -p codexmanager-service --test rpc -j 2 rpc_app_settings_` | 0；2 / 0 / 0 | `rpc-settings.log` |
| `cargo test -p codexmanager-service --test gateway_logs -j 2 stream_timeout_failover::` | 0；8 / 0 / 0 | `gateway-timeouts.log` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 commands::settings::ui::tests` | 0；6 / 0 / 0；另一个 main harness 为 0 项，不重复计数 | `tauri-settings.log` |
| `cargo test -p codexmanager-service --lib -j 2 rpc_dispatch::network_async::tests::` | 0；4 / 0 / 0，原权限断言及 fresh-child 锁竞争受控回归均通过 | `network-fixture-fixed.log` |

Tauri 测试独立使用其 manifest；根 workspace 排除了 `apps/src-tauri`，不能用根测试代替。六项设置测试包含保存失败不执行 OS 操作、OS 应用失败明确已提交及重试语义、成功顺序和原有自动启动决策；全部使用模拟闭包，没有修改真实自动启动配置。

`artifacts/round3-s01/network-fixture-change-review.json` 记录 `network_async.rs` 修复前后字节对照：生产 `#[cfg(test)]` 前缀逐字节不变，原 `let actor` 到文件结尾逐字节不变，后者 SHA256 两侧均为 `00F0897D697E97F2896BFEBA888927964155E868198D08FDD2A54D3D0DF1C856`。因此 20 项成员拒绝、两项管理员例外及允许 `account/proxy/test` 的原断言均保留；独立只读复核再次直接计算了该对照。该静态证据与本轮 `network-fixture-fixed.log` 的实际 4 项通过分别登记，不互相替代。

| 已实际完成的前端命令 | 退出码与结果 | 日志 |
| --- | --- | --- |
| `pnpm.cmd -C apps install --frozen-lockfile --ignore-scripts` | 0；安装完成，未修改依赖清单和锁文件 | `dependency-install.log` |
| `pnpm.cmd -C apps run test:runtime` | 0；250 passed / 0 failed；skipped、cancelled、todo 均为 0 | `frontend-runtime.log` |
| `pnpm.cmd -C apps run build:desktop` | 0；编译、类型检查和静态页面生成 19/19 完成；路由表 17 项 | `frontend-build-desktop.log` |

前端命令退出码、日志 SHA256 和统计来自 `artifacts/round3-s01/frontend-static-results.json`。`apps/package.json` 的 `build` 与 `build:desktop` 都执行 `next build`，已执行后者，不重复相同构建；未改变导航或页面缓存，不额外执行 navigation 测试。静态产物完成不代表实际部署或图形浏览器端到端通过。

| 最后源码对应的轻量检查 | 退出码与结果 | 日志 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0；包含 network 夹具最后修改 | `fmt-workspace-network-final.log` |
| `rustfmt --check --edition 2021 apps/src-tauri/src/commands/settings/ui.rs` | 0 | `fmt-tauri-settings-final.log` |
| `git diff --check` | 0；输出中的换行提示保留 | `diff-final.log` |
| `node artifacts/round3-s01/validate-ci.cjs` | 0；YAML 可解析，双方设置/路由/事务/Tauri 及超时、network、固定时间路径/WAL 入口均在 | `ci-validation-network-final.log` |

上述四项实际命令、时间、退出码和日志 SHA256 追加于 `artifacts/round3-s01/light-results.jsonl`；此前失败的 CI 模块解析日志仍保留，未用当前通过覆盖。

### 最终完整 workspace

唯一名称 `workspace-final` 的完整 `cargo test --workspace -j 2 --no-fail-fast` 从 UTC `2026-10-17T14:07:46.228Z` 运行到 `14:25:54.451Z`（Asia/Shanghai 22:07:46—22:25:54），**exit 0，26 个 harness，2743 passed / 0 failed / 19 ignored / 0 measured / 0 filtered**。本次使用最终非文档源码、独立工作树 target、`-j 2` 和默认测试并发；没有提前中止或选择性跳过失败目标。

日志为 `logs/workspace-final.log`，与首轮失败的 `logs/workspace.log` 分别保存。`workspace-final-summary.json` 逐项检查每条测试完成记录、各 harness 汇总和命令完成记录，确认全部目标闭合、19 项 ignored 名称/理由与源码一致，12 项 cfg 排除未出现。核心 530 项、服务 1894 项、HTTP 网关日志 63 项、RPC 50 项、SeaORM 52 项及 Web 38 项等均通过；原权限测试和新增持锁正负控、设置/WAL/游标组合回归都在此次完整运行中通过。不把子进程内部结果重复相加，不把 ignored/cfg 排除计为通过。

### 独立 Web

最终独立 `cargo test -p codexmanager-web -j 2` 从 UTC `2026-10-17T14:25:54.452Z` 运行到 `14:27:18.912Z`，**exit 0，38 passed / 0 failed / 0 ignored / 0 filtered**，日志 `logs/web.log`。实际重新构建并执行，不以来源结果或完整 workspace 中的同包计数替代本项。

### 最终源码和工作区稳定性

最终全量前后清单分别为 `artifacts/round3-s01/source-before-final-workspace.json`、`source-after-final-validation.json`。两侧均为 55 个路径、50 个非文档文件；逐项比较路径、SHA256 和大小，50 个非文档文件完全一致。本文只在测试结束后补充结果；文档更新不计为产品源码漂移。先前 `source-before-acceptance.json` 仅属于首次验收阶段。

`final-verification.json` 记录最终复核：HEAD 仍为 `f14bd735dca9203d7430958480b82ba7713641a9`、暂存区为空；没有版本、依赖清单/锁文件或迁移文件变化；最终 workspace、独立 Web、Tauri 和 network 定向执行均为 exit 0。它同时核验 54 份 R02/R05 来源日志以及本轮两份失败日志、3 份原 preflight 库、原 network 库和 lock，共 61 份证据 SHA256 保持不变；本轮 32 份验证日志有独立大小/SHA256 清单。

`process-exit-verification.json` 于 UTC `2026-10-17T14:28:24.3574624Z` 确认本会话记录的运行器/测试子进程、本工作树 target 程序和前端进程均已退出，机器快照中 cargo/rustc 也为空。没有暂存、提交、推送、创建 PR、合入或发布；主工作区和来源工作树没有被修改。S01 可交父会话冻结，S03 仍需对最终统一源码独立验收。
