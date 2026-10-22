# `og` 集成 tsrs：类型检查进入作者循环

> 执行包。范围是引擎仓库这一侧：`og` 工具、`packages/engine`、
> `packages/create`、`packages/knowledge`、`scripts/install-tools.sh` 与
> 门禁。规范来源：`docs/design.md` §2.1（`og` 工具）、`AGENTS.md`
> §Non-negotiable contracts（"Observable from the first frame"、"errors as
> values"）、`docs/ci.md` §Toolchain（版本唯一陈述处）。持久结论落到
> `docs/design.md` §2.1（`og check` 的契约）、`docs/ci.md` §Toolchain（钉版）
> 与 `packages/knowledge/skills/og-game/SKILL.md`；证据写进本包的
> `evidence/`，不写进 `docs/`。放在引擎仓库而非平台仓库的 `goals/`，因为
> 本包触碰的每个文件都在这里；平台侧只有 §4.8 的一条 ask。Wave 3 完成后
> 删除本包，过程记录在 Git 历史里。
>
> tsrs 是 TypeScript 7（Go 实现，tsgo）的逐函数 Rust 移植
> （oxc-project/tsrs，MIT）。本包钉 `0.18.0-ts7.1.0-dev.20260929`
> （移植 `microsoft/TypeScript@b85298b6a81f`，7.1.0-dev；二进制报
> `7.1.0-dev.20260930.4`）。一致性：13,458/13,462 错误基线，12,779/12,779
> types/symbols 基线，13,392/13,392 JS emit；`--watch`、`-b`、`--api`
> 可用；`--init`、`--showConfig`、`--generateTrace`、`--locale` 不可用；
> 没有 Windows 与 Intel macOS 包。

## 一、现状与问题

- **`og` 打包不检查类型。** `og run`、`og dev`、`og test`、`og build`
  都经 `crates/og-cli/src/project.rs::bundle_native` →
  `packages/engine/src/node/build.ts` → `Bun.build`，Bun 剥掉类型即出
  bundle。一个类型错误要到运行时、或根本不出现。
- **检查是作者自己的事，且只对人说得通。** `packages/knowledge/skills/og-game/SKILL.md`
  §2 让 agent "每次改动后自己跑 `bun x tsc --noEmit`"；输出是人读的文本，
  不是 `og` 其它输出那样的 JSON 事件（`og dev --json` 的 `reload`、
  `og test --json`、`dump_*`）。一个 agent 的循环里，类型错误不是值。
- **检查器从 npm 来，运行时解析。** 每个模板与示例在 `devDependencies`
  钉 `typescript: "7.0.2"`，每条检查命令是 `bun x --package typescript@7.0.2
  tsc`（`packages/engine/package.json` `typecheck` 三次、
  `crates/og-idl/tests/idl.rs` 一次、`examples/*/package.json` 各一次），
  `.agents/setup` L46–49 预热 bun 缓存只为这一条。沙箱里没有缓存就是一次
  registry 往返；`og doctor` 对它一无所知；`docs/ci.md` §Toolchain 的
  TypeScript 行说"钉在三处"，而其它每个工具钉在一处。
- **速度今天不是瓶颈，但架构是。** tsgo 7.0.2 检查 `packages/engine` 一个
  配置 0.25–0.30 s（`tests/evidence/toolchain-2026-10.md`）。问题不在那
  0.3 s，在它不在循环里、不是值、不受 `og` 钉版。tsrs 的意义：一个
  Rust 二进制，和 `og` 同一条分发与钉版路径，无 node、无 npm、无 JS 运行
  时；它的速度与内存（等检查线程下报 ≤ 8× 快、内存 −70 %）是 `og dev`
  300 ms 预算里放得下"每次保存都检查"的前提，不是目的。
- **引擎本质不变。** 游戏代码是 TypeScript，Rust 留在引擎里；tsrs 是引擎
  侧的工具，作者永远不碰它，和 `wasm-opt`、`brotli` 一样由
  `scripts/install-tools.sh` 钉版安装、`og doctor` 报告。

## 二、决策（本包提议，2026-10-21；⚑ 待 owner）

