# S03：统一版本与最终验收

## 结论与最终实际结果矩阵

**本地功能验收通过。根workspace 2821/0/19、Tauri 93/0、前端319项、网关/HTTP、14/17/2 SDK回放及真实Chromium流程均通过。最终Tauri格式仍exit1，保留14文件28段既有格式债，因此不能称所有检查通过。19 ignored与12 cfg排除项均未计入通过。**

| 最终项目 | 实际结果 | 证据（target/s03-evidence） |
| --- | --- | --- |
| 根 workspace（批准保留v6） | **2821 passed /0 failed /19 ignored**；26个Cargo父target完整收尾 | exit 0；log SHA `8EBDB68D4E4A2DADFB4FB94F2A42ADBC6EB434F1ED6AAC551872D878440FDC18` |
| fixture定向（批准保留v6） | parent **65/0/0**；11个child各1/0，raw76不双算 | exit 0；log SHA `D4C2D128A877881C6F6C1F0329FD6A5F3515E745A1C69EBB9F2BF08B5E1AD281` |
| 前端runtime（批准保留导入版） | **319 passed**，0 failed/skipped/cancelled/todo | exit 0；log SHA `81161AFC9A136C56EFD041C32DD1F88CFED5525FFB39589D9F2849B43F93113A` |
| 前端build /desktop static export | 均exit0，各19页面；18个原apps输入SHA不变 | exit 0；log SHA `491199AF27FB7B59B1E67F2CEF37D642BF5C456B1F52EF19304B08D826666A26`；exit 0；log SHA `0492E6B84BDAB0ABCCDE8B831ED8A0F1BFB28E4698228EF5F5D951BF34C9E343` |
| 根workspace格式（批准保留v6） | exit0 | exit 0；log SHA `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` |
| 最终Tauri migration定向 | **4/0/0**；裸文件只读及原不覆盖断言均ok | exit 0；log SHA `272D98E9B5B2197E05CC1CDB99E4FB80FA6B04AE29C5595BAE184986885E8682` |
| 最终独立Tauri全量 | **93/0/0**；main/doc各0，无过滤 | exit 0；log SHA `5CB0D1939BA181AEB85AD4BD151925381E8B8E51846F2D4B4E9764BD0BE8548B` |
| 最终Tauri格式 | **exit1：未通过**；14文件28段既有基线格式债 | exit 1；log SHA `11498C391EB42F84BFFFFC5848ACBB1A3DC08CEB9B72D5913C9EB5008B4A9853` |
| gateway定向（v7） | parent **982/0/0**，另2 child各1/0；14对新SSE/expected | exit 0；log SHA `8C5106D1AC2CA9E783CC9C2A3307E50951F4697D509E5C38B14C6C264520CFA8` |
| gateway本地HTTP（v7） | **75/0/0**；17份nonstream+2份public HTTP stream | exit 0；log SHA `795045735CE19176CFF101E23D96D85F1847C1EA7237546C3C6EF1E0089DBF3F` |
| Web独立全量（v7） | **42/0/0**；导出资产、proxy、重定向及释放回归 | exit 0；log SHA `43977B54B5938C4AF07222141F999B43B5823F0E39D8E5C0CC520652CE591C45` |
| 运行二进制（v7） | service+Web构建exit0；hash/size/mtime绑定探针 | exit 0；log SHA `BFB2695B8A1094DC8920B692DE63B17B91CCA96ADFBAE00AAD10AB35C28A9B58` |
| SDK reader回放（v7） | 完整消费14份；无allow-partial | exit 0；log SHA `08EC47BD4AF0F7C72672E75B5944558397F94BABF51933AEB8ACF044BF6BAC43` |
| SDK nonstream回放（v7） | 完整消费17份；无allow-partial | exit 0；log SHA `A74A1D35EA9696AFE748CB581167C2B76894923C0895F23081154163434730E9` |
| SDK public stream回放（v7） | 完整消费2份；无allow-partial | exit 0；log SHA `A1CDEF052773780C45EB42C4E0006E981907624545F3FC8C7344117E9815B6AC` |
| 真实Chromium及本地service/Web（v8） | 7浏览器检查、12 Web检查、直连/proxy各5协议检查；三进程自然退出0；完整性ok | exit 0；log SHA `219CF0E30F67578DE8783F9BB31968ADFF098999A67C9D05E5963DF19A8CE2B5` |

