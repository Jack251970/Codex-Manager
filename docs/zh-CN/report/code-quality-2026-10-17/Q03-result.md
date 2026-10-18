# Q03 设置批量保存一致性实施结果

## 交付状态与边界

- 实施日期：2026-10-17。
- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。
- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\quality-q03\Codex-Manager`。
- 状态：Q03 实现完成，定向验证通过。首次全量验证出现两项失败；对应网关完整集与 SeaORM 完整 lib 串行复跑均通过，首次全量结果仍保留为 exit 101。未提交、推送、创建 PR、合并或发布，版本未修改。
- 原仓库 `D:\Data\Code\Codex-Manager` 仅用于读取任务书；未改写总任务书。全部构建和修改位于本工作树。
- 测试使用临时数据库、临时目录和模拟副作用；没有操作生产服务、真实账号凭据或真实外部数据库。

## 根因复核

1. 批量设置原先逐字段校验、逐字段持久化，后续字段非法会留下前面已保存的字段。
2. 多个网关单项 setter 先改变运行态，再尝试写库；持久化失败时运行态已经变化。
3. 保存响应会重新读取、同步并重写整份配置，扩大写集，可能覆盖其他请求刚提交的字段。部分复合设置需要在同一写入边界内读取旧值并合并。
4. SQLite 与 SeaORM 没有统一的设置批量提交抽象；远端设置缓存也需要跟随成功的事务更新，避免迟到的读结果覆盖新缓存。
5. 桌面自动启动操作原先发生在服务保存之前，混合非法设置可能已经改变系统状态。

## 实现

### 校验、事务与发布

- `app_settings/api/patch.rs` 先纯校验和标准化全部字段，生成本次修改的键和值及运行态发布计划。非法路由、请求头、代理、cron、环境值中的 NUL 等在事务前拒绝。
- SQLite `Storage::save_app_settings` 和 SeaORM `SettingsRepository::save` 在单个事务中保存指定键；支持底层批量删除，批次中写入失败回滚。服务既有空值保存语义保留。
- 事务成功后才发布运行态；代理客户端在提交前构建。SeaORM 不依赖插入 ID，避免 MySQL 对相同值 upsert 返回零受影响行时误判失败。
- 批量 patch 与复用协调器的 setter 使用设置写锁覆盖准备、合并、事务、发布和响应快照；GET 的旧同步/迁移路径也受同一锁协调。没有运行态副作用的 UI 单键 setter 仍直接持久化。仅是进程内协调，没有新增多服务分布式同步。
- 保存响应使用不重载运行态、不重写设置的快照路径。单项网关、服务地址、Web 认证/分发等设置复用相同提交边界。
- `gateway/transport/set` 四项设置由一个 patch 提交，避免较后的非法间隔值造成部分保存。原有参数转换与响应字段保留；空 patch 现在也会初始化/读取设置存储。
- 远端缓存仅在事务成功后更新本次键；远端读取和缓存安装与提交串行，避免旧读覆盖新缓存。

### 复合字段与安全边界

- 窗口 UI 挂载与旧版轻量模式反向字段同批保存，保持新字段优先。
- 后台任务和配额保护先基于当前快照合并，再提交；后台任务只发布本次显式提供的字段。
- 环境编辑器仅发布变化的环境键，不重放陈旧快照。显式设置与同批环境项重叠时，显式设置优先，并同步已持久化的重叠环境项。普通无关设置不创建全量环境快照。
- 环境重载保留无关的运行态路由、并发数、User-Agent 版本、配额保护与后台任务；显式代理准备使用目标环境参数构建客户端。
- Web 密码在提交前哈希，校验最终认证模式与分发组合。保留账户计费锁、分发锁及密码模式需要密码的约束；兼容旧接口仅清空密码的行为，不改变认证安全检查。
- `current_web_auth_mode` 由单次设置列表快照中的模式和密码判定，避免合法批次从密码模式切到账户模式时组合出旧模式与新空密码；Web 密码分支再次读取哈希时若缺失则拒绝认证。补充确定性缺失哈希/cookie 回归及双存储路径的并发模式读取回归。
- SSE 单项设置继续返回环境覆盖后的实际生效值，同时保留用户请求值落库，维持旧接口语义。
- 路由 setter 仍保留原有游标重置行为；相同规范值不重置属于 Q04，没有在本任务实现。

### 桌面外部副作用

- 服务保存成功后才执行自动启动系统操作，并协调桌面保存和发布顺序，避免迟到的旧请求最后应用。
- 数据库提交后，运行态或响应构造若报错，错误分别带 `settings_persisted_runtime_apply_failed` 或 `settings_persisted_response_failed`，明确设置已保存；不宣称数据库已回滚。
- 自动启动系统操作失败会明确提示设置已保存、具体字段失败、需要重试。测试通过模拟闭包验证，没有修改真实系统自动启动项。

## 改动文件

- 存储：`crates/core/src/storage/{settings.rs,settings_tests.rs,traits/domain.rs}`；`crates/storage-seaorm/src/{settings.rs,domain/{sqlite.rs,seaorm.rs,tests.rs}}`。
- 设置入口、快照、协调和缓存：`crates/service/src/app_settings/{api/{mod.rs,patch.rs,current.rs},store.rs,runtime_sync.rs,gateway.rs,service.rs,ui.rs,mod.rs}`。
- 环境准备/发布：`crates/service/src/app_settings/env_overrides/{manager.rs,process.rs,normalize.rs,snapshot.rs,mod.rs}`。
- 网关纯标准化与代理准备：`crates/service/src/gateway/{core/runtime_config.rs,core/tests/runtime_config_tests.rs,routing/route_hint.rs,routing/selection.rs,routing/tests/route_hint_tests.rs,mod.rs}`。
- 后台任务：`crates/service/src/usage/{refresh/settings.rs,refresh/mod.rs,usage_refresh.rs}`。
- Web 边界：`crates/service/src/auth/{app_manager.rs,web_access.rs}`。
- RPC：`crates/service/src/rpc_dispatch/gateway.rs`。
- 桌面：`apps/src-tauri/src/commands/settings/ui.rs`。
- Web 认证读取：`crates/web/src/{auth.rs,auth_tests.rs}`。
- 回归：`crates/service/tests/{app_settings_atomic.rs,app_settings.rs,rpc.rs}`；CI 定向入口 `.github/workflows/ci.yml`。
- 本报告。无 schema、版本或依赖清单变更。

## 验证记录

以下命令均在上述独立工作树执行。Rust 使用 `-j 2`；桌面 manifest 与根 workspace 共用本工作树的 `target` 并串行等待构建锁，没有使用其他工作树的可写输出。

| 命令 | 实际结果 |
| --- | --- |
| `cargo test -p codexmanager-core -j 2 storage::settings::tests` | exit 0，5 passed |
| `cargo test -p codexmanager-storage-seaorm -j 2 settings` | exit 0，7 passed |
| `cargo test -p codexmanager-service -j 2 --test app_settings` | 首次 exit 1：删除旧快照重写逻辑时漏保留 `persisted_text_value`；恢复该读取辅助函数后重跑 exit 0，35 passed。后续修改由最终 workspace 再验证 |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 commands::settings::ui::tests` | 首轮及最终源码复跑均 exit 0，6 passed；系统操作使用模拟闭包。最终日志 `target/q03-tauri-settings-retest.log` |
| `pnpm.cmd -C apps install --frozen-lockfile --offline --ignore-scripts` | exit 0，依赖由本机缓存复用；无清单/锁文件修改 |
| `pnpm.cmd -C apps run build:desktop` | exit 0，Next 编译、类型检查和 19 个静态路由导出成功 |
| `pnpm.cmd -C apps run test:runtime` | exit 0 |
| `cargo test -p codexmanager-service -j 2 --test app_settings_atomic -- --test-threads=1` | exit 0，2 passed；两种存储分发均执行完整断言组 |
| `cargo test -p codexmanager-service --test rpc -j 2 rpc_gateway_transport_invalid_interval_preserves_all_settings_and_runtime` | exit 0，1 passed |
| `cargo test --workspace -j 2 --no-fail-fast` | exit 101：汇总 2717 passed、2 failed、19 ignored；具体失败和复核见下文 |
| `cargo fmt --all -- --check` | exit 0 |
| 修改 Rust 文件的 `rustfmt --check --edition 2021`、`git diff --check` | exit 0，包含最终新增并发测试 |

