# R05：Q03 全量失败复核与可靠性修复

## 交付状态

- 日期：2026-10-17。最终状态：R05 范围验收通过，交付为独立工作区内未暂存改动。完整 workspace 真实 exit 0：2728 passed、0 failed、19 ignored；独立 Tauri 设置 6 passed、0 failed，前端 runtime 250 passed，桌面静态构建成功。该结论不代替 R02 的 Q03+Q04 组合验收。
- 实施工作区：`C:\Users\qxnm\.codex\worktrees\round2-workspace-failures\Codex-Manager`。
- 干净对照工作区：`C:\Users\qxnm\.codex\worktrees\round2-r05-clean-control\Codex-Manager`。
- 两者共同 HEAD：`f14bd735dca9203d7430958480b82ba7713641a9`，由本会话创建并附加的 managed worktree。
- 不提交、暂存、推送、创建 PR、合并或发布。原仓库及第一轮来源工作区只读；测试仅使用临时 SQLite、临时目录和本地 mock，不操作真实账号、外部数据库或生产服务。

## 输入与证据完整性

- 只读任务书：原仓库 `docs\zh-CN\report\代码质量第二轮组合验收与补漏-2026-10-17.md` 和 `代码质量改进实施任务-2026-10-17.md`，已按绝对路径阅读；没有改写。
- 输入仅为 `C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round2-3e2657e8\Q03`，没有导入 Q04 或其他 Qxx。
- manifest SHA256：`222EC98D5A808F6547A9F622E16DA255F2BF8E1468CEDB08109A8AE33107034C`。
- tracked.patch SHA256：`08ECC10D851D9CE1071B059ECD4AD9C84A32E534E8B6B1C31371BC4513C4A0B4`。
- manifest 的 39 个 tracked、2 个 untracked 文件 SHA256 全部核对通过；共同基线和初始干净状态通过；`git apply --check`、`git apply` 均 exit 0，并复制全部新增文件。
- 初次对导入文件逐字节复核遇到 Windows 换行差异（首个为桌面 `settings/ui.rs`）；对全部 41 个文件进行 CRLF/LF 归一化后的严格文本比较，全部一致。输入文件本身的 SHA256 没有放宽。逐文件证据：`target/r05-evidence/input-validation.json`。
- 无第二份输入，无补丁冲突；没有以整文件覆盖合并两份任务。第一轮 `Q03-result.md` 原样保留为来源报告。
- 原日志只读复制到本工作区 `target/r05-evidence/original-q03-workspace-test.log`，322663 bytes，SHA256：`CB97D2384DF913F1419959A0AF1383881E897927BB199F878AB94E6A776A439D`。原首次全量仍记为 exit 101：2717 passed、2 failed、19 ignored。

所有下述相对日志路径，均位于实施工作区；干净对照的 target 独立保留在对照工作区。没有复用其他会话的 target/out。

## 有限对照

修复活动源码前，预先限定基线和 Q03 各三轮，每轮按顺序运行以下两条命令，均保留默认测试线程并发，不带 `--test-threads=1`：

1. `cargo test -p codexmanager-service --test gateway_logs -j 2`
2. `cargo test -p codexmanager-storage-seaorm --lib -j 2`

基线先执行完毕，再编译/测试 Q03，避免两个 Rust 重型验证同时运行。wrapper 逐条保存真实 cargo 退出码，诊断失败后继续完成预定轮次；不把 wrapper 最终退出码当作 cargo 的退出码。机器可读记录：`target/r05-evidence/comparison-results.jsonl`。

| 版本 | 轮次 | gateway：退出码；passed/failed/ignored | SeaORM：退出码；passed/failed/ignored | 日志前缀 |
| --- | --- | --- | --- | --- |
| 干净基线 | 1 | 101；56/1/0；双账号次数观测失败 | 101；45/1/8；原 WAL 恢复失败 | `baseline-{gateway,seaorm}-1.log` |
| 干净基线 | 2 | 101；56/1/0；双账号次数观测失败 | 101；45/1/8；原 WAL 恢复失败 | `baseline-{gateway,seaorm}-2.log` |
| 干净基线 | 3 | 101；56/1/0；原单账号 requests=[] | 0；46/0/8 | `baseline-{gateway,seaorm}-3.log` |
| Q03 未修复 | 1 | 101；44/13/0；13 项 raw client read timeout | 0；51/0/8 | `q03-{gateway,seaorm}-1.log` |
| Q03 未修复 | 2 | 0；57/0/0 | 0；51/0/8 | `q03-{gateway,seaorm}-2.log` |
| Q03 未修复 | 3 | 101；56/1/0；双账号次数观测失败 | 0；51/0/8 | `q03-{gateway,seaorm}-3.log` |