所有通过项均有真实exit0、retained log/hash及源码前后校验。总证据审计 `audit-final-result.json` 仍为FAIL，与格式exit1一致；`final-acceptance-summary.json`仅在全部功能job、必需用例、ignored/cfg、fixture及独立runtime审计通过后给出功能PASS，并明确保留格式例外。证据复用来自父会话批准的精确源码兼容桥，不把来源历史结果当作最终重跑。

- 工作区：`C:\Users\qxnm\.codex\worktrees\round3-unified-final\Codex-Manager`；共同基线 `f14bd735dca9203d7430958480b82ba7713641a9`，创建完成、编辑前干净。
- 范围为 Q01/Q02/Q03/Q04/Q06/Q07 与 R01–R05 经 S01/S02 整合的成果；Q05/Q08/Q09 独立成果不含。
- 仅本工作树修改，未暂存、提交、push、PR、合入或发布；未处理 Q08 拒绝清理文件，未操作真实账号/用户库/凭据或系统自动启动。

## 冻结输入与合并完整性

| 输入 | 冻结文件记录 | index SHA256 | manifest SHA256 | tracked.patch SHA256 |
| --- | ---: | --- | --- | --- |
| R01 | 14 tracked + 9 untracked = 23 | `6A728FFC70DF32DB3A46ED57B5E9119D5CDA213AD1086560E13B2CD174B520CE` | `547D6EBE06323E098D50548A7418388CBE51DF00B74BCAE49DB4D788C2E98510` | `7647969BB0322C0EC4E9BAFA97A6B1ED68BC8BB30478AE229F79D48AC0B44BA3` |
| S01 | 47 tracked + 8 untracked = 55 | `22D6BB07519AF45C434782642674A89E43349CC9ABD4473C370EAE7CEC27B1D3` | `5178569194E505B47E6221D93072B8459589C4C2DDE79C8DBB2BF3B488333A47` | `2AE4856FAF64546EB079A7D432D78C0229BE7FC1D2F6E7854473BC5E5594ED16` |
| S02 | 23 tracked + 15 untracked = 38 | `0DA716B4789AEBCA7E565ED61DDE013274ACB28A433B34566A3C518C67F24089` | `647997E7A333F43DAE8BEE3FED8AD48CA14AC9629E332A6DD698EF6A3DF35B78` | `AC8CD0D44FEA1875C716773975DF1ED063D1164003FE57E58FAC7D2987415D7E` |

冻结根分别为 `.codex/handoffs/code-quality-2026-10-17-round3`、`code-quality-2026-10-17-final-s01`、`code-quality-2026-10-17-final-s02`。未读取动态来源树，也未复制来源 target、依赖或旧日志。

- 合计 **116 份文件记录，并集 113 路径：100 非文档 + 13 来源文档**。所有 116 份冻结副本均逐文件核验 raw SHA256；97 个不重叠非文档文件在导入阶段保持来源原字节。
- 仅三个共享路径：`.github/workflows/ci.yml`、`gateway_logs/support.rs`、`gateway_logs/stream_timeout_failover.rs`。
- CI 三方干净合并，SHA256 `CC2AFD320478214BC02CEE5940667CB6619E47F8F7E7459DA9FCDE8A29AF96FD`。YAML 可解析，完整保留 35 个 step 的字段并集、S01 独有 3 步、S02 独有 8 步，含三个 SDK、fixture 环境/Python 顺序及完整 `gateway::route_hint::` 检查。
- stream timeout 两方内容完全相同。support 三方预计算出现两个冲突，首次导入 exit 1 且未写源码；确认 S02 仅增加 read_phase 分类、脱敏输出和回归，逆向移除增量后全文等于 S01，才接受 S02 字节，最终导入 exit 0。
- R01 首次导入与 S01 部分文件存在 Git CRLF 转换；先确认规范全文相等，再恢复冻结原字节。该换行诊断与 support 冲突均保留历史记录，不能作为测试失败或运行通过证据。

证据：`target/s03-evidence/r01-import.json`、`r01-import-history.json`、`s01-import.json`、`s02-import.json`、`s02-import-history.json`、`s02-merges/`、`ci-final-static.json`、`source-imported-original.json`。

## 最终源码与证据复用边界

最终v8 sourceDigest：**`97F808F2FCF9F7AD4B9136443547ECC080B1EE890A4C8B75F079734F0CB44A8F`**。`source-repaired-final-v8.json` SHA256 `A24515404372407CA92D4EBB36B70495AF926E2068FC7140AB64BFFE305536B7`；**114非文档+13来源文档=127 manifest路径，另S03报告合计128 Git变更路径；15个accepted repair paths**。