| # | 决策 | 后果 |
|---|---|---|
| D1 | **tsrs 是钉版二进制，不是 `og-cli` 的依赖 crate。** tsrs 全部 crate `publish = false`，Node API 自称不稳定，近几个版本在修 use-after-free 与数据竞争。 | `og` 以子进程调用它；升级 tsrs 改一行 `TSRS_VERSION`，不动 `Cargo.lock`、不重编译引擎、不进 wasm 体积门。 |
| D2 | **只从 npm 包取二进制，用 SRI 校验。** GitHub Release 无资产；平台包 `@tsrs/<platform>` 含 `tsrs` 与 `lib.*.d.ts`，两者必须同目录（npm 原生构建 `noembed`，无内嵌回退）。 | `scripts/install-tools.sh` 下载 registry 的 tarball，按钉死的 `dist.integrity`（sha512）校验，整目录放到 `$BIN/../lib/tsrs-<ver>/`，软链 `$BIN/tsrs`。 |
| D3 | **`og check` 是新命令，`run`/`test`/`build` 在打包前检查，`dev` 每次重建并行检查。** 诊断是值：`{file,line,col,code,message}` 的 JSON，与帧、dump 同一条观察通道。 | `og run`/`og test`/`og build` 有类型错误即停（`--no-check` 逃生）；`og dev` 不因类型错误拒绝热重载（Bun 照样出 bundle），但每次 `reload` 事件带 `check`，agent 看得到。 |
| D4 | **tsgo 7.0.2 仍是门禁权威，直到 §六 的切换判据满足。** tsrs 移植的是 7.1-dev，与 7.0.2 的差异混着上游变化。 | 过渡期门禁加一步 `check-conformance`（functional lane）：两个检查器同样 `--checkers` 跑引擎三配置、模板与五个示例，**tsrs 漏报 tsgo 报出的错误**才 FAIL，其余差异记进证据。 |
| D5 | **`og` 按链找 tsrs：`OG_TSRS` → 引擎包旁的 `node_modules/@tsrs/<platform>/tsrs` → `PATH`。** 创建的项目已经 `bun install` 取 `@origingame/engine`；引擎包以 `optionalDependencies` 精确钉 `@tsrs/{darwin-arm64,linux-x64-gnu,linux-arm64-gnu,linux-x64-musl,linux-arm64-musl}`，一次安装、无运行时 npm。仓库内的示例没有 `node_modules`，走 `install-tools.sh` 装的 `PATH`。 | 不把第三方二进制再打进 `packages/engine/native/`（`pack.sh` 不多一个平台矩阵，`lib.*.d.ts` 不复制两份）。沙箱镜像只要已 `bun install` 引擎包就带着 tsrs。⚑ 引擎包依赖面多五个可选包，owner 确认。 |
| D6 | **检查线程数默认 `min(4, cores)`，`OG_CHECKERS` 覆盖。** tsrs 默认每核一线程到 16；门禁与多 agent 并行时机器已满载（`AGENTS.md` §Build and gate discipline）。 | 基准与一致性对比两边同 `--checkers`；`og dev` 里检查与 Bun 打包并行，合计仍以 300 ms 为预算。 |
| D7 ⚑ | **切换判据**（§六）：连续两个 wave 末的 `check-conformance` 零差异（含上游 7.1-dev 复核后的差异归零），且 Wave 0 的无漏报语料仍零漏报，则 Wave 3 把 `packages/engine` `typecheck`、`idl.rs`、示例与模板切到 `og check`，`typescript` 从 `devDependencies` 移除。 | 判据由 owner 定；本包按此执行并在 §九记录。 |
| D8 ⚑ | **钉版策略**：跟 tsrs 的 release 走（它每版钉一个 TypeScript 提交），只在 wave 末移，移动时 `docs/ci.md` 表、`TSRS_VERSION`、SRI、`optionalDependencies` 四处同一提交。 | 与 wgpu、wasm-bindgen 同一纪律。 |

## 三、范围与非目标

范围：`og check`；`run`/`test`/`build`/`dev` 的检查接入；tsrs 的安装、
钉版、`og doctor`；诊断解析与 JSON 形状；门禁的 `check-conformance` 步；
知识包、模板、`create` 的文字与脚本；Wave 3 的权威切换。

非目标：
- **不替换 Bun 打包。** tsrs 的 emit 一致性 13,392/13,392，但 `og` 的
  bundle 是 `Bun.build` 的树摇、压缩与 wasm 侧的产物布局，与类型检查无关。
- **不替换 IDL 生成器与 drift 门。** `.d.ts` 由 `og-idl` 生成；tsrs 只检查。
- **不用 `tsrs --api`。** 那是 TS7 原生 API 的镜像（172 个方法，
  MessagePack/JSON-RPC），客户端与服务端须同一 TypeScript 提交，为编辑器
  与长会话设计；`og` 每次检查一个子进程足够，`og dev` 用自己的监视循环
  而非 `tsrs --watch`。
- **不用 `@tsrs/wasm`。** 浏览器内检查（平台 Player 页、线上编辑）是平台
  的事；记在 §4.8 供平台取用。
- **不改 `tsconfig` 的选项集。** 模板的选项（`Bundler`、`paths`、
  `verbatimModuleSyntax`、`isolatedModules`、`allowImportingTsExtensions`、
  `exactOptionalPropertyTypes`、`noEmit`、`types: []`、`lib ES2022`、
  `extends`、`include` glob）tsrs 全部实现。