基线复现结论：原单账号空记录和原 WAL 恢复失败均已实际复现；双账号测试共用同一观测 fixture，前两轮只观测到健康次账号，HTTP 成功但次数断言失败。WAL 第三轮通过不抵消前两轮失败。不同版本的 SeaORM 用例数差异来自 Q03 新增回归，不能据此省略失败。

### 额外 13 项超时证据

Q03 首轮 13 个失败均发生在 `gateway_logs/support.rs` 原第 209 行 `read_to_string(...).expect("read")`，系统错误 10060。第二轮全部 57 项通过，第三轮这些用例也通过，但没有据此抹掉首轮失败。原 helper 丢失已接收部分数据，历史日志不足以分辨未响应、部分响应或完整响应后等待 EOF；本轮先增加失败诊断，保持原 2 秒超时和失败语义，再限定两个新进程定向复核，不作无限重试。

代码检查未发现设置锁自重入：one-shot 在监听前完成设置同步，GET 使用 unlocked 内层同步，网关 HTTP 路径不取得设置写锁。响应路径在字节通道结束后仍等待 finalization/accounting completion，存在字节发送后收尾尚未结束的阶段；这只是待区分的路径，尚不能追溯断言首轮 13 项都因 EOF 迟到。客户端 panic 跳过 `server.join()` 还有迟到线程风险，不能把等待环境锁的“over 60 seconds”提示当作请求耗时。

13 个确切失败名称（均为 `--test gateway_logs`）：

- `anthropic::gateway_claude_failover_cross_workspace_strips_session_affinity_headers`
- `anthropic::gateway_claude_protocol_end_to_end_uses_codex_headers`
- `basic::gateway_applies_migrated_legacy_compact_model_forward_rules`
- `basic::gateway_applies_saved_model_forward_rules_to_codex_responses_request`
- `basic::gateway_follows_client_model_even_when_account_source_mapping_exists`
- `hybrid_routing::hybrid_aggregate_first_aggregate_only_skips_active_account_and_uses_aggregate_api`
- `hybrid_routing::hybrid_aggregate_only_streaming_responses_forces_identity_and_logs_success`
- `hybrid_routing::hybrid_dual_route_falls_back_to_aggregate_when_account_pool_is_empty`
- `images::native_codex_image_generation_responses_request_passthroughs_tool_and_sse`
- `prompt_cache::gateway_native_parent_thread_uses_root_cache_affinity_and_preserves_client_key`
- `hybrid_routing::hybrid_aggregate_first_aggregate_only_failure_never_uses_account_pool`
- `hybrid_routing::hybrid_aggregate_first_dual_route_falls_back_once_and_preserves_tools`
- `hybrid_routing::hybrid_aggregate_first_dual_route_streams_from_account_after_aggregate_failure`

## 根因与修复

### 网关观测边界

原 fixture 在 tiny_http 提供请求并完整读取 body 后才发送捕获记录；`server.join()` 等待下游服务结束，不等待 mock 完成。`try_iter()` 当下为空不能证明请求未发送。历史日志没有 accept/header/body 事件，不能追溯认定那一次究竟停在哪个阶段。新的 actor 按 ready、accept、headers、body、capture、EOF/完成事件收集；三个确定性回归覆盖延迟捕获、不同取消阶段及无连接收尾。另有 poison 和 reader panic 回归，确保失败也继续回收其他 reader、Drop 不再引发第二次 panic。

产品错误文本来自包住 `builder.send()` 的 200ms 头超时，涵盖连接、发送和首部等待；下游提前取消有不同错误出口。没有据历史空通道断言产品取消故障。TCP_NODELAY 默认开启、Bytes body 已就绪，客户端没有等待 100 Continue；不能据部分 body 或零连接事件推断产品一定错误取消。