- v6 digest `1172A71F27451109F980732C5EEB828E1C45BA2F1F71A2B7E6CBA60642529C31`。旧111非docs到v7逐SHA相同；v7只新增独立Tauri migration.rs、Cargo.toml和Cargo.lock。只允许指定v6三个stem复用：workspace-backend-isolated、fixture-isolation-backend、fmt-workspace-backend-isolated。
- v7 digest `E4B803158DA41BEAF7E25A770CC608BFB97E09086198D8AD8A8BE84310F753DE`。v8仅把新增checkpoint的map_err按rustfmt展开，余113非docs逐SHA不变；唯一字符串替换严格核验。父批准保留不依赖独立Tauri的v7 gateway/HTTP/Web/build/SDK；最终Tauri定向/full/fmt及新的runtime-probe-zstd在v8执行。
- 前端三项来自导入版digest `914CE49D6E55AA416572DD48F08209E6CF3F221E829D5ABFA2CB8239B0F0A5C5`，18个原apps输入到最终逐SHA相同。独立Tauri测试依赖及Rust修复不影响Next输入；按明确批准保留，不能称v8重跑前端。
- Tauri新增dev SQLx为已锁0.8.6，features镜像shim已有依赖；独立Cargo.toml仅新增dev条目，独立Cargo.lock仅CodexManager依赖列表增加sqlx。没有新增crate或core/shim改动，根Cargo.lock仍为v6字节。兼容性详见v7-source-compatibility-review.json及最终审计逐项sourceEvidence。

验证器记录完整argv、UTC起止、真实exit、日志SHA和源码前后hash。重型任务顺序执行，编译-j2、测试默认并发；无RUST_TEST_THREADS/--test-threads，无新增skip、业务时限放宽或弱化断言。清除继承服务/DB/代理变量，仅用target内临时库、假token、TEMP/TMP；不读真实用户库。依赖安装不是SDK/浏览器/运行通过证据。

## 必须保留的运行历史

| 版本 / 运行 | 真实 exit | 实际结论与证据（`target/s03-evidence/`） |
| --- | ---: | --- |
| 导入版首次 workspace | 101 | 2814/4/19；四个 Web 资产测试当时无 apps/out（3个HTTP404/200断言与1个缺资产panic，非4个HTTP404）；`workspace-result.json`/log |
| 导入版第二次 workspace | 101 | service 在 32-request 屏障挂起后终止；四个 FAILED 标记保留。867/0/11 仅完成 harness 的不完整统计；`workspace-final-*` 与 hang-termination JSON |
| 首次 fixture 定向 | 101 | E0433 编译失败，0 tests 执行；`fixture-isolation-result.json`/log |
| 编译修正后定向 | 101 | parent 53/1/0，另 5 child；唯一 WebSocket capture 缺 model_groups。raw 58/1 不双算；`fixture-isolation-final-*` |
| v3 / v4 定向 | 0 / 0 | 各 parent 55/0/0 + 6 child 各 1/0；raw 61 不作 parent；`fixture-isolation-repaired-*` / `fixture-isolation-verified-*` |
| v4 完整 workspace | 101 | **canonical 2820/1/19**，26 个 parent 完整收尾；service 1953/1/8，唯一 legacy slug migration lock would block；`workspace-repaired-canonical-review.json` |
| v5 定向 | 0 | **parent 56/0/0 + 7 child 各 1/0**，raw 63 不双算；`fixture-isolation-storage-*`，log SHA `BDF0D9ECE654DA30C09F73ABB805F08E001E905AAFBB9D06FB9CF6BF1FA5D568` |
| v5 完整 workspace | 101 | **canonical 2820/1/19**，26 个 parent 完整收尾；service 1953/1/8，唯一 auth identity/mysql URL 失败，legacy migration 本轮 ok；`workspace-storage-isolated-canonical-review.json` |

v5 完整 log SHA256：`3E73BDD08C28CDD169D1E638C426B4C4366D42746A58305B47D33604013F536D`。唯一失败 `auth::tokens::tests::resolve_existing_account_for_login_with_tags_uses_identity_candidates`（1656）；panic 指向 `auth_tokens_tests.rs:262:6`（3300），`resolve account: "database URL is required for mysql"`（3301），service closing 3307；legacy migration ok 为 3064。