- Windows 与 Intel macOS 作者：tsrs 没有包。`og check` 找不到 tsrs 时报
  一行（`og doctor` 同样），回退 `bun x --package typescript@<pin> tsc`
  并在 JSON 里标 `tool: "tsgo"`；不为它们写第二条路径。

## 四、设计

### 4.1 分发与钉版

`scripts/install-tools.sh` 新增一段，与 `wasm-opt` 段同形：

```sh
TSRS_VERSION="${TSRS_VERSION:-0.18.0-ts7.1.0-dev.20260929}"
# @tsrs/<platform> 的 dist.integrity（npm view … dist.integrity），移版时一起改
case "$os-$arch" in
  Linux-x86_64)   pkg=linux-x64-gnu;   sri='sha512-EdBwBgNICGu1r1h23ixdcKwsMo6E9ll7ZsY3mNERNYqE1Pwdtcat99d5E7zB1J+oy/Xk6YqiizaB6nolIjpDTw==' ;;
  Linux-aarch64)  pkg=linux-arm64-gnu; sri='<Wave 0 记录>' ;;
  Darwin-aarch64) pkg=darwin-arm64;    sri='sha512-sAirMpaGA9uRQm2W+t4zC7PjnijNuFa4PCSIfLKKo0JyAUCRfnMeev0UGBQ9F3g+9Nz187tvikUs5Irh5+eajA==' ;;
  *) pkg="" ;;
esac
# https://registry.npmjs.org/@tsrs/$pkg/-/$pkg-$TSRS_VERSION.tgz → 校验 sha512 →
# 解出 package/tsrs 与 package/lib.*.d.ts 到 $BIN/../lib/tsrs-$TSRS_VERSION/，
# ln -sf 到 $BIN/tsrs；已装且 `tsrs --version` 报 7.1.0-dev.20260930.4 则跳过
```

版本比对用 `tsrs --version` 输出里的 TypeScript 版本串（包版本号不在二进
制里），`install-tools.sh` 头部注释记录两者的对应。`docs/ci.md` §Toolchain
加一行：

| Tool | Version | Pinned by |
|---|---|---|
| tsrs（`og check` 的类型检查器；过渡期 tsgo 7.0.2 为权威，见 goals/tsrs） | 0.18.0-ts7.1.0-dev.20260929（TypeScript 7.1.0-dev.20260930.4） | `TSRS_VERSION` 与 SRI 在 `scripts/install-tools.sh`，`packages/engine/package.json` `optionalDependencies` |

`.agents/setup` 不改：它已经 `step scripts/install-tools.sh`。

`og doctor`：`TOOLS` 加 `Tool { name: "tsrs", role: "type-checks game code
(og check, and before og run/test/build)", pin: Pin::TsrsVersion, repo_only:
false }`。`Pin::TsrsVersion` 在仓库内从 `scripts/install-tools.sh` 的
`TSRS_VERSION` 行读默认值（与 `rust_channel`、`wasm_bindgen_lock` 同形），
仓库外不比版本。找不到：`problem`，修复行指向
`scripts/install-tools.sh` 或 `bun install`（按 §4.2 的链说哪一个）。

### 4.2 定位、调用与退出码

`crates/og-cli/src/check.rs`（新）：

- **定位**：`OG_TSRS`（文件路径）→ `<engine package>/../@tsrs/<platform>/tsrs`
  （`project::engine_package` 已解析引擎包；`<platform>` 由
  `std::env::consts::{OS, ARCH}` 与 glibc/musl 判定，musl 由
  `cfg!(target_env = "musl")`）→ `PATH` 上的 `tsrs`。三处都没有：
  `Err(NO_TSRS)`，一行说安装方式；`og check` 用 tsgo 回退（§三）。
- **调用**：`tsrs -p <tsconfig> --noEmit --incremental false --pretty false
  --checkers <N>`，`current_dir` 为项目根，stdout 收集，stderr 透传。
  `--incremental false` 让每次检查无状态（`.tsbuildinfo` 不落盘，
  确定性）；`og dev` 的增量来自 tsrs 自身的速度，不是磁盘状态。
- **退出码**（TypeScript 的 `ExitStatus`）：0 干净；1、2 有诊断（`--noEmit`
  下通常 1，直接传坏文件时实测 2，两者都是"完成了，带诊断"）；3 项目无效
  （`tsconfig` 解析失败，诊断在 stdout，无文件位置）；4 项目引用循环；
  5 未实现；信号或其它 → 工具故障。0–4 走解析；5 与故障报 `og check:
  tsrs failed (<status>)` 并原样打印 stdout/stderr，退出 1，**不**当作
  "无错误"。