新的证据还发现 Windows 系统代理已开启，空代理环境变量不能排除系统代理；hyper-util 的系统 Matcher 没有默认回环豁免。测试环境现在在原共享环境锁内设置仅回环地址的 NO_PROXY/no_proxy，按后设置先恢复、最后释放环境锁，隔离本地 mock 与继承代理。不会改系统代理设置或产品代理逻辑；显式 Proxy::all 仍必须经过指定代理。新增真实本地 HTTP 回归以沉默代理做负控、回环豁免做正控、显式代理做保护，三者均已通过；当前主机实际 Matcher 为 `before=true / after=false`。历史无连接那次仍缺少当时的 Matcher/套接字证据，不能回推为唯一根因。

### WAL 一致性边界

原测试的 `drop(Storage)` 不等待共享 SQLx pool 完成关闭，不能作为只复制主文件的安全边界。原失败发生在 import 前；基线已实际复现，因此不能归咎于 Q03 设置事务。

修复固定旧读事务，明确制造已经提交但仍只在 WAL 的值；使用现有 SQLite Backup API 获得完整视图，验证源库、独立 legacy 恢复库和正常 JSONL 导入。损坏校验直接阻止 import；业务表保持为空，迁移初始化的锁行保持原值。四项 preflight 默认并发回归均已通过，最终全量结果另列。

## 相对 Q03 快照的额外修复

以下是本工作区相对冻结 Q03 的全部额外文件；其余 Q03 文件保留导入内容，不引入 Q04 的策略发布逻辑，不修改迁移、版本或依赖。

| 文件 | R05 增量及保证 |
| --- | --- |
| `crates/rusqlite/src/lib.rs` | 产品修复：构建和发布运行时受同一锁保护，避免冷启动竞争失败者在异步上下文释放运行时；构建错误仍返回，后续调用可重新尝试。只在连接初始化时取锁，不改变查询执行方式。 |
| `crates/rusqlite/src/runtime_tests.rs`（新增） | 两项冷槽回归：并发首次初始化只构建一次、共享同一实例；构建失败不填槽且后续成功可复用。 |
| `crates/service/tests/gateway_logs/stream_timeout_failover.rs` | 明确 mock 监听/读取/捕获完成边界；三个阶段回归、两个异常清理回归和一个真实代理对照。原两个业务测试的 200ms、502 文本、1..2 / 2..5 请求次数、耗时与 trace/wire SHA 断言保持。诊断不输出凭据或请求/响应 body。 |
| `crates/service/tests/gateway_logs/support.rs` | 本地 mock 在既有环境锁内设置回环 NO_PROXY/no_proxy，恢复后才释放锁；不影响显式代理。原 HTTP helper 只增加失败诊断，保留 2 秒超时、读错误即失败、既有重试次数及成功判定。 |
| `crates/storage-seaorm/src/transfer/preflight_tests.rs` | 固定旧读事务和 committed WAL sentinel，使用 SQLite Backup API 获得一致性快照；校验源库不变、独立恢复库、损坏导入不写目标、正常 JSONL 导入。 |
| `crates/service/src/auth/tests/auth_account_tests.rs` | 临时 SQLite 私有初始化完成后才发布路径；纳秒时间戳加序号防路径复用，RAII 在 panic 时也恢复环境。新增真实 SQLite/注入 panic 回归，保留两个 auth 测试的原业务断言。 |
| `crates/service/tests/gateway/availability/upstream_headers.rs` | 纯请求头测试的六处设置/恢复调用使用运行态内层接口，避免依赖其他测试遗留的数据库；全部 header 断言保留。originator/residency 持久化仍由现有 app_settings 集成测试验证。 |
| `crates/service/tests/rpc.rs` | 临时目录加入时间戳并明确创建失败；完成私库初始化后才发布 DB_PATH，环境字段先恢复、最后释放原环境锁；登录往返的四次请求后各等待对应 one-shot listener/HTTP 收尾。独立启动的 profile reconciliation 不归该 join，不能宣称所有后台工作已完全 drain。保留 Q03 在此文件中的全部新增测试。 |
| `.github/workflows/ci.yml` | 保留 Q03 的十条设置测试与独立桌面入口；新增两个默认并发 R05 定向命令。冷启动回归也由既有 SQLite shim 入口覆盖。 |
| `docs/zh-CN/report/code-quality-2026-10-17-round2/R05-result.md`（新增） | 本报告；第一轮来源报告不改写。 |

