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
- **wgpu is used as published (the owner, 2026-11-04).** The engine
  depends on the crates.io wgpu and never vendors or patches it; a
  speed-up or fix that seems to need a change inside wgpu is built in the
  engine's own code (og-render, og-host-web) instead.
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
  streams) switched to a new branch `<stream>` from main. Only `take`
  switches a lane (it claims it atomically; never a bare `git switch` or
  `git reset` in a lane you did not take). After the merge,
  `scripts/lane.sh free lane-N` puts it back on main for the next stream.
  Only a lane's first build is cold, and sccache (`.cargo/config.toml`)
  serves that one the main checkout's dependencies. Never `git worktree
  add` a fresh one per round, never remove a lane to free disk, and never
  delete a worktree's `target/` (nor the main one's): a stale crate is
  cleaned with `cargo clean -p <crate>`, never wholesale. Disk is kept by
  `scripts/clean-targets.sh` (the owner, 2026-11-05, after /Volumes/app
  filled), installed on macrunner to run daily at 05:00 (`--install`): in
  every worktree nothing is building in, it removes `target/*/incremental`
  and runs `cargo sweep --time 1`; run it by hand when space runs low,
  never delete a target/ instead. `scripts/lane.sh
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
- **Sub-threads: eight at a time, in waves, Claude Code on Opus 5.5 medium.**
  When the coordinator delegates streams to separate Amp threads
  (`create_thread`), at most eight author streams run at once (macrunner
  and the art director are not counted). The rest wait, paused with their
  work pushed, and the next one starts only when a running stream has
  finished its whole scope (the owner, 2026-11-02 latest: thirteen at once
  stalled threads and hit rate limits). Each wave starts fresh threads,
  each owning several related items end to end, so efficiency comes from
  fewer threads that talk less. Work is cut large and submitted rarely
  (the owner, 2026-11-04, to shorten macrunner's queue): one thread owns
  a whole feature area, works through its items on one branch, checks
  them on its orb, and sends macrunner one head for the area, or at most
  a few heads at natural milestones, never one per item or per fix; a
  review note from art or macrunner is answered on the same branch and
  resent once. An author sends those heads to
  macrunner itself and reports to the coordinator only when an item is
  done or blocked, never per batch. Earlier instructions:
  2026-10-23, eight at once was too many while gates ran beside each
  other; 2026-11-01, five instead of three; 2026-11-02, eight, now that
  each thread draws on its own Mac satellite and only macrunner runs the
  gate; 2026-11-02 later, thirteen, then back to five, then eight (the owner, to speed up, with no author on a GPU). macrunner's queue is kept short:
  only the newest head of each stream waits, and a superseded or stale
  head is dropped, never landed). Every
  such thread runs in `external-agent:claude-code` mode with features
  `["model:claude-opus-5-5", "effort:medium"]` (the owner, 2026-11-01;
  high before), the project ref
  `origingame/origin-engine`, and its base commit named in the prompt;
  never another model, never a mode picked for the task. One escalation
  (the owner, 2026-11-04): a problem that has resisted several rounds of
  fixes (its attempts and evidence recorded) goes to a new Claude Code
  thread with features `["model:claude-fable-5-1", "effort:high"]`, briefed
  with every failed attempt and its evidence; the original thread stops on
  that problem.
  Claude Code is the default, not a rule for everything (the owner,
  2026-11-04): macrunner, hard logic and backend work, and splitting
  oversized source files may run in new Amp `high` threads. An Amp
  `high` thread other than macrunner is archived once its in-flight
  work lands, its open items going into a new thread's brief, in the
  mode the item needs. A file split is briefed to move code verbatim (only
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
- **Features first; compatibility at the end (the owner, 2026-11-02).**
  Until the feature work is done, one platform judges pixels: `metal` on
  macrunner, whose references already exist. DX12 and Chrome or Edge on
  studio-win, Safari, phones (iPhone, Android with Adreno and Mali) and any
  per-backend references are a compatibility phase after the features,
  not per merge or per wave. **Started early (the owner, 2026-11-04):**
  Windows (DX12, Chrome, Edge, Firefox) on studio-win runs now beside
  the feature work (Safari on macrunner is paused so features keep
  macrunner, and Android on the owner's Pixel waits for the phone; both
  are to-dos), in one thread that fixes
  engine bugs at the root, records per-backend references only for
  legitimate differences, commits its evidence under
  `tests/evidence/compat/` and sends heads to the art runner; Metal stays
  the reference platform. The web stays WebGPU only, with no WebGL2
  tier, and nothing runs on lavapipe. **Authors never touch a GPU**: no
  Mac satellite, no Modal, no cloud GPU. An author checks its work on its
  orb with `cargo check -p`, the edited crates' unit tests, typecheck and
  the logic tests, names in its report the frames its change should
  move, and pushes. macrunner draws and records the moved frames when it
  lands the branch (reading only the ones that fail tolerance) and sends
  the moved frames of visual work to the art director, whose notes fold
  into the author's next round. Only macrunner runs `scripts/check.sh`,
  wasm builds and references. **Superseded (the owner, 2026-11-04, to
  speed up):** author threads run on macrunner in their own lanes with
  the Metal GPU, run their own tests, scoped GPU suites and frames, record
  their own references on their branch, and push the branch. One art
  runner on macrunner reviews every stream's frames as the art director
  (looks only, against commercial games) and, when it approves, rebases
  and pushes to main without rerunning what the author ran (one quick
  only when the rebase conflicted). Only breakage blocks on its own.
  Authors without a GPU (orbs) name the sheets and the art runner renders
  them. GPU work takes a GPU slot (`og_gpu_acquire` in `scripts/lock.sh`,
  three at once on macrunner by default, `OG_GPU_SLOTS`); frame-time and
  perf measurements, wave-end and the full gate take every slot
  (`og_gpu_acquire_all`), one acquisition per scene or run, never one
  held across a chain of them, so queued single slots get in between
  (it lets the single waiters queued before it go first). The slot code
  is one machine-wide copy (`~/.cache/og-gate/gpu-lock.sh`, the newest
  `og_gpu_lock_version` any lane has wins); source `scripts/lock.sh`,
  never a private copy, and never override its `sleep` or poll; nobody runs the full gate, `--all`, wasm builds
  or web-reference beside others. Long-running
  threads are retired and replaced by fresh ones before their context
  fills. macrunner carries two landing sessions
  (the owner, 2026-11-04, to speed landing): one for the UI side
  (og-ui, kits, style packs) and one for the render side (rendering,
  terrain, motion frames), the second in a lane worktree. They share
  the machine-wide gate lock, so their gates still queue, and each
  rebases onto origin/main right before it pushes. No contact sheets or films unless the
  coordinator asks for one. No `references/vulkan-cpu/` frame is recorded.
- **Fix at the root; port the state of the art (the owner, 2026-11-03).**
  A problem found anywhere (a scene, a kit, the site, a test) is fixed in
  the engine where it arises, never worked around, faked or waived in the
  game, kit or page; a kit's interim workaround is removed once the engine
  fix lands. A missing feature is implemented carefully and completely:
  its author first searches the current state of the art (recent papers,
  talks, permissive open-source implementations) and ports the best fit,
  its licence kept under `third_party/` or in the file header, rather
  than hand-rolling a simplistic version. The introduction site
  (engine.origingame.ai) deploys only once the features it shows are done.
- **Landing on the author's word (the owner, 2026-11-02).** To speed
  development, a branch whose author reports it drawn, read and tested
  is merged on that report, with no second review by the coordinator.
  Landing a branch is a rebase, `scripts/check.sh quick` and a push;
  the functional lane and the re-recording of reference frames run at
  milestones (a stream's batch set finished, or a few landings
  together), not per branch. A failure found there goes back to the
  branch that caused it as a fix-forward commit. The rules below say
  what each lane runs; this rule says how often.
- **When to run the gate.** Function first, measurement later. After a
  batch of merges on main, the one session that merges runs, on
  macrunner, the functional lane (`scripts/check.sh functional`: fmt, clippy, every
  test with the GPU reference frames, drift, every style pack's reference
  sheets (`ui-styles`), the packages' tests on the
  native host) and that is what a merge must pass. Landing is kept short
  (the owner, 2026-11-02): a merge builds native only and never rebuilds
  the wasm (with a stale web module the cross-host tests skip, and the
  wave's end runs them); branches whose authors report them
  complete and tested are landed in batches, rebased
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

- **There is no GitHub CI (the owner, 2026-11-04).** The gate is
  `scripts/check.sh` on macrunner and nothing else. Never subscribe to
  GitHub checks after a push, never wait on them, and never mention them
  in a report ("no checks" is neither a pass nor news).

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
