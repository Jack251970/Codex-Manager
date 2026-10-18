# Q06：Web 请求完整响应超时与取消

## 状态与基线

- 日期：2026-10-17。
- 状态：实现完成；本任务前端与 Web 壳验证通过，等待审阅/与 Q01 整合。仅本地改动，未提交、推送、合并、创建 PR 或发布。
- 基线：`f14bd735dca9203d7430958480b82ba7713641a9`（0.6.4）。创建后 `git rev-parse HEAD` 与该 SHA 一致，初始 `git status --short` 为空。
- 独立 managed worktree：`C:\Users\qxnm\.codex\worktrees\quality-q06\Codex-Manager`。编辑前检查 `list_artifacts`（空），通过 `create_worktree(ref=上述 SHA, name=quality-q06)` 创建，等待 completed 后才读写工作区。
- 已读取原仓库总任务书的执行约定、Q06、交付与整合顺序，以及本工作区根和 `apps/AGENTS.md`。原仓库总任务书及产品代码未修改。

## 根因与调用面

基线 `fetchWithRetry` 在 fetch 返回响应头后清除 timer，并在 finally 中移除父 signal 监听。RPC 随后才执行 `response.json()` 或错误体的 `text()`，运行时探测也在 helper 返回后执行 JSON 读取；这些阶段不受原 deadline/父取消约束。HTTP 重试还直接丢弃前一个 Response，没有主动释放响应体。

错误体读取中的宽泛 catch 会吞掉读取失败，然后尝试第二次 `json()`，可能把取消变成普通 HTTP 错误。`init.signal` 原先也被内部 signal 覆盖，只有 `options.signal` 被监听。

完整检索确认生产调用仅有 RPC（`transport.ts` 两处传入 `postJsonRpc`）和运行时探测。账号测试和用量刷新 SSE 分别由 `account-test-events.ts`、`usage-refresh-events.ts` 使用 EventSource，不经过此 helper。

## 实施内容

| 文件 | 改动 |
| --- | --- |
| `apps/src/lib/utils/request.ts` | 增加泛型第四参数 `ResponseReader<T>`，同一次尝试的 deadline 覆盖 fetch、响应体读取和解码；受控 await 同时竞速取消，不依赖 fetch/reader 一定响应 signal；解码后检查单调时钟，拒绝超过 deadline 的同步解析结果。合并 `options.signal`/`init.signal`；取消优先，退避为 0 也检查取消。重试前 abort 并尽力 cancel 前次 body，不等待不合作的 cancel promise；晚到 Response 同样释放。所有退出路径清理 timer/监听。 |
| `apps/src/lib/api/rpc-http.ts` | fetcher 改为执行第四参数 reader，成功 JSON 和错误文本/JSON 都在生命周期内读取。只返回数据后才执行解包、业务错误处理和登录跳转；发送前及消费结果前复查调用方取消。错误体读取失败原样抛出，不吞取消、不重复消费 body。 |
| `apps/src/lib/api/transport-runtime.ts` | 探测 JSON 在 reader 内完成，再规范化和缓存；失败状态的未读 body 由 helper 释放，迟到结果不能覆盖已缓存的回退状态。 |
| `apps/tests/request-utils.test.mjs` | 扩充至 20 项请求控制回归，转译临时目录在加载后清理。 |
| `apps/tests/rpc-http.test.mjs` | 扩充至 20 项 RPC 回归；适配四参 fetcher，使用真实 helper，包含临时 loopback HTTP 服务。 |
| `apps/tests/transport-runtime.test.mjs` | 新增 4 项运行时探测/缓存回归。 |
| `docs/zh-CN/report/code-quality-2026-10-17/Q06-result.md` | 本交付说明。 |

无 reader 的重载仍在响应头处转移 Response 所有权，调用方负责后续流消费/取消；不会把 JSON 总时限强加给流式请求。两个现有 SSE 消费方没有变更。reader 必须只读取/解码，不执行缓存写入、跳转或后续业务请求，避免不合作 reader 晚完成时产生副作用。

维持基线的重试次数和 HTTP 状态策略。新增受控体读取发生 deadline 时可按已配置次数重试；JSON 语法错误、非超时读取失败、RPC/业务错误不会新增重放。超时仍为每次尝试的完整响应时限，重试及退避沿用原有语义。

## 实际验证

除表中注明外，命令均在上述独立 worktree 根执行。未复用其他 worktree 的可写输出目录。