### 修复中实际暴露的问题

- `fixed-stream-timeout.log`：5 项默认并发，exit 101；4 passed、1 failed。三个新 fixture 回归通过，双账号用例完整捕获 5 个产品请求；单账号用例只有 ready、结束唤醒、关闭事件，没有产品连接，仍为 `requests=[]`。因此捕获完成边界修复不能单独解释或解决全部网关失败。
- `fixed-wal-preflight.log`：4 项默认并发，exit 101；1 passed、3 failed。三项均在兼容层首次运行时初始化时触发 Tokio `Cannot drop a runtime in a context where blocking is not allowed`。原 `OnceLock` 的 get/build/set 间隙允许并发创建多份运行时，竞争失败者在异步上下文释放最后一个 Arc 而 panic。现在构建与发布受同一互斥锁保护；失败返回错误并保留空槽，后续调用仍可重新尝试。
- `runtime-cold-init.log`：`cargo test -p rusqlite --lib -j 2 runtime_tests:: -- --nocapture`，exit 0；2 passed、0 failed、0 ignored、15 filtered。通过本地冷槽和受控 builder 事件验证单次构建、共享指针及构建错误后恢复，避免全局槽已暖启动掩盖问题。
- `runtime-fixed-preflight.log`：`cargo test -p codexmanager-storage-seaorm --lib -j 2 transfer::preflight_tests:: -- --nocapture`，exit 0；4 passed、0 failed、0 ignored、55 filtered。没有降低测试并发。
- `gateway-target-diagnostic.log`：修复运行时并增加脱敏诊断后，原定向命令 exit 0；5 passed、0 failed、0 ignored、55 filtered。实际目标匹配本地 mock，单账号完整捕获 2 条；双账号前两条在 body 完整之前取消，随后 3 条完整。单次通过没有被用来抹去先前无连接的失败。
- `workspace-default-concurrency.log`：第一轮修复后全量实际启动，随后为应用独立审查发现的 mock 异常清理及代理隔离修复主动中止；进程树终止后真实 exit 1。该部分日志保留，没有完整测试总数，不能算全量通过。最终完整命令另写独立日志，不覆盖此记录。
- `gateway-isolated-final.log`：代理隔离及异常收尾修复后，同一定向命令 exit 0；8 passed、0 failed、0 ignored、55 filtered。实际系统 Matcher 为 `before=true / after=false`，受控代理负控、回环直连正控与显式代理三项行为均通过；双账号捕获 5 条、单账号捕获 2 条。日志中的两次 injected panic 是预期故障注入，均已捕获且对应测试通过。随后仅补局部 REQUEST_METHOD 的保存/移除/恢复和不打印原代理值的恢复断言，纳入最终完整 workspace 验证。
- 第一轮完整 `workspace-final.log`：exit 101；2724 passed、3 failed、19 ignored。gateway_logs 63/0、SeaORM 51/0/8 ignored、SQLite shim 17/0、Web 38/0 均通过。新增失败为 `auth::account::tests::resolve_refresh_target_prefers_explicit_account_id_over_current_account`（私库先发布再初始化，缺 group_name）、`gateway::availability_tests::upstream_headers::codex_header_profile_uses_dynamic_originator_and_residency_requirement`（前例 panic 未恢复 DB_PATH，借用其库后缺 service_tier）、`rpc_chatgpt_auth_tokens_login_read_logout_roundtrip`（首次 start_server 打开新库时 database is locked）。前两项级联路径有实际相同数据库绝对路径证据；初始化重叠的具体第二个调用者没有被追踪，不能声称已完整复现每个线程交错。详细合计见 `workspace-first-complete-summary.json`。三项修复及第二轮完整验收结果另列，不覆盖此失败日志。
- 第二轮完整 `workspace-final-after-isolation.log`：真实 exit 0；26 个 harness/doc-test 汇总为 2728 passed、0 failed、19 ignored、0 filtered。core 530/0，service 单元 1880/0/8 ignored，app_settings 35/0、atomic 2/0，gateway_logs 63/0，RPC 50/0，SeaORM 51/0/8 ignored，Web 38/0，SQLite shim 17/0；其余套件计数见 `workspace-final-summary.json`。原网关/WAL 两项、第一轮新增三失败及新增 auth panic 恢复回归均实际通过。此结果对应应用 auth/header/RPC 隔离后的源码，不覆盖先前失败和中止记录。