- **tsconfig 解析**：项目根的 `tsconfig.json`；没有的（裸入口 `og run
  src/main.ts`，或没写 `tsconfig` 的 `project.ogt` 项目）在
  `.og/tsconfig.check.json` 合成一份：`compilerOptions` 取自模板
  `templates/game/tsconfig.json`（`include_str!`，单一来源），`include`
  为入口文件，`paths` 把 `@origingame/engine` 与
  `@origingame/engine/playtest` 指到解析出的引擎包的 `types/index.d.ts`
  与 `src/playtest.ts`。合成文件带注释头说明它是生成的。

### 4.3 诊断解析与 JSON 形状

原生 CLI 没有 JSON 诊断开关（`--diagnostics` 是计时统计；JSON 只在
`@tsrs/wasm`）。解析 `--pretty false` 的文本，与 tsrs 自己的基准解析器
同一语法：

```
<path>(<line>,<col>): error TS<code>: <message>
error TS<code>: <message>                       ← 无位置（配置、全局）
    <continuation>                              ← 缩进行并入上一条 message
```

```rust
pub struct Diagnostic {
    pub file: Option<PathBuf>, // 相对项目根
    pub line: Option<u32>, pub col: Option<u32>, // 1 起
    pub code: u32,
    pub message: String,       // 续行以 '\n' 连接
}
pub struct Report {
    pub tool: &'static str,    // "tsrs" | "tsgo"
    pub version: String,
    pub config: PathBuf,
    pub ms: f64,
    pub diagnostics: Vec<Diagnostic>,
}
```

`--json` 一行：`{"event":"check","ok":false,"tool":"tsrs","version":
"7.1.0-dev.20260930.4","config":"tsconfig.json","ms":41.3,"diagnostics":
[{"file":"src/main.ts","line":12,"col":7,"code":2322,"message":"…"}]}`。
文本模式原样打印 tsrs 的行（编辑器可点），末尾一行
`og check: N error(s) in M file(s) (tsrs, 41 ms)` 或 `og check: ok (tsrs,
41 ms)`。

解析器的单元测试在 `check.rs` 里：有位置、无位置、多行续行、Windows 路径
含 `(`、消息里含 `: error TS`、空输出、退出码 3 的配置错误。每个用例
都是 tsrs 实际输出的字面量。

### 4.4 `og check [project|entry] [--json] [--checkers N] [--tool tsrs|tsgo]`

- `Project::open` 同其它命令；`--tool` 默认按 §4.2 的链，`tsgo` 显式走
  `bun x --package typescript@<pin> tsc`，`<pin>` 是 `check.rs` 的一个常量
  `TSGO_VERSION`，`idl.rs` 与 `scripts/tsrs-diff.sh` 读同一处（Wave 3 前
  模板的 `devDependencies.typescript` 是它的拷贝，`check-conformance` 核
  对相等）。两条路径同一解析器、同一 JSON。
- 退出码：0 无诊断，1 有诊断，2 参数错，其它工具故障同 §4.2。
- `main.rs` `USAGE` 加一行，列在 `test` 前。
- `docs/design.md` §2.1 的 `og` 命令表加 `check`，一句契约（打包前检查，
  诊断是值，`--no-check`）。

### 4.5 `run`、`test`、`build`、`dev`

- **`run::prepare`**（`run`、`test` 的共用入口）：cook 之后、
  `bundle_native` 之前 `check::run(&p, &package, checkers)`；有诊断 → 文本模
  式打印诊断，`Err("<N> type error(s); fix them or pass --no-check")`。
  `--json` 下诊断以 `check` 事件打印在 `run` 的 JSON 流里（与 `summary`
  同一流）。`--no-check` 跳过。`og test --json` 的结果里加
  `"check": {ok, ms, errors}`，检查失败计一次 `failed`，playtest 不跑
  （bundle 都没出）。
- **`bundle::run`**（`og build`）：同样在 `web_build` 前；`--publish` 不接受
  `--no-check`（发布构建必须过检查）。
- **`dev`**：`src/` 改动后，`bundle_native` 与 `check::run` **并行**
  （`std::thread::scope`，各自子进程），bundle 一出就写、热重载照常；检查
  结果到了打印一行 `check: ok (38 ms)` 或诊断，`--json` 下 `reload` 事件
  多 `"check": {"ok":…, "ms":…, "errors":N}`，诊断本身单独一行 `check`
  事件（`reload` 行保持短）。检查未完成时 `reload` 先出，`check` 事件随后；
  `--exit-after N` 等最后一次检查落地再退。`assets/` 改动不检查。
  300 ms 预算的口径不变（rebuild + swap）；`check` 的 ms 单独报，Wave 0
  的数字决定它是否也应计入（§六）。

### 4.6 门禁：`check-conformance`