| 命令 | 实际结果 |
| --- | --- |
| `pnpm.cmd -C apps install --frozen-lockfile --child-concurrency=2 --network-concurrency=4` | exit 0，安装本工作区依赖，lockfile 未改。 |
| `node --test --test-concurrency=2 tests/request-utils.test.mjs tests/rpc-http.test.mjs`（`apps/`，改动前） | exit 0，基线已有测试 6/6。已有测试未覆盖 headers 后的响应体生命周期。 |
| `node --test tests/request-utils.test.mjs`（`apps/`） | exit 0，20/20。 |
| `node --test tests/rpc-http.test.mjs`（`apps/`） | exit 0，20/20。 |
| `node --test tests/transport-runtime.test.mjs`（`apps/`） | exit 0，4/4。 |
| `node --test --test-concurrency=2 tests/request-utils.test.mjs tests/rpc-http.test.mjs`（`apps/`，改动后） | exit 0，40/40。 |
| `pnpm.cmd -C apps run test:runtime --test-concurrency=2` | exit 0，288 passed / 0 failed / 0 skipped，包含上述 44 项 request/RPC/runtime 用例。新测试由现有 `node --test` 入口自动发现，无需改 CI 筛选。 |
| `$env:CIRCLE_NODE_TOTAL='3'; $env:RAYON_NUM_THREADS='2'; $env:UV_THREADPOOL_SIZE='2'; pnpm.cmd -C apps run build` | exit 0，TypeScript 检查及 19 个静态页面导出成功，页面生成使用 2 workers。`build` 与 `build:desktop` 在当前 package.json 中均为 `next build`。 |
| `pnpm.cmd -C apps exec eslint src/lib/utils/request.ts src/lib/api/rpc-http.ts src/lib/api/transport-runtime.ts tests/request-utils.test.mjs tests/rpc-http.test.mjs tests/transport-runtime.test.mjs` | exit 0。 |
| `cargo test -p codexmanager-web -j 2 -- --test-threads=2` | exit 0，36 passed / 0 failed / 0 ignored。首次独立构建耗时 7m34s，测试 1.26s；输出目录为本 worktree 的 `target/`。编译有 service 未使用导入 warning，对应文件本次未改。 |
| `git diff --check` | exit 0。 |

验证证据与边界：

- 真实 Node fetch + `127.0.0.1:0` HTTP：200 与 401 立即发送响应头及未完成 JSON 前缀，分别验证完整响应 deadline 和 headers 后父 signal 取消。确认每例仅一次请求、无登录跳转；finally 销毁本例 socket 并关闭本例服务。
- 受控流/reader：成功 JSON 与错误文本均覆盖 `options.signal`、`init.signal`；body timeout 后下一次尝试可成功；退避及零延迟取消均不启动下一次请求；429/503 body 在下次 fetch 前释放，即使 cancel promise 永不结算也不阻塞。
- 正常约 1.4 MB 分块 JSON 完整解码；模拟同步解码超过 deadline 后不返回结果。JavaScript 无法抢占正在执行的同步 JSON 解析，本实现保证解析后的过期结果不交给业务层，不声称能中途终止同步解析。
- 不合作 fetch 晚返回时 body 被取消且 reader 不启动；不合作 JSON/错误文本晚完成时不写消费方缓存、不发送依赖该结果的后续 POST、不跳转登录。自定义 fetcher 在读取完成后取消也不消费结果。
- 运行时缓存测试实际转译生产模块，只替换测试配置：deadline 缩短为 80 ms，超时用例 retries=0，清空 configured RPC base，Tauri 检测 stub=false；HTTP 失败用例保留 retries=2 验证状态不重试。覆盖超时回退、迟到 JSON 不覆盖缓存、失败 body 释放、成功完整读取后的缓存复用。不是对生产 2500 ms 三次尝试耗时的实测。
- raw Response 流测试在 header timeout 之后继续读取第二个 SSE 样式数据块；实际 EventSource 未经修改或真实浏览器长连接验收。

## 未验证项与整合风险

- 未执行 `pnpm.cmd -C apps run test:navigation` 或浏览器端到端验收；本次没有修改页面、路由、弹窗或导航缓存。HTTP 证据来自 Node fetch，本次不声称已验证完整浏览器 + `codexmanager-web` 部署链路。
- 原 `scripts/tests/web_runtime_probe*.ps1`、`web_ui_smoke.ps1` 已在基线移除（见 `docs/zh-CN/TESTING.md`）；没有将不存在的脚本计为通过。本次使用新增 loopback HTTP/runtime 模块回归。
- 未执行 `cargo test --workspace -j 2` 或 `cargo test --manifest-path apps/src-tauri/Cargo.toml -j 2`：未修改 Rust、桌面命令或桌面生命周期；Web 壳单独验证。根 workspace 的验证不代表 Tauri 验证。
- **Q01/Q06 必须组合复验**：本任务没有复制 RPC 操作分类策略。基线默认重试可能重放非幂等 RPC；本次覆盖“不因迟到结果触发后续写请求”和“不因调用方取消继续尝试”，并不宣称解决 Q01 的发送结果未知重放问题。整合建议先 Q01 后 Q06，核对 `request.ts`、`rpc-http.ts`、fetcher 类型/四参测试适配及各方法重试参数，重跑完整 transport/runtime、真实 HTTP 与非幂等 RPC 回归；桌面部分另按 Tauri manifest 验证。
- 局部测试通过不等于组合版本验收通过。本工作区没有合并任何其他任务成果。
- 本次未接触真实数据库、账号或生产服务。HTTP mock 服务、连接和新增测试的转译临时目录已清理；保留本工作区依赖、构建输出供审阅，未清理其他会话资源。
