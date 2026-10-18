# R02：设置一致性与调度组合验收

## 工作区与交付范围

- 日期：2026-10-17。
- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\round2-settings-routing\Codex-Manager`。
- HEAD / 共同基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。
- 修改前 `list_artifacts` 为空；按指定 ref 创建工作区，创建操作 `b493ec51-c659-48cc-ba01-421b851dc001` 返回 completed 后检查 HEAD 和干净状态，再导入。
- 原仓库两份任务书仅按指定绝对路径只读读取；原仓库、第一轮来源工作区与冻结目录均未修改，也未读取其他任务不断变化的工作树。
- 所有代码、测试、报告和构建产物都位于本工作区；测试仅使用临时 SQLite、临时目录和 mock 上游。没有操作真实账号、数据库、现有服务或系统自动启动项。
- 不提交、不暂存、不推送、不创建 PR、不合入主仓库、不发布；保留可审查的本地改动。
- 当前结论：Q03 + Q04 已实际整合，组合新增环境重载缺陷已修复，定向设置/调度和完整测试中的相关断言通过；**完整 workspace 未通过**，R05 登记的一个网关测试在本组合再次失败，不能宣称统一版本全绿。

## 冻结输入与导入

冻结根目录：`C:\Users\qxnm\.codex\handoffs\code-quality-2026-10-17-round2-3e2657e8`。

| 输入 | manifest SHA256 | tracked.patch SHA256 | 核验 |
| --- | --- | --- | --- |
| Q04 | `15DC5AA55915989968954B8E3F17042B09CEF5C194613D721E0B2C1769523246` | `DFE358A066F49C9B08153C308CBD43D16F742807D35BC775725CE4FA0374DF91` | 基线、patch、2 tracked、2 untracked 全部符合 manifest |
| Q03 | `222EC98D5A808F6547A9F622E16DA255F2BF8E1468CEDB08109A8AE33107034C` | `08ECC10D851D9CE1071B059ECD4AD9C84A32E534E8B6B1C31371BC4513C4A0B4` | 基线、patch、39 tracked、2 untracked 全部符合 manifest |

逐文件 SHA 与导入后的换行归一化文本对照保存在 `target/r02-evidence/inputs-verified.json`；Git `core.autocrlf=true` 导致部分导入文件的物理字节与冻结文件换行不同，没有用物理 hash 差异冒充源码冲突。输入的两个第一轮报告原样保留在 `docs/zh-CN/report/code-quality-2026-10-17/`，它们属于来源证据，不替代本次验证。

1. Q04：`git apply --check <Q04>/tracked.patch` exit 0 后 apply exit 0，复制 manifest 的全部新增测试和报告。
2. Q03：完整 `git apply --check <Q03>/tracked.patch` exit 1，唯一失败文件为 `crates/service/src/gateway/routing/route_hint.rs`。排除此文件再 check/apply 均 exit 0，复制全部新增文件，没有删除项。
3. 对照 `git show HEAD:crates/service/src/gateway/routing/route_hint.rs`、当前 Q04 文件及 Q03 `tracked/` 全文件逐处整合。保留 Q03 的 `normalize_route_strategy()` 纯校验，setter 调用该校验后通过 Q04 `apply_route_mode()` 幂等发布；环境 reload 同样复用幂等入口。
4. `route_hint_tests.rs` 的 Q03 校验测试可正常应用；保留 Q04 全部状态/别名/真实切换回归，没有覆盖整文件。

## 组合问题与新增回归

首次组合回归真实复现两个新增一致性缺陷：在运行态 balanced、策略环境值缺失时，无关环境 patch 以及 GET 补载持久化环境值都会先 reload 成默认 ordered，再恢复 balanced。Q04 的同值幂等入口无法阻止这两次真实模式变化，A、B 后实际返回 A，严格期望 C 的断言失败。

修复没有通过临时写进程环境或保存旧模式后重新 setter 补偿：

- `route_hint.rs` 抽出辅助配置重载，新增 preserving 入口只更新 P2C 窗口、TTL、容量及原有 manual preferred 缓存失效；不读取旧模式再发布，因此不覆盖并发 setter 的较新模式。
- `gateway/mod.rs` 的统一 reload 内部函数保留其他模块原有顺序，并支持保留路由策略的调用入口。
- `env_overrides/process.rs` 对不涉及路由策略的环境发布使用 preserving 入口。GET 同步补载环境时，策略环境值缺失/空白也使用 preserving 入口，随后仍由 `runtime_sync` 发布真实持久策略；非空显式环境值继续走正常 reload 和既有优先规则。
- setter 与正常 reload 继续用 Q04 幂等 `apply_route_mode()`；真实策略改变继续清三类调度 map 和维护计数。无关 reload 不再临时切换策略。
- CI 的既有设置步骤将单个纯校验筛选扩展为整个 `gateway::route_hint::`，纳入 Q04 与 R02 组合回归，未改发布流程。

新增组合测试位于 `crates/service/src/gateway/routing/tests/route_hint_settings_tests.rs`，保留原 4 项并增加 6 项：

- A、B 后连续 GET、无关 UI patch、canonical/别名批量保存、legacy setter 和回读均继续 C；真实切换 ordered 后切回 balanced 才重新从 A、B 开始。
- runtime balanced 且策略环境变量缺失时，无关环境 patch 和 GET 补载持久化环境值均继续 C。
- 空白策略环境值、显式别名环境优先和数据库策略真实变化。真实重置测试先将自然轮询推进至本应返回 B 的位置，再切换策略断言 A，避免把正常回绕误认成重置。
- 非法混合 patch 与事务中段 trigger 故障比较完整设置行（含时间戳）、不触发 reload 的运行态快照、进程环境及下一次 C，避免 GET 修复运行态掩盖提前发布。
- 每例以本测试可执行文件 `--exact` 子进程执行，隔离进程生命周期的 `ENV_OVERRIDE_BASELINE` 和存储缓存；未增加 ignore、未降低全量测试并发。

首次测试还发现两项新用例业务断言结束后的 Windows 临时目录清理失败：直接 SQLx 连接池的 Drop 不保证当场释放文件句柄。修正为父进程创建并拥有临时目录，子进程恢复运行态/环境，父在 `.output()` 确认子进程退出后恰好清理一次并断言结果。没有 sleep、重试、跳过或吞清理错误，也没有修改数据库连接/迁移/备份实现。首次失败日志和当时源码副本保存在 `target/r02-evidence/route-settings-before.log` 与 `before-reload-fix/`，首次失败临时资源路径记录于 `initial-fixture-resources.json`。

## 验证记录

命令工作目录均为本工作区。Rust 使用本工作区自己的 `target`、`-j 2`，各阶段串行运行；前端构建在 core/SeaORM 窄测试完成后、service 首次构建前错峰执行。设置了仅属于本次验证的 `CODEXMANAGER_DB_PATH` / `CODEXMANAGER_TEST_DB_DIR` 和 SQLite 默认后端，测试内部继续使用各自临时资源。完整输出位于 `target/r02-evidence/`，未使用来源旧日志替代本次验证。

| 实际命令 | 退出码 / 通过、失败、忽略 | 日志 |
| --- | --- | --- |
| `cargo test -p codexmanager-core -j 2 storage::settings::tests` | 0；5 / 0 / 0（其他 target 的 0 用例筛选未计为通过） | `core-settings.log` |
| `cargo test -p codexmanager-storage-seaorm -j 2 settings` | 0；7 / 0 / 0 | `seaorm-settings.log` |
| `cargo test -p codexmanager-service --lib route_hint::settings_tests -j 2 -- --nocapture`（修复前） | **101；5 / 4 / 0**。两项 C→A 产品失败，两项 Windows 目录清理失败；子进程失败输出会重复打印，不能重复累计子进程统计 | `route-settings-before.log` |
| `cargo test -p codexmanager-service --lib route_hint -j 2 -- --nocapture`（修复后） | 0；25 / 0 / 0，包含全部 10 个设置/调度组合用例 | `route-hint-after.log` |
| `cargo test -p codexmanager-service --lib app_settings -j 2` | 0；20 / 0 / 0，与路由筛选有 2 项重叠，不累计成独立用例总数 | `app-settings-lib.log` |
| `cargo test --workspace -j 2 --no-fail-fast` | **101；2730 / 1 / 19**，未传 `--test-threads=1`；开始 19:29:59，结束 19:50:12（Asia/Shanghai） | `workspace.log`、逐 target 汇总 `workspace-summary.json` |
| `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2 commands::settings::ui::tests` | 0；6 / 0 / 0；系统操作用模拟闭包，无真实自动启动修改 | `tauri-settings.log` |
| `cargo test -p codexmanager-web -j 2` | 0；38 / 0 / 0；包含认证缺失哈希拒绝、cookie、HTTP RPC proxy、SSE/WebSocket mock 回归 | `web.log` |
| `pnpm.cmd -C apps install --frozen-lockfile --offline --ignore-scripts` | 0；647 包复用缓存、0 下载，无锁文件/依赖清单修改 | `frontend-install.log` |
| `pnpm.cmd -C apps run test:runtime` | 0；250 / 0 / 0，cancelled / todo 均 0 | `frontend-runtime.log` |
| `pnpm.cmd -C apps run build:desktop` | 0；Next 编译、TypeScript 检查、19/19 页面生成完成，路由表列出 17 条静态路由（含 /_not-found） | `frontend-build-desktop.log` |
| `cargo fmt --all -- --check` | 0 | `fmt-workspace.log` |
| `rustfmt --check --edition 2021 apps/src-tauri/src/commands/settings/ui.rs` | 0 | `fmt-tauri-settings.log` |
| `git diff --check` | 0 | `diff-check.log` |
| `node target/r02-evidence/validate-ci.cjs` | 0；使用本工作区 js-yaml 4.1.1 解析并断言 route_hint、atomic、Tauri settings 入口 | `ci-validation-final.log`（前次等效 inline 校验另存 `ci-validation.log`） |

`apps/package.json` 的 `build` 与 `build:desktop` 均为 `next build`，已执行后者，不重复相同构建。未改前端导航/页面缓存，不额外执行 `pnpm.cmd -C apps run test:navigation`。仓库没有适用于此次设置/鉴权变更的独立 Web runtime probe 脚本，Web HTTP/proxy mock 回归由 codexmanager-web 测试入口执行；不以静态页面导出声称完整 Web 部署或浏览器端到端通过。

一次证据脚本参数绑定错误未启动 Cargo，输出在 `seaorm-settings-harness-error.log`；修正为显式参数后才执行上表 SeaORM 命令。原错误记录仍在 `results.jsonl` 并追加日志改名说明，没有计为测试结果。

### 完整 workspace 的实际结果

完整命令保留默认测试并发。core 单元 530、core 集成 48、service 单元 1891、新旧设置集成 37、RPC 50、SeaORM 51、Web 38 等实际通过。service 单元 8、外部 service 集成 3、SeaORM 8 项忽略，共 19；需要真实隔离 MySQL/PostgreSQL、指定外部后端或联网条件，未开启，不算通过。逐测试忽略原因及各 target 完整统计见 `workspace-summary.json`。

唯一失败：`stream_timeout_failover::gateway_single_silent_account_still_obeys_stream_header_timeout`，`crates/service/tests/gateway_logs/stream_timeout_failover.rs:321:5`，断言 `single-account transport recovery must stay bounded: []`。该测试 target 为 56 passed / 1 failed，完整 target 未通过；该日志保留了失败 stdout、准确测试名、行号与最终 `error: 1 target failed`。此前的 502、200ms 错误提示、耗时边界断言已越过，实际失败发生在读取 mock 记录后的 1..2 次数断言，与 R05 登记现象一致。本轮没有修改该测试或其 mock fixture，也没有用复跑替换失败记录或使用 R05 工作区结果。

R05 另一个登记项 `transfer::preflight_tests::failed_checksum_and_connection_preserve_source_and_allow_clean_restore` 在本次默认并发全量中实际通过；这不证明原问题已修复或不会再出现。本工作区没有改动迁移/备份/恢复逻辑，也未复制活动 SQLite 主文件或 WAL 充当一致性快照。

Tauri 独立 manifest 设置测试实际通过 6 项，覆盖保存失败不执行系统操作、系统操作失败明确已保存/重试语义和成功顺序。根 workspace 明确不含 Tauri，本报告采用单独运行结果，不用根结果代替桌面验证。

独立 Web package 也实际通过 38 项，包含本轮导入的密码哈希缺失拒绝与既有 HTTP/RPC、SSE、WebSocket mock 代理回归。没有执行真实 HTTP 并发认证或浏览器完整端到端，不能将 helper/本地 mock 覆盖扩张为部署验收。

## 相对 Q04 + Q03 输入的额外增量文件

| 文件 | 本轮额外内容 |
| --- | --- |
| `crates/service/src/gateway/routing/route_hint.rs` | 同时整合 Q03 纯校验和 Q04 幂等发布；再拆辅助配置重载与 preserving 入口 |
| `crates/service/src/gateway/mod.rs` | 共享完整 reload 内部实现，增加保留策略的入口 |
| `crates/service/src/app_settings/env_overrides/process.rs` | 环境发布/补载调用正确入口，不再临时切模式后恢复 |
| `crates/service/src/gateway/routing/tests/route_hint_settings_tests.rs` | 原 4 项外加 6 项组合回归、每例进程隔离及父进程负责清理 |
| `.github/workflows/ci.yml` | Q03 已有设置步骤中，纯校验单例扩展为 route_hint 全组 |
| 本 `R02-result.md` | 本次组合验收结果、输入校验和实际证据边界 |

`route_hint_tests.rs` 仅整合双方已有测试，没有额外 R02 产品改动。Q03 的其余输入文件均保留其导入语义；两份来源报告原文未修改。最终 scope 检查记录于 `target/r02-evidence/scope-review.json`：39 个已跟踪文件修改、5 个新增文件（2 个新增测试、2 个第一轮报告、本报告），没有版本、锁文件或迁移变更；暂存区为空，HEAD 仍为基线。

2026-10-17 20:00（Asia/Shanghai）最终 `git diff --check` / 暂存区检查均 exit 0。本工作区路径下的可执行进程检查为空，测试和 mock 服务已退出，见 `target/r02-evidence/final-process-check.json`。构建告警保留在各日志，没有因告警改动无关源码。首次失败 fixture 的两处临时目录路径保存在 `initial-fixture-resources.json`，作为失败资源证据保留；后续成功组合用例由父进程一次性严格清理。

## 一致性与后续汇总边界

- Q03 的数据库批次原子提交、全部校验后持久化、提交后逐字段发布、不同字段并发合并与明确副作用失败语义须同时保留。
- `SETTINGS_WRITES` 只协调本进程设置入口，普通网关读者不持该锁；运行态仍逐字段发布，可能看到过渡状态。它不是跨数据库、内存与系统操作的全局事务，也不是所有读者的原子快照。
- 真实策略切换时，已在获取游标锁前读取旧策略的请求仍可能按旧策略继续；preserving 入口消除了无关 reload 的暂时切换，没有扩张为所有请求同时切换策略的保证。其他运行态仍会广泛重载并逐字段恢复。
- 提交后发布/响应/桌面系统副作用失败不会回滚已提交数据库或已执行的先前字段，保留 `settings_persisted_runtime_apply_failed`、`settings_persisted_response_failed` 与桌面已保存提示；没有弱化 Web 密码、认证、分发与计费锁。
- 真实 MySQL/PostgreSQL、真实上游、跨平台系统自动启动和图形浏览器端到端未验收；SQLite 上实际执行 SeaORM 路径与 MySQL/Postgres SQL 生成检查不等同真实外部存储验收。
- R05 后续增量不在本工作区；R02 与 R05、R03 与 R04 以及共享 CI 的最终统一整合仍须重新验收。独立任务完成不等于统一版本全部通过。
