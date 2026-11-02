# Origin Engine repository guidance

Normative design: `docs/design.md`, and for the game UI `docs/ui.md`. Platform contracts this engine implements:
`origingame/docs/game-workflow.md` (the production contract),
`origin-agent/packages/contracts/src/probe.ts` (the probe, protocol 5),
`origingame/docs/design-language.md` (tokens and components for the player
shell and the marks only; the engine's and origin-agent's own design
language is `packages/design` here). The execution package during the build-out is
`origingame/goals/origin-engine/plan.md`; the release path and the plan of
what remains to launch, `docs/release.md`.

## Ownership

- This repository owns the engine runtime, its TypeScript API, the cook, the
  `og` tool, the project templates and the generated wiki. It never owns
  accounts, money, publishing, threads or the agent loop: those stay in the
  platform, reached through the host ABI with game-scoped tokens.
- The platform consumes this repository only through published packages
  (`@origingame/engine`, `@origingame/create-game`, the knowledge package) pinned
  by version, installed from the npm registry: never a layout built from a
  git revision, never a checkout or `cargo` on the platform's side
  (`docs/release.md` §1, the owner's decision of 2026-10-22). A version is
  tested locally (the install test, the platform's local stack) before a
  sandbox installs it. The sandbox of the development stage is an InstaBox
  machine (dashboard.instabox.sh; the owner, 2026-10-22), where the install
  test runs too (`tests/install/instabox.ts`); E2B is production's provider
  and the last consumer of a version. No engine script targets E2B for
  development. No platform code imports engine internals; no engine code
  imports platform code.
- The platform is two repositories since the agent-repo split
  (2026-10-27): the private `originai-game/origin-agent` holds the agent's
  server, runner, web, protocol, Stage, knowledge and iOS, including the
  probe contract, the engine pin (`server/scripts/e2b/origin-engine.json`)
  and the knowledge tree `knowledge/` that the engine's knowledge mounts
  into; everything else stays in `originai-game/origingame`. origin-agent's
  `scripts/knowledge/mount-engine.ts` imports `installed`, `mount` and
  `summary` from `packages/knowledge/scripts/mount.ts`: their signatures
  are a public contract, and changing them is a break (a minor version
  with a `changelog.json` breaking entry). The platform also reads the
  knowledge straight from the installed package, with no mounted copy,
  through `packages/knowledge/scripts/lib.ts`: `payload(roots: Roots):
  PayloadFile[]` (every mounted file at its production path, sorted by
  path) and `defaultRoots(pkg?: string): Roots`, with the shapes
  `PayloadFile { path: string; source: string; bytes: Uint8Array;
  symlink: boolean }` and `Roots { pkg: string; template: string; engine:
  string }`, are the same public contract beside `installed`, `mount`
  and `summary`: kept stable, and a change to any of them is a break.
  The art brief is the same kind of contract (`docs/art.md` §2.2):
  `packages/knowledge/scripts/brief.ts`'s `artBrief`, `readLook` and
  `presetLook`, re-exported by `@origingame/create-game`, which the
  platform calls for every asset request.
- Engine-level game knowledge lives here, not in the platform (the
  owner's decision of 2026-10-28): a skill that uses only `og` and the
  engine API is this knowledge package's (`packages/knowledge/skills/`,
  mounted at `game/skills/<name>/`), including the craft skills that moved
  from origin-agent (architecture-discipline, component-forge,
  level-texturing, localization, recipe-craft, visual-acceptance). They
  name no platform tool or service; the platform's own skills and
  instructions stay in origin-agent. A skill's name is a contract: games
  pin `skill:<name>` packs by content hash, so a skill is never renamed,
  and a changed body is a new pack version.

## Non-negotiable contracts

- **Rust stays inside the engine.** Game code is TypeScript. A game author
  (human or agent) never runs `cargo`; the sandbox ships the prebuilt wasm and
  native addon. A feature that would require game-side Rust is an engine
  feature and lands here.
- **One API, two hosts.** The TypeScript API is generated from one IDL for
  the wasm host and the native host. A member that exists on one and not the
  other is a bug, not a platform difference; backend limits are reported
  through capability queries, never by missing members.
  Edit `crates/og-api/api/<module>.ogt` for that module (`ui.ogt` for UI);
  `prelude.ogt` holds the header and units, and `parts.txt` fixes the order.
- **Deterministic by construction.** Fixed tick, seeded randomness owned by
  the engine, no wall-clock reads inside the tick, snapshot and restore of
  the whole world. Rendering never feeds back into simulation.
- **Observable from the first frame.** The command channel, the probe,
  `dump_world`/`dump_entity`/`dump_ui` and frame capture are part of the
  runtime in every build configuration except `publish`, where only the
  player-facing probe members remain.
- **Reflection is the single schema.** Components are declared once with
  their metadata; the inspector, Tune, the probe, scene text and the wiki all
  read that schema. Nothing is described twice.
- **Two text formats, one blob format.** Scenes and assets are `.ogt` text
  (git-diffable, line-based), cooked into `.ogb` blobs with a type hash, a
  version and a cook stamp. No hand-rolled binary formats.
- **The API is designed for a reader who has never seen it.** Desc structs
  whose zero value is a sensible default, no hidden state, errors as values,
  one documented entry per module, names that follow one convention, and the
  three.js vocabulary (`Vector3`, transforms, material parameters, glTF
  loading) wherever it does not contradict engine semantics.
- **Rendering never blocks the engine thread on the web.** The engine runs in
  a Worker with an OffscreenCanvas; the main thread owns input and the shell.
- **Brand is one ink.** The boot splash, loading state and "made with" mark
  use `packages/brand` of the platform (`genesis`, `progress` motions) and the
  design tokens; the engine never introduces a second palette or mark.
- **One design language for the engine and origin-agent, apart from the
  platform.** The engine and origin-agent share their own design language,
  decoupled from the platform's (`origingame/docs/design-language.md`
  styles the portal, ops and mail; the owner, 2026-11-02). This repository
  owns it: `packages/design` (published as `@origingame/design`) holds the
  language document and its `tokens.json`, seeded from the platform's
  language of 2026-11-02 (the neutral ladder, flat glass on floating chrome
  only, borderless controls) and evolving here from then on. og-ui's
  default style, which replaces the grey `prototype`, is generated from it
  into `crates/og-ui/src/theme.rs` and checked for drift; origin-agent pins
  the published package by version, as it pins the engine. Roles only
  games need (health, mana, xp, gold, the HUD) live in the same tokens. The
  default style is for origin-agent and prototypes only: every game ships
  its own style (`type style`, a look's `ui` line), and a publish build
  refuses one still in the default (`docs/ui.md` §12.1). The language binds
  the default style alone, never a game's.

## Build and gate discipline

The gate was measured at seven minutes warm with four agents compiling the
same crates into private target directories (143 GB of `target/`), so these
rules exist to keep it fast (`docs/ci.md` §Lanes has the numbers):

- **One target directory per checkout.** Everyone, agents included, builds
  into the checkout's `target/` and lets cargo's lock serialise; nobody sets
  `CARGO_TARGET_DIR=target/agent-*` or copies the tree to build "in
  isolation".
- **Parallel rounds reuse persistent worktrees: the lane pool.** A stream
  that must build beside another takes a lane, `scripts/lane.sh take
  lane-N <stream>`: a persistent worktree `/Volumes/app/origin-engine-lane-N`
  (made once by `scripts/worktree.sh`, its `target/` kept warm across
  streams) switched to a new branch `<stream>` from main. After the merge,
  `scripts/lane.sh free lane-N` puts it back on main for the next stream.
  Only a lane's first build is cold, and sccache (`.cargo/config.toml`)
  serves that one the main checkout's dependencies. Never `git worktree
  add` a fresh one per round, never remove a lane to free disk, and never
  delete a worktree's `target/` (nor the main one's): a stale crate is
  cleaned with `cargo clean -p <crate>`, never wholesale. `scripts/lane.sh
  --list` shows every lane and the branch it carries. One more lane,
  `origin-engine-measure`, carries no stream: the wave-end measurements
  (`--all`, web-reference) run there pinned at a
  commit, so main never waits for them.
- **One gate at a time, on the whole machine.** `scripts/check.sh` holds
  `~/.cache/og-gate/og-gate.lock` and then `target/og-gate.lock` (a release
  `build.sh` holds the target's); a second one, in any worktree, waits
  (`scripts/lock.sh`, `docs/ci.md`). Never bypass it to run gates beside each
  other: they are slower together than in a queue. The gate runs two steps
  at once by default (`OG_GATE_JOBS`) and reniced by 10.
- **Parallel agents write code; one place runs the gate.** When several
  agents work at once (worktrees), none of them runs `scripts/check.sh`, a
  test lane, `og test`, web-reference or a reference capture: the machine
  measured a load average past 100 with three worktree gates and a lavapipe
  container beside each other, and every one of them then fails on
  timeouts. An agent's loop is `cargo check -p <crate>` and the unit tests
  of the crate it edits, run one at a time; it commits on its branch and
  reports. Nothing broader: no sweep of other projects' reference frames
  through `og run --commands`, no "every frame unmoved" check across
  templates and games, no browser runs to confirm a native change, no
  wasm or release build to measure (the owner's instruction, 2026-10-22:
  development is what agents are for; the coordinator's functional lane
  after the merge is the check of every reference frame, and the wave's
  end measures). A frame an agent re-records is one its change must
  move; a test an agent adds is the one that fails on the old code. The coordinator (the one session that merges into main) is the
  only runner of the gate, the review and the reference captures, once per
  merge, serially. The same applies to a human beside running agents.
- **Sub-threads: at most eight at a time, Claude Code on Opus 5.5 medium.**
  When the coordinator delegates streams to separate Amp threads
  (`create_thread`), it keeps at most eight running and starts another
  as soon as one has reported and been merged (the owner's instructions:
  2026-10-23, eight at once was too many while gates ran beside each
  other; 2026-11-01, five instead of three; 2026-11-02, eight, now that
  each thread draws on its own Mac satellite and only macrunner runs the
  gate). Every
  such thread runs in `external-agent:claude-code` mode with features
  `["model:claude-opus-5-5", "effort:medium"]` (the owner, 2026-11-01;
  high before), the project ref
  `origingame/origin-engine`, and its base commit named in the prompt;
  never Fable or another model, never a mode picked for the task. One
  exception (the owner, 2026-11-01): splitting oversized source files
  runs in Amp `high` mode threads, briefed to move code verbatim (only
  `mod`, `use` and visibility edits; no new checks, error handling,
  helpers, renames or comments) and reviewed by the coordinator with
  `git diff --color-moved` before merging. A
  stopped or paused thread is not resumed: its pushed commits are merged
  or dropped by the coordinator, its findings go into the brief of a new
  thread, and the new thread starts from the merged `origin/main`.
- **One profile per lane.** Everything in the gate that runs native code
  (tests, `og` for drift, cook, the addon the bun tests load) is the `dev`
  profile, whose heavy dependencies are already optimised per package in
  `Cargo.toml`. `release` (thin LTO, one codegen unit) is for what ships natively:
  `packages/engine` packs, the perf lane; `wasm` (release with the
  load-time crates at opt-level z) is every wasm build, the one the size
  gate measures. A step
  that compiles the same crate in a second profile is a defect.
- **Tests never run `cargo`.** A Rust test uses `env!("CARGO_BIN_EXE_og")`; a
  bun or browser test uses the `og` the gate's `build.sh` put in
  `packages/engine/native/` (`tests/support/og-bin.ts`); a cook made once per
  project is reused by every test that needs it. A test that shells out to
  `cargo run` is waiting behind the build lock and may rebuild the world.
- **Two lanes.** `scripts/check.sh quick` (fmt, clippy, drift, the logic
  tests of every crate but og-tests, the knowledge and create packages) is
  the loop while working, under half a minute warm.
  `scripts/check.sh` (full) adds the gfx, perf-shaped and browser-shaped
  suites, the wasm build, the size gate and determinism, and is what a
  commit to main must pass. Changes to rendering, animation, physics, the
  tick or the export run the full lane before they are called done.
- **Real GPUs only; no lavapipe.** The compatibility baseline is current
  hardware and browsers (`docs/design.md` §8, the owner, 2026-10-27).
  Every GPU run happens on the two runners' GPUs: `metal` on macrunner,
  DX12 natively and with Chrome and Edge on WebGPU on studio-win; the web
  is WebGPU only, with no WebGL2 tier (the owner, 2026-11-01). Nothing runs
  on lavapipe any more (the owner, 2026-11-01: it was slow, ui-styles took
  25 min on an orb against 2 on macrunner, and judged nothing): an orb
  runs nothing that draws on its own Linux machine. It draws on its Mac
  satellite instead (`mac_sidecar`, mac.large, macos-27; the owner,
  2026-11-02): a native dev build there (`RUSTC_WRAPPER= cargo build -p
  og-host-node -p og-cli`, then `OG_GATE=1 sh scripts/build.sh --native`,
  bun 1.4.4) draws `og test` playtests, `og run` films and `og ui check`
  pixel-identical to macrunner's `metal` references (measured 2026-11-02:
  RMSE 0 on every kit phone frame). There an agent draws the frames its
  change moves, one playtest at a time (`--jobs 2` dropped frames), reads
  them against `docs/ui.md` §15.4 and fixes them before reporting; it
  never runs `scripts/check.sh`, a wasm build or a recording there. The
  coordinator still runs the gate and records references on macrunner
  (and studio-win for DX12 and the browsers) when it lands a branch. No
  `references/vulkan-cpu/` frame is recorded. After each
  batch of merges the coordinator runs both lanes on the runners, and a
  publish needs both green on its commit.
- **When to run the gate.** Function first, measurement later. After a
  batch of merges on main, the one session that merges runs, on
  macrunner, the functional lane (`scripts/check.sh functional`: fmt, clippy, every
  test with the GPU reference frames, drift, every style pack's reference
  sheets (`ui-styles`), the packages' tests on the
  native host) and that is what a merge must pass. Landing is kept short
  (the owner, 2026-11-02): a merge builds native only and never rebuilds
  the wasm (with a stale web module the cross-host tests skip, and the
  wave's end runs them); branches whose authors have drawn and read
  their frames clean on a Mac satellite are landed in batches, rebased
  one after another and passed through one functional run; and the
  session that lands draws no review frames, reading only the frames
  that fail tolerance before recording them. The wasm build and its
  size, determinism and the browser tiers are measurements of a wave's
  end: `scripts/wave-end.sh` (`scripts/check.sh --all`, then
  `scripts/check.sh web-reference`, serially, on the runners) once per wave, with its summary in the report, never per
  merge. A size or
  frame-time number is a target to pursue there, not a reason to hold a
  functional merge. Never run `--all` after every edit. When a reference frame changes, re-record
  it and rerun only its lane: `scripts/check.sh --only test` (og-tests' gfx
  references), `--only bun-test` (a template's playtest frames),
  `scripts/check.sh web-reference` (the browser tiers); `--only` adds the
  steps the named one reads (`docs/ci.md` §When to run what).
- **Concurrent steps.** The gate runs its steps concurrently as their
  inputs allow (test and wasm-opt after wasm-build, size-gate and bun-test
  after wasm-opt), each with its log in `target/evidence/gate/<step>.log`,
  printed whole when the step ends; the first FAIL stops the rest. A new
  step that reads another's output declares it in `step_needs`
  (`scripts/check.sh`), never by its position in the lane.
  `OG_GATE_JOBS=1` runs one step at a time.
- **Path-aware steps.** Both lanes skip what none of its inputs changed
  since it last passed (`docs/ci.md` §Lanes has the rules): the GPU
  reference tests, the wasm build, wasm-opt and size gate (their last
  outputs are reused), drift, web-reference. Each skip is printed with its
  reason; `--all` runs everything, and a wave's end always does. Do not work around a
  skip by touching files, and do not delete `target/og-gate/`: pass `--all`.
- **Parallel test binaries.** The test steps use `cargo nextest` when it is
  installed, one process per test. A fixture several tests read is made
  once across their processes under a file lock (og-cook's
  `tests/support`), and what cannot run beside others is declared in
  `.config/nextest.toml` (test groups, slots, priorities), never handled by
  sleeping or retrying.
- **No cold rebuilds for size.** `build.sh` under the gate (`OG_GATE=1`),
  and standalone wherever the gate has built, takes the dev addon and `og`
  and the wasm the wasm-build and wasm-opt steps already produced; only
  `OG_RELEASE=1` (what `pack.sh` and every shipping build set) compiles
  release and the wasm itself.

## Validation

- Toolchain versions (Rust 1.99.1 and edition 2024, wgpu 30, binaryen 133,
  wasm-bindgen 0.2.129, TypeScript 7.0.2, bun 1.4.4) and where each is
  pinned: `docs/ci.md` §Toolchain, the one place they are stated. A version
  move changes that table and its pin in the same commit.

- Rust: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
  (or `scripts/check.sh quick`, which runs the same with nextest).
- Engine tests: `og test` in `tests/` (logic, multicore, gfx against committed
  reference frames, perf against baselines). A gfx test compares pixels; a
  perf test compares against a baseline with a tolerance. A test that ran no
  checks fails.
- API: the generated `.d.ts` and the wiki are build outputs checked for drift
  by the gate's drift step; hand edits to generated files fail the build.
- Every change that alters rendering, animation, physics or the tick carries a
  scripted `og run --commands` run whose frame was read and described in the
  change.
- Every stream adds the frames it read and the numbers it measured to the
  showcase archive (`bun scripts/showcase.ts add`, `docs/ci.md` §Showcase);
  nothing under a `rejected/` directory, and withdrawn work is marked with
  `showcase.ts exclude`.