CI 增加一个设置回归步骤，包含 core/SeaORM 设置、服务环境/代理/后台任务准备、既有设置集成、新事务集成及 RPC 筛选；在既有 macOS 桌面构建任务中加入设置副作用测试。使用本工作树依赖中的 `js-yaml` 4.1.1 解析完整 workflow、断言新增命令和位置均成功，定向 `git diff --check` exit 0；未触发远端 CI。

### 全量结果与失败复核

全量测试已执行完毕。core 单元 530、其余 core 集成 48、service 单元 1879、既有 app_settings 35、新 atomic 2、RPC 50、Web 38 等通过。全量中服务单元 8 项忽略，服务外部集成 3 项忽略，SeaORM 8 项忽略；这些需要真实外部数据库或联网的测试不算通过。完整日志位于工作树的 `target/q03-workspace-test.log`（未跟踪构建输出）。

两项失败：

1. `stream_timeout_failover::gateway_single_silent_account_still_obeys_stream_header_timeout`，`crates/service/tests/gateway_logs/stream_timeout_failover.rs:321`。502、200ms 超时提示与耗时边界断言已通过，但立即读取模拟上游通道时得到 `requests=[]`，未满足 1..2 次记录。该测试/fixture 与请求发送、stream 超时代码没有本次修改；本次 trace 为正确本地 URL、单账号候选、200ms 头超时、最终 417ms 返回 502。模拟上游在完整解析请求体后才发记录，存在观测时序可能，但未用此推测替代复核，也没有声称基线必然同样失败。
2. `transfer::preflight_tests::failed_checksum_and_connection_preserve_source_and_allow_clean_restore`，`crates/storage-seaorm/src/transfer/preflight_tests.rs:249`。物理文件复制后的 legacy adapter 读取 `transfer_fixture` 得到 `None`，期望 `Some("retained")`。只读检查本次失败资源：源与恢复主文件同为 1,343,488 bytes，SHA256 相同，均没有该键和值；源 `-wal` 为 4,136,512 bytes，键和值仍在 WAL（offset 325488/325504），导出 jsonl 也包含它们。失败发生在 `import_snapshot` 前，源 fixture 调用 core 的单项 setter，没有调用 SeaORM `SettingsRepository::set`。测试、fixture、rusqlite 连接关闭实现与基线均无修改；core 单项 setter 只是抽出相同 SQL。现有 fixture 将丢弃 pool 视作可安全复制主文件，但异步连接池关闭不保证此时已 checkpoint，主文件复制遗漏了已提交 WAL。此项作为已有恢复测试/连接关闭假设的 HANDOFF，没有借本任务修改迁移或备份行为，也没有实际运行基线对照来声称已复现基线失败。