v4/v5/v6 独立 parser 均逐 Cargo target 最后 parent summary 平衡首 registered count；正常 FAILED 为 **COMPLETE execution / FAIL acceptance**。各完整日志均只有 26 个 parent summary、无额外 child summary；child 成功输出被捕获，既不从 parent canonical 中扣除 child，也不额外相加。v6 `workspace-backend-isolated-canonical-review.json` 独立确认为 2821/0/19（service lib 1954/0/8、Web 42/0/0），11 child 的可见独立结果取定向日志单列；先前八个失效点均在本轮完整 workspace 为 ok。workspace canonical JSON保存所有target和19 ignored名称/原因/行号；最终audit-final-result.json另外列出12个feature cfg排除项，二者均不计通过。来源历史另见输入报告（R02 2730/1/19、R05 2728/0/19、S01 preflight 3/1 与 workspace 2741/1/19、R03 954/2、R04 60/6、S02 955/2），不计入本轮通过。

Tauri新增历史均保留原result/log：

| 运行 | 真实exit与结果 | 证据及边界 |
| --- | --- | --- |
| v6 fmt-tauri | exit1，14文件28段 | 当时migration未改；log SHA `06C9374E9151E959655A0CCFEC1C150EDB92ECF53776038A7428F5504DEC90E0` |
| v6 tauri-full | exit101，91 passed/1 failed | 唯一原never-overwrite字节断言；log SHA `279DE34EF3B696C0F86AE04F118BF4A830A1CEC1655A7226AC2D1B9C49AF4DE4` |
| v7 Tauri定向/full | exit0，4/0与93/0 | checkpoint及真正只读回归后通过；最终另在v8复验 |
| v7 fmt-tauri-checkpoint | exit1，同14文件但29段 | 1段来自新增map_err，不能全归基线；log SHA `DB980CE0AFCE5F4F6FE94883C1F7925817D17D902A8E8920DB70EA3FC23911AE` |
| v8 fmt-tauri-formatted | exit1，14文件28段 | 仅修复新增map_err格式；原migration两段按hunk内容保留，最终基线债未通过 |
| v7 runtime-probe | exit1，4.172秒 | fixture aiohttp缺backports.zstd，zstandard解码报400，未进入browser；log SHA `D198CB0ADA2C43CD7CD735AFF9FB1956DF9C7AF9BE503864CAA00271F15F01A6`。已有init/service自然exit0、processes_exited=true；非应用协议失败结论 |
| v8 runtime-probe-zstd | exit0，9.657秒 | 仅在target/s03-python补backports.zstd1.8.0，import确认HAS_ZSTD=True；新report/fixture目录、原脚本/断言不变，真实browser与协议通过 |

## 修复事实与严格因果边界

- 插件与 transport 本地 fixture 使用既有环境锁及 RAII no_proxy client/cache 恢复；保留原 20 秒代理策略、32 请求并发、fast 2 秒与全部业务断言。绝对期限覆盖启动与排空，异常先释放/abort/drain/回收，再恢复原 panic；生产 builder 未因此扩大时限。
- WebSocket capture 库先私有准备再发布；73728B migration `.bak` 仅初始 9 表，证明某次 initialize_storage 观察到半成品，但 precise interleaving 未捕获。v5 仅给故意旧 schema fixture 加 envguard 前的 bounded exact-child 入口，并为 custom_market_without_source 补 envguard；修复前只有该 custom 分支会无锁 openDB，显式 builtin 不读 DB。
- **v6 四个 backend mutator 已实施 exact-child 入口隔离**：`incomplete_remote_configuration_cannot_fall_back_to_sqlite`、`pool_survives_operations_and_rejects_live_backend_changes`、`incomplete_remote_configuration_never_authenticates_or_reads_local_keys`、`stored_proxy_test_does_not_read_sqlite_when_remote_configuration_is_incomplete`。入口均在既有 envguard 前；移除新增 supervisor 前缀后，四个文件均与 before 快照逐字节相等（独立 JSON 亦复核），原 body/权限/错误断言保留。
- auth 测试和身份断言未修改，仍要求 `subject-fallback`。v6定向和workspace已通过，最终Tauri/协议/SDK/浏览器功能也已通过；基线Tauri格式仍单列为未通过。实际 backend 写入 owner、owner 栈、精确交错仍未知，不能把候选 mutator 写成唯一历史根因，也不能宣称所有 reader 已隔离。
- 旧 binary 固定 SHA `2118C7237282E9D2CBB1A4575876543F8DA85B362878E534A268AA21DA0CFC64` 代理负控 0/1 exit101、NO_PROXY 正控 1/0 exit0；无强杀、无外部转发，进程回收。证据 `transport-proxy-controls-old-binary/`；负控是预期观察而非通过，`historicalCausalityEstablished=false`，不能替代修复后源码验收。