`scripts/tsrs-diff.sh`（新）：对 `packages/engine` 的三个 `tsconfig*.json`、
`templates/game`、`examples/{brasswick,hollowbell,kestrel-stair,lantern-vale,
meridian-loop}`，两个工具各跑一次（同 `--checkers 4 --noEmit --incremental
false --pretty false`，tsrs 加 `--checkerAssignment go --noLazyMembers` 取
最接近 tsgo 的顺序），解析成 `(file,line,col,code)` 集合比较：

- tsgo 报而 tsrs 未报（漏报）→ FAIL，列出每条；
- tsrs 报而 tsgo 未报，或消息文本不同 → 写
  `target/evidence/gate/check-conformance.log`，PASS；
- 两边都干净 → 一行 PASS。

`scripts/check.sh` 的 functional lane 加步 `check-conformance`，无
`step_needs`（它直接调两个检查器，读源码，不读任何构建产物）；path-aware：`packages/engine/types/**`、`templates/**`、
`examples/**/*.ts`、`scripts/tsrs-diff.sh` 未变则跳过。quick lane 不跑。
差异复核：把差异所在项目再用 `bun x --package
typescript@7.1.0-dev.20260930.4 tsc` 跑一次（只在手动复核时，不在门禁），
7.1-dev 也报的不是 tsrs 缺陷，记进 `evidence/` 并在 §九 注明。

### 4.7 知识包、模板、`create`

- `packages/knowledge/skills/og-game/SKILL.md` §2 第 137–146 行换成：
  `og check` 检查类型；`og run`/`og test`/`og build` 打包前自动检查，类型
  错误在 `--json` 的 `check` 事件里；`og dev` 每次保存并行检查并在
  `reload` 事件里报。示例的 `paths` 一段保留（它说明的是仓库内示例的解
  析方式，仍然成立）。`packages/knowledge/scripts/validate.ts` 跑过。
- `templates/game/package.json`、`examples/*/package.json`：`"check": "og
  check ."`；`"typecheck"` 与 `devDependencies.typescript` **Wave 3 前保留**
  （tsgo 仍是权威，编辑器与 `bun run typecheck` 继续可用）。
- `packages/create`：`tests/` 加一例，创建的项目 `og check .` 退出 0；
  注入一个类型错误后退出 1 且 JSON 里有 `code: 2322`。

### 4.8 平台 ask（`docs/platform-asks.md` 加一行）

| # | Ask | Why | 引擎侧的替代 |
|---|---|---|---|
| 10 | `engine` 类沙箱镜像在预热 `bun install` 时保留 `@origingame/engine` 的 `optionalDependencies`（`@tsrs/<platform>`），不传 `--omit=optional`；镜像体积预算 ≤ 600 MB（`docs/phase-1.md` M7）内再计一个 tsrs 二进制（Wave 0 量） | 沙箱里的 agent 跑 `og check` 不该碰 registry | 镜像没装时 `og check` 回退 tsgo（`bun x`，需要网络或缓存），`og doctor` 报 tsrs 缺失 |

另记一条非 ask 的信息给平台：`@tsrs/wasm` 提供 JSON 诊断（`diagnostics:
"json"`），是浏览器内检查的可用件；引擎不消费。

## 五、实施波次

一个 agent 的循环是 `cargo check -p og-cli` 与它改的 crate 的单元测试；
门禁由合并的那个会话跑（`AGENTS.md` §Build and gate discipline）。每行一
个提交，直接推 main。

### Wave 0 — 测量（无 owner 决定；决定 D3 里 `dev` 的并行策略与 §六 的阈值）

| # | 工作 | 文件 | 通过标准 |
|---|---|---|---|
| 0.1 | **在本 orb 装钉版 tsrs**：按 §4.1 手动取 `@tsrs/linux-x64-gnu` tarball，校 SRI，解到 `~/.local/lib/tsrs-<ver>/`。记录二进制与 `lib.*.d.ts` 的体积。 | `goals/tsrs/evidence/wave-0.md` | `tsrs --version` 报 `7.1.0-dev.20260930.4`；sha512 与钉值相等。 |
| 0.2 | **等线程基准**：tsgo 7.0.2 与 tsrs，`--checkers {1,2,4,8}`，`--noEmit --incremental false --pretty false`，各 5 次取中位数，墙钟与峰值 RSS（`/usr/bin/time -v`），对象：`packages/engine` 三配置、`templates/game`、五个示例。另记两者的**启动开销**（空项目一文件）。 | `goals/tsrs/evidence/bench.sh`（随包删除）、`evidence/wave-0.md` 的表 | 表齐全；每格有中位数与极差；机器状态（`nproc`、负载）记在表头。 |
| 0.3 | **一致性语料**：干净项目两边零诊断；再造 `goals/tsrs/evidence/corpus/`：20 个游戏代码里典型的错误各一文件（组件字段类型错、缺 import、`exactOptionalPropertyTypes` 违例、`verbatimModuleSyntax` 的 type-only import、`paths` 解析失败、`defineSystem` 的读写集类型、`Vector3` 误传数组、playtest API 误用……），两边跑，比 `(file,line,col,code)`。 | `evidence/corpus/*.ts` + 一个 `tsconfig.json`、`evidence/wave-0.md` | 零漏报；每条不一致都用 `typescript@7.1.0-dev.20260930.4` 复核并归类（上游差异 / tsrs 缺陷）。 |
| 0.4 | **go/no-go**：对照 §六 的阈值写结论；不达标则本包在 §九 记"停在 Wave 0"并说明哪个数字。 | `evidence/wave-0.md` | 结论一段，数字引用表格。 |