串行复核命令：

- `cargo test -p codexmanager-service --test gateway_logs -j 2 -- --test-threads=1`：exit 0，57 passed，包含此前失败的 200ms 超时测试；流式/非流式、tools、Responses/Chat、图片和既有 Anthropic mock 上游回归均通过。日志 `target/q03-gateway-logs-retest.log`。
- `cargo test -p codexmanager-storage-seaorm --lib -j 2 -- --test-threads=1`：exit 0，51 passed、8 ignored，包含此前失败的恢复测试。日志 `target/q03-seaorm-lib-retest.log`。

构建存在未使用函数/导入、既有测试可移除 `mut` 等 warning；未按告警批量修改本任务之外的代码。验证记录区分 warning、忽略项与实际失败，不将初次全量结果改写为通过。

串行完整测试集通过后，没有再运行第二轮全 workspace；不能将分别通过的复跑说成一次全量命令 exit 0。未复现的网关记录时序和 WAL 主文件复制问题保留上述证据与 HANDOFF，组合版本仍应关注这些既有测试的稳定性。

最终检查：`cargo fmt --all -- --check`、桌面修改文件的 `rustfmt --check --edition 2021`、`git diff --check` 均 exit 0；workflow YAML 与 10 条设置命令/桌面测试位置检查通过。差异中常见密钥模式扫描无匹配，版本/依赖清单/迁移目录无变更；HEAD 仍为上述基线。工作树保留 39 个已跟踪修改文件，以及新增事务测试和本报告，未暂存或提交。未发现该工作树测试启动的遗留服务进程。