- **Tauri备份完成边界修复**：Backup完成SQLite事务，但shim目标固定WAL且SQLx池Drop异步归还/关闭，返回不保证main文件已独立完整。旧日志首读4096B（1page），二读262144B但header已标341pages；同version第二次调用仅early-return、没有open/copy，支持完成边界缺口而非覆盖路径。未声称捕获唯一线程时序。
- `copy_db_snapshot`只在drop(backup)后对目标query_row执行TRUNCATE checkpoint并读取busy/log/checkpointed；SQL/读取错误、busy非0或帧不等均Err。源库不checkpoint、不改core/shim/全局pool关闭，也无新增sleep/retry/业务timeout。新回归保持source活跃，返回后立即仅copy主.bak至新路径，前后无wal/shm；direct SQLx read_only(true)+immutable(true)验证账户acc-1与integrity ok，await close后断言结果。原version/path/never-overwrite字节断言完整保留。保证限于函数返回时独立主文件可读，不额外声称fsync断电持久性。

## 协议、SDK与真实浏览器证据

本轮fresh target/s03-final-stream生成14对SSE/expected，target/s03-final-http生成17份nonstream与2份public HTTP stream。reader覆盖large interleaved UTF-8/CRLF、多choice/tool；HTTP覆盖Responses/Chat、stream/nonstream、tools/tool_calls、图片/base64、终结/预算/取消/错误等必需用例。主审计逐case检查并绑定fixture hash/size。官方openai2.29.0/httpx0.28.1通过httpx.MockTransport消费本轮Rust/本地HTTP产物，无allow-partial；SDK未直接连接Rust网关或真实上游，不称直连E2E。

runtime-probe-zstd使用本轮service/Web exe及apps/out，--duration 0 --browser仅为本地功能验收。独立审计绑定build/probe/current/fixture二进制hash、size、mtime及探针脚本SHA，检查真实导出资产、12 Web检查（含权限/成员key归属/SQLite读回）、Chromium7检查（匿名重定向、引导、真实bootstrap/React、登录会话、fixture API key页RPC、logout撤销、无未捕获错误），直连service及Web proxy各5协议检查。WebSocket关闭1000；initialization/service/Web均自然exit0，无force/shutdown_error，processes_exited=true，SQLite integrity=ok，request_logs_readback=10。Chromium版本145.0.7632.6，RPC200共13次/401共1次、无page_errors；逐项结果见runtime-probe-zstd.json与fixture/browser-report.json；独立runtime审计PASS，共697项。

R01的Node真实HTTP、mock IPC、native TCP与Web handler属于分层证据，未证明frontend IPC→native TCP全链路。safe desktop方法保留caller/native两层有限重试；Web生产proxy无body总deadline（250ms仅测试注入），abort future不证明浏览器断连传播；native per-I/O timeout不是整包绝对deadline，frontend取消不能中止已开始的native业务，resetCredit600秒保留。S01设置锁不是DB/runtime/OS全局事务，OS副作用为mock闭包；SSE byte budget不是RSS实测。

## 忽略项、范围与最终收尾

19个实际ignored保持原名/原因，涉及隔离MySQL/PostgreSQL/显式网关后端及live仓库/skills/价格源；12个feature cfg排除项未编入此次二进制，单列且不算通过。完整名称/原因/行号与cfg来源见audit-final-result.json及workspace-backend-isolated-canonical-review.json。真实账号/凭据/用户DB、真实上游/外部资源、真实OS设置副作用、跨平台和远端CI未验证；本地Chromium流程已验。Q05/Q08/Q09继续排除。

CI YAML解析和35step完整字段并集通过，含S01独有3步/S02独有8步及三个SDK顺序，CI文件SHA `CC2AFD320478214BC02CEE5940667CB6619E47F8F7E7459DA9FCDE8A29AF96FD`；仅为静态结构证据，不称远端CI成功。新增diff/未跟踪文件有限secret signature审查无命中，仅保留路径/种类不输出值，不声称证明所有秘密均不存在。最终状态见final-state.json：HEAD保持共同基线、index为空、128路径均在批准集合、114非docs无漂移、git diff --check exit0、精确工作树exe/argv匹配无残留build/test/service/browser进程。

**最终交付：本地功能验收PASS；Tauri格式门禁仍FAIL（14文件28段既有基线债）。未暂存、提交、push、PR、合并、发布或自行导出handoff；冻结交接由父会话处理。**