## 验证记录

| 实际命令 | 结果 | 日志（`target/r05-evidence/` 下） |
| --- | --- | --- |
| `pnpm.cmd -C apps install --frozen-lockfile --offline --ignore-scripts` | exit 0，无清单/锁文件改动 | `frontend-install.log` |
| `pnpm.cmd -C apps run test:runtime` | exit 0；250 passed、0 failed、0 skipped | `frontend-runtime.log` |
| `pnpm.cmd -C apps run build:desktop` | exit 0；类型检查、编译及静态页面生成 19/19 成功；路由表列 17 项 | `frontend-build-desktop.log` |
| `cargo fmt --all -- --check` | exit 0；包含最后的 auth/header/RPC 隔离改动 | `after-isolation-format.log`（此前 `final-format.log` 保留） |
| `node target/r05-evidence/validate-ci.cjs` | exit 0；完整 YAML 及设置/桌面/R05 入口校验 | `final-ci.log` |
| `git diff --check` | exit 0；包含最后隔离改动，仅 LF/CRLF 提示，无空白错误 | `after-isolation-diff.log`（此前 `final-diff-check.log` 保留） |
| `cargo test --workspace -j 2 --no-fail-fast`（第一轮完整） | exit 101；2724 passed、3 failed、19 ignored；完整失败保留 | `workspace-final.log` |
| `cargo test --workspace -j 2 --no-fail-fast`（补齐隔离后） | exit 0；2728 passed、0 failed、19 ignored、0 filtered；不带测试线程限制，未设置 RUST_TEST_THREADS | `workspace-final-after-isolation.log`、`workspace-final-summary.json` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 commands::settings::ui::tests` | exit 0；6 passed、0 failed、0 ignored、77 filtered；main target 0 项；根 workspace 排除 Tauri，仅使用本实施工作区自己的 target | `tauri-settings.log`、`tauri-settings-summary.json` |

完整命令、退出码及独立日志路径记录在 `target/r05-evidence/validation-results.jsonl`；主动中止的说明另存 `validation-notes.jsonl`。所有 Rust 重型构建/验证由根会话串行安排，`-j 2`；主实施和干净对照各使用自己的输出目录。没有以重复原样运行直到通过的方式替换已失败证据。

逐条命令的总目录另见 `target/r05-evidence/validation-catalog.json` 和同名 `.md`：包含全部有限对照、两次各单项的新进程诊断、修复期定向命令、主动中止、两次完整运行、前端和最终 Tauri 结果。完整运行与 partial 结果分开；各条 Rust 计数来自对应日志，不能将定向结果相加冒充一次全量。日志没有退出码尾注的前端三条命令，以本会话已完成工具结果的实际 exit 0 为来源；构建 19/19 是页面生成数，路由表为 17 项。编译警告仍在原日志中，没有把无错误编译等同真实系统操作验收。

第一轮独立静态复核证据：`target/r05-evidence/final-static-review.json`（SHA256 `EFC41596A2C27E62DC1E7D9DA3B2D4E834267BD43CE292A08C6C42C5AFF10556`）。当时冻结 Q03 的 41 文件中，40 个经 CRLF 归一化后保持一致，仅 CI 有预期增量；第一轮来源报告逐字节一致。

补齐隔离后的独立复核另存 `target/r05-evidence/final-static-review-after-isolation.json`（SHA256 `56CB09474FA4348275C5A0AFCD6314EF1B8D79CB6E3BCCCF6BB84FDAF54463B2`），状态仅为 `STATIC_CHECKS_PASSED`，没有覆盖原记录。41 个冻结输入哈希全部重新核对；当前 39 个文件经 CRLF 归一化后保持一致，CI/RPC 为两处预期重叠，R05 额外文件恰为 10 个，合计 49 个变更路径与 Git 状态相符。复核期间源码哈希稳定。Auth 原 2+7 项业务断言逐字、顺序保留；header 恰有六处运行态 setter 替换；RPC 新增四次 join 之外的业务段和 Q03 atomic 回归保持不变。原网关 28 项业务断言全部保留，无新增 ignore、测试线程限制或产品环境变量。静态复核不代替运行验证；报告在验证后继续更新，源码哈希独立检查。

最终完整结果另外经过独立解析复核：`target/r05-evidence/final-workspace-result-review.json`（SHA256 `B3647E8F589DDDD6A5B261EFE7E91D163AAC80786DAD1C3536CA7F2B66995C75`）。26 个汇总、逐项结果及各 harness 的 running 数量互相一致，与 `workspace-final-summary.json` 的 2728/0/19/0 逐字段相符；实际完成记录的退出码为 0。19 个 ignored 的名称和理由与源码属性逐字一致，12 个 cfg 排除项没有出现在注册结果；第一轮完整的三个失败在最终日志均恰好出现一次 ok。此复核没有再次运行测试，也不声明桌面测试结果。

## 19 个既有忽略项

最终完整日志中的 19 项均未执行，不计通过；其中 service 单元 8、remote_auth_listener 2、remote_gateway 1、SeaORM 单元 8。分类为 14 项隔离远端数据库、1 项显式网关后端和 4 项实时外网探针。以下保留源码的原始 ignore 理由。

| 确切测试名 | 原始忽略理由 |
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

这些用例需显式选择 ignored。远端数据库还需对应 service/storage 驱动特性和指向隔离测试库的 TEST URL；仅设置 URL 不会自动执行。显式 gateway 后端可选 sqlite/mysql/postgres，sqlite 分支自行创建临时库。四项外网探针分别访问 GitHub 仓库、skills.sh/GitHub、仓库持久化安装以及外部价格目录。本轮未提供或操作这些外部资源。

另有 12 项远端数据库用例受 mysql/postgres 特性控制，没有编入默认 SQLite harness，不计通过数，也不计入上述 19 ignored；涉及 warmup、模型、reset credit、usage、import 和 preflight。逐项源码名、行号、原始属性及启用条件在 `target/r05-evidence/ignored-tests-review.json` 和同名 `.md` 中；导入/preflight 四项还需保留 sqlite 模块特性。记录中的建议命令没有执行。

## 最终交付核对

- `target/r05-evidence/final-handoff-check.json` 记录最终核对：实施与干净对照 HEAD 均仍为 `f14bd735dca9203d7430958480b82ba7713641a9`，暂存区为空，对照源码干净。
- 实施工作区恰有已审查的 49 个变更路径（冻结 Q03 的 41 个，加 R05 的 10 个，减 CI/RPC 两处重叠）；除最终更新的本报告外，其余文件与最后独立静态复核哈希完全一致。没有版本、迁移、依赖清单/锁文件或范围外文件变更。
- 最后按本会话两个 worktree 的 target 绝对路径检查进程，没有遗留测试/构建可执行进程。前端 out 和所有 Rust target 属于本会话自己的工作区，证据日志全部保留。
- 未暂存、提交、推送、创建 PR、合并、发布或改动原仓库；没有操作生产账号、外部数据库或真实系统自动启动配置。未处理 Q08 拒绝清理的 17 个文件，未与其他用户会话通信。

## 未验证项与后续汇总

- 不操作真实 MySQL/Postgres、联网技能/价格目录、真实上游或生产账号；忽略项以最终全量日志逐项记录，不能报通过。
- 桌面系统副作用仅使用模拟回归，不执行真实自动启动变更；没有跨平台系统实机验收、生产部署或远端 CI。
- Q03 数据库批次原子提交及进程内写锁不等于跨数据库、运行态、系统操作的全局事务；普通网关读取者不持设置写锁。
- R02 后续整合必须同时保留 Q03 `normalize_route_strategy()` 纯校验和 Q04 `apply_route_mode()` 幂等发布，同规范策略/别名不重置三类 map/维护计数；本工作区没有 Q04，不能据 R05 通过宣称该组合通过。
- R05 增量仍需在明确统一交接点与 R02、R03/R04 等成果汇总，并处理 CI 重叠、重跑最终组合验收。五份独立报告不等于统一版本全部通过。