### 新增回归覆盖

- SQLite 与 SeaORM 的事务回滚、只更新指定键、并发不同键和适配层行为；MySQL/Postgres 参数化 upsert SQL 生成断言。
- 两条服务存储分发路径均使用独立子进程与临时 SQLite 文件，隔离全局连接池、缓存和进程环境。
- 混合合法/非法设置后完整数据库行（含时间戳）、运行态、环境均不变；通过 SQLite trigger 注入事务中段失败并检查全部回滚。
- 不同字段并发 patch，复合配额/后台任务不同子字段并发 patch，GET 与 PATCH 并发，以及数据库审计 trigger 验证响应不重写其他键。
- 单项 setter 持久化失败不发布运行态；Web 最终组合、密码清空兼容、认证/分发并发约束和计费锁。在两种存储分发下各执行 20 轮密码/账户模式切换与 320 次并发读取，禁止观察到 `none`；Web 密码分支缺少哈希时，无 cookie 或旧 cookie 均拒绝。
- 环境字段与显式设置优先级、无关环境修改不覆盖更新后的运行态；代理准备不改变运行态，目标环境缺省值与显式空值语义。
- RPC transport 的非法 SSE 间隔不造成前面字段部分保存。
- 桌面保存失败不调用系统操作；系统失败返回明确已保存语义；成功后执行并返回设置。

## 未验证范围与整合注意

1. 未连接真实 MySQL/Postgres：SeaORM 已实际执行 SQLite 事务/服务分发，MySQL/Postgres 仅验证生成 SQL；不能作为真实外部数据库验收。
2. 未运行真实 Windows 自动启动变更或图形界面交互；验证了桌面 Rust 边界测试和静态构建，未在 macOS/Linux 实机运行系统操作。
3. 没有使用真实账号或上游服务测试网络业务，没有部署/生产验收。环境重载仍保留既有客户端构建回退语义，本任务没有重构通用 HTTP 客户端错误处理。
   Web 覆盖为并发模式 getter 与密码判定 helper 回归，没有执行真实 HTTP 并发鉴权测试。
4. 本地写锁约束本进程设置 API；不提供跨进程合并同一个复合 JSON 设置或跨服务同步运行态的保证。
   真正远端存储在提交时连接中断，可能存在提交结果不明的数据库协议边界；本任务没有引入分布式提交确认或重放协议，也未用 SQLite 测试替代这一外部验收。
5. 与 Q04 重叠于 `runtime_sync.rs`、`api/current.rs` 和 `routing/route_hint.rs`。按任务书先验证/整合 Q04 的相同值不重置语义，再整合 Q03；组合版本仍需重跑设置、调度、workspace 与桌面回归。
6. CI 的定向命令会纳入仓库入口；未触发远端 CI。工作树测试成功不等于已合入主仓库。
7. 数据库批次是原子提交，运行态仍逐字段发布，普通网关读者不持设置写锁，发布期间可能观察到过渡状态。后续发布失败时，先前运行态/系统副作用可能已经生效；错误明确要求重启或重试，没有跨数据库、运行态、系统操作的整体回滚。