### Wave 1 — `og check`（D1、D2、D5、D6）

| # | 工作 | 文件 | 通过标准 |
|---|---|---|---|
| 1.1 | **安装与钉版**：`install-tools.sh` 的 tsrs 段（§4.1），头部注释加一行；`docs/ci.md` §Toolchain 加行；aarch64 与 darwin 的 SRI 记齐。 | `scripts/install-tools.sh`、`docs/ci.md` | 干净机器上脚本装出的 `tsrs --version` 正确；`shellcheck` 过；再跑一次报 `present` 并跳过。 |
| 1.2 | **解析器**：`Diagnostic`、`Report`、`parse(&str) -> Vec<Diagnostic>`、退出码映射（§4.2、§4.3）与单元测试。 | `crates/og-cli/src/check.rs` | `cargo test -p og-cli check::` 过；测试用例是 tsrs 实际输出的字面量，含续行与无位置两类。 |
| 1.3 | **定位链与调用**：`OG_TSRS` → `node_modules/@tsrs/<platform>` → `PATH`；`--checkers` 默认；tsgo 回退。 | `crates/og-cli/src/check.rs` | 三条路径各一个集成测试（`env!("CARGO_BIN_EXE_og")`，临时目录里放假 `tsrs` 脚本回放固定输出），不跑 `cargo`。 |
| 1.4 | **`og check` 命令**：参数、`USAGE`、合成 `tsconfig`（§4.2）、文本与 `--json` 输出。 | `crates/og-cli/src/check.rs`、`main.rs`、`docs/design.md` §2.1 | `og check templates/game` 退出 0；`og check examples/brasswick --json` 一行合法 JSON；裸入口 `og check examples/brasswick/src/main.ts` 合成配置并退出 0。 |
| 1.5 | **引擎包的 `optionalDependencies`**（D5），`packages/engine/README.md` 一句。 | `packages/engine/package.json` | `bun install` 在 linux-x64 落下 `node_modules/@tsrs/linux-x64-gnu/tsrs`；`og doctor` 在创建的项目里报 `tsrs` 来自 `node_modules`。 |
| 1.6 | **`og doctor`**：`Tool` 行与 `Pin::TsrsVersion`。 | `crates/og-cli/src/doctor.rs` | 仓库内报 `tsrs <ver> (TSRS_VERSION)`，缺失时修复行正确；仓库外跳过版本比对。 |
| 1.7 | **`check-conformance`**（§4.6）。 | `scripts/tsrs-diff.sh`、`scripts/check.sh` | functional lane 多一步，当前树零漏报 PASS；path-aware 跳过的理由被打印；`docs/ci.md` §Lanes 加一行。 |

### Wave 2 — 进入循环（D3）

| # | 工作 | 文件 | 通过标准 |
|---|---|---|---|
| 2.1 | **`run`/`test`/`build` 打包前检查**，`--no-check`，`--publish` 拒绝 `--no-check`；`og test --json` 的 `check` 字段。 | `crates/og-cli/src/run.rs`、`test.rs`、`bundle.rs` | 对 `examples/brasswick` 注入一个类型错误：三条命令退出 1 且不出 bundle；`--no-check` 下行为如旧；`og run --commands` 的既有测试（`crates/og-cli/tests/`）不动。 |
| 2.2 | **`dev` 并行检查**与 `reload`/`check` 事件（§4.5）。 | `crates/og-cli/src/dev.rs` | `og dev examples/brasswick --json --exit-after 2`：两个 `reload` 事件各带 `check`，注入错误后第二个 `check` 事件列出它且热重载仍 ok；`docs/design.md` §7 的 300 ms 段加一句 `check` 的口径（按 Wave 0 结论）。 |
| 2.3 | **知识包**：SKILL.md §2 文字（§4.7），`validate.ts`。 | `packages/knowledge/skills/og-game/SKILL.md` | `bun run validate` 过；不再出现 `bun x tsc`。 |
| 2.4 | **模板与示例的 `check` 脚本**；`create` 的测试（§4.7）。 | `templates/game/package.json`、`examples/*/package.json`、`packages/create/tests/` | `bun run test` 在 `packages/create` 过；`scripts/check.sh --only bun-test` 过。 |
| 2.5 | **平台 ask**（§4.8）。 | `docs/platform-asks.md` | 一行，形状与 #9 同。 |

### Wave 3 — 权威切换（D4、D7；判据满足后）

| # | 工作 | 文件 | 通过标准 |
|---|---|---|---|
| 3.1 | **`packages/engine` `typecheck` → `og check`**（三配置，`--tool tsrs`）；`test` 脚本不变。 | `packages/engine/package.json` | `bun run test` 过；时间对比写 `evidence/wave-3.md`。 |
| 3.2 | **`idl.rs` 的测试经 tsrs**：`env!("CARGO_BIN_EXE_og") check packages/engine`。 | `crates/og-idl/tests/idl.rs` | 测试过，不再 `bun x`。 |
| 3.3 | **移除 `typescript` devDependency**：模板、示例、`.agents/setup` L46–49；`docs/ci.md` TypeScript 行改为"tsrs 为检查器，tsgo 仅 `check-conformance`"。 | 各 `package.json`、`.agents/setup`、`docs/ci.md` | `rg 'typescript@7' --type json` 只剩 `scripts/tsrs-diff.sh` 的对照一处。 |
| 3.4 | **`check-conformance` 降为 wave 末测量**：从 functional lane 移到 `scripts/wave-end.sh`。 | `scripts/check.sh`、`scripts/wave-end.sh`、`docs/ci.md` | functional lane 少一步；wave 末报告有它的摘要。 |
| 3.5 | 删除本包，§九 的最终状态并入 `docs/phase-1.md` 当轮的一段。 | `goals/tsrs/`、`docs/phase-1.md` | — |

## 六、验证与验收

Wave 0 的阈值（go/no-go，§五 0.4）：
- 一致性语料**零漏报**（硬条件：循环里的检查器漏报比没有检查器更坏）；
- `--checkers 4` 下 tsrs 的墙钟 ≤ tsgo 的，在每个对象上；
- 单文件项目的启动开销 ≤ 60 ms（`og dev` 里与 Bun 打包并行后不动预算）；
- 二进制 + `lib.*.d.ts` ≤ 40 MB（镜像预算）。

每个 wave：`scripts/check.sh quick` 在改动的 crate 上过是 agent 的交付；
合并者跑 `scripts/check.sh functional`。改 `dev.rs` 的 2.2 不改渲染、tick
或导出，不触发 full lane；但 `og dev` 的 300 ms 证据（`tests/evidence/P2/
tooling/m4-m5-tooling.md` 的口径）在 2.2 后重录一次进 `evidence/wave-2.md`
（8 次 reload，最坏值），确认并行检查没有挪动 rebuild + swap。

Wave 3 的切换判据（D7，⚑）：连续两个 wave 末 `check-conformance` 零差异
（上游复核后），语料零漏报，且 tsrs 自 Wave 0 以来没有发布含 checker
正确性修复而未被本仓库采用的版本。

验收：
- 创建的项目里 `og check .`、`og run .`、`og test .`、`og build . --publish`
  在类型错误上各自一致地停；`og dev --json` 每次保存出 `check`。
- `og doctor` 报 tsrs 与其钉版，缺失时一行修复。
- `AGENTS.md` 与 `docs/ci.md` 关于"版本只陈述一处"的规则在
  tsrs 上成立：`TSRS_VERSION` 一处，`optionalDependencies` 是它的拷贝并
  在 `check-conformance` 里核对相等。

## 七、风险与对策

| 风险 | 对策 |
|---|---|
| tsrs 移植 7.1-dev，与 7.0.2 权威的差异混着上游变化 | 每条差异先用 `typescript@7.1.0-dev.20260930.4` 复核（§4.6）；只有 7.1-dev 不报而 tsrs 报、或反之，才记 tsrs 缺陷；D4 的 FAIL 只认漏报。 |
| 项目自述 Node API 不稳定、近期修过 UAF 与数据竞争 | D1：子进程、钉版、只用 CLI；崩溃（信号）按工具故障报而不是"无错误"（§4.2）。 |
| 无 Windows / Intel mac 包 | 回退 tsgo（§三），`og doctor` 一行说明；不写第二套。 |
| `lib.*.d.ts` 与二进制分离会让 tsrs 静默用错 lib | 安装整目录、软链二进制（§4.1）；`og check` 启动时校验 `lib.es2022.d.ts` 与二进制同目录，否则报错不跑。 |
| 多 agent 并行时检查线程抢机器 | D6 默认 `min(4, cores)`；`OG_CHECKERS=1` 给满载的机器；`og dev` 一次只一个检查在飞（新保存取消上一个子进程）。 |
| `og dev` 并行检查把 300 ms 预算挤掉 | Wave 0 先量启动与单项目耗时；2.2 后重录 8 次 reload 证据；超预算则 `check` 改为重载后串行，预算口径不变。 |
| `optionalDependencies` 在某些安装器下被省略（`--omit=optional`） | 定位链的第三环是 `PATH`，`og doctor` 说清来自哪一环；平台 ask #10 把镜像说明白。 |
| 上游停止维护 | 钉版不动即可继续工作；D4 的 tsgo 权威在切换前一直在，切换后 `check-conformance` 作为 wave 末测量保留一条回退证据。 |

## 八、执行方式与证据

- 一个 agent 线程按 wave 做，行即提交，推 main；不跑门禁、不录参考帧、
  不做浏览器运行（`AGENTS.md` §Build and gate discipline）。合并者跑
  functional lane。
- 证据在 `goals/tsrs/evidence/`：`wave-0.md`（基准表、语料结果、go/no-go）、
  `wave-2.md`（`og dev` 的 8 次 reload、`check` 事件样例）、`wave-3.md`
  （切换前后 `bun run test` 时间、`check-conformance` 两个 wave 的零差
  异记录）。每个文件头写提交 SHA、日期、机器。
- 持久结论的去处：`og check` 的契约 → `docs/design.md` §2.1；钉版 →
  `docs/ci.md` §Toolchain；作者说明 → SKILL.md；平台 → `docs/platform-asks.md`。
  本包在 Wave 3 的 3.5 删除。

## 九、进度

| 日期 | 事 |
|---|---|
| 2026-10-21 | 本包写成于 `66f65c03`。tsrs 的事实（版本、包、退出码、诊断格式、选项支持、一致性数字）核自 oxc-project/tsrs 的 `docs/COMPATIBILITY.md`、`docs/STATUS.md`、`docs/USAGE.md`、`npm/README.md`、`bench/run.py`，当日。等 owner：D5、D7、D8。Wave 0 未开始。 |
| 2026-10-22 | **Wave 0** `f23984a8`：go，四个阈值全过（`evidence/wave-0.md`：`--checkers 4` 下 tsrs 为 tsgo 墙钟的 20–47 %，启动 23 ms，32.5 MB；22 个语料文件 23 条诊断零漏报、零多报、消息逐字相同）。 |
| 2026-10-22 | **Wave 1** `c3196eaa`…`722fcc87`：`install-tools.sh` 的 tsrs 段（五个平台 SRI 记齐）、`og check`（`check.rs`，解析器 8 个单元测试，定位链/失败/合成配置 4 个集成测试）、`og doctor` 的 tsrs 行、引擎包 `optionalDependencies`（D5 按提议落地；`bun install` 实测落下 `@tsrs/linux-x64-gnu`，bun 1.4.2 无视 `libc` 也装上 musl 包，记进 ask #13）、functional lane 的 `check-conformance`。实测与计划不同处：tsrs 对配置错误退出 2 而非 3；`--version` 自带包版本，`og doctor` 直接比 `TSRS_VERSION`；找到但不可用的 tsrs（无 lib 文件、`OG_TSRS` 指空）报错而不回退 tsgo，只有“找不到”才回退。 |
| 2026-10-22 | **Wave 2** `bd3b2e96`…`2d7c282a`：`run`/`test`/`build` 打包前检查、`--no-check`、`--publish` 拒绝它；`og dev` 并行检查（一次一个在飞，新保存杀掉旧检查进程）；SKILL.md；ask #13。发现：除 `templates/game` 外十个模板（fx-showcase、ui-kit、八个 archetype）从未作为项目被检查过，`tests/*.ts` 的 `node:` 模块与全局在 `types: []` 下各报 10–16 个错误（tsgo 同报），接入后会挡住 `og run`/`og test`；各加 `host.d.ts` 修复，`check-conformance` 扩到全部模板。`og dev` 证据见 `evidence/wave-2.md`（检查 33–48 ms，不挪动 rebuild；本 orb 只有 lavapipe，swap 670–1220 ms 与检查无关，Metal 上的 300 ms 复测归合并者）。 |
| 2026-10-22 | **Wave 3 未开始，按 D7 的判据不能开始**：判据要连续两个 wave 末 `check-conformance` 零差异；今天是第一次（19 个配置 + 语料全部零差异，对 tsgo 7.0.2 与 7.1.0-dev.20260930.4 都是），npm 上 tsrs 最新仍是 0.18.0。等 owner：D7、D8（D5 已按提议实施，可否决）。 |

