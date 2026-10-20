# Origin Engine repository guidance

Normative design: `docs/design.md`. Platform contracts this engine implements:
`origingame/docs/game-workflow.md` (the production contract),
`origingame/origin/packages/contracts/src/probe.ts` (the probe, protocol 5),
`origingame/docs/design-language.md` (tokens and components for the UI toolkit
and the player shell). The execution package during the build-out is
`origingame/goals/origin-engine/plan.md`.

## Ownership

- This repository owns the engine runtime, its TypeScript API, the cook, the
  `og` tool, the project templates and the generated wiki. It never owns
  accounts, money, publishing, threads or the agent loop: those stay in the
  platform, reached through the host ABI with game-scoped tokens.
- The platform consumes this repository only through published packages
  (`@origingame/engine`, `@origingame/create`, the knowledge package) pinned
  by version. No platform code imports engine internals; no engine code
  imports platform code.

## Non-negotiable contracts

- **Rust stays inside the engine.** Game code is TypeScript. A game author
  (human or agent) never runs `cargo`; the sandbox ships the prebuilt wasm and
  native addon. A feature that would require game-side Rust is an engine
  feature and lands here.
- **One API, two hosts.** The TypeScript API is generated from one IDL for
  the wasm host and the native host. A member that exists on one and not the
  other is a bug, not a platform difference; backend limits are reported
  through capability queries, never by missing members.
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

## Build and gate discipline

The gate was measured at seven minutes warm with four agents compiling the
same crates into private target directories (143 GB of `target/`), so these
rules exist to keep it fast (`docs/ci.md` §Lanes has the numbers):

- **One target directory per checkout.** Everyone, agents included, builds
  into the checkout's `target/` and lets cargo's lock serialise; nobody sets
  `CARGO_TARGET_DIR=target/agent-*` or copies the tree to build "in
  isolation".
- **Parallel rounds reuse persistent worktrees.** A round that must build
  beside another takes `scripts/worktree.sh <name>`: it creates
  `/Volumes/app/origin-engine-<name>` on branch `<name>` from main once and
  reuses it (branch, changes and `target/` as they are) every time after,
  so only the first build there is cold, and sccache (`.cargo/config.toml`)
  serves that one the main checkout's dependencies. Never `git worktree add`
  a fresh one per round, and never delete a worktree's `target/` (nor the
  main one's): a stale crate is cleaned with `cargo clean -p <crate>`,
  never wholesale. A worktree is removed only when its stream of work is
  over for good.
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
  reports. The coordinator (the one session that merges into main) is the
  only runner of the gate, the review and the reference captures, once per
  merge, serially. The same applies to a human beside running agents.
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
  the loop while working and the PR signal, under half a minute warm.
  `scripts/check.sh` (full) adds the gfx, perf-shaped and browser-shaped
  suites, the wasm build, the size gate and determinism, and is what a
  commit to main must pass. Changes to rendering, animation, physics, the
  tick or the export run the full lane before they are called done.
- **When to run the gate.** During a round, the path-aware full lane
  (`scripts/check.sh`, no flag) runs once per batch of merges on main, by
  the one session that merges; it reruns only what the batch reaches. Run
  `scripts/check.sh --all` once, at the end of the round, before the final
  commit, and put its summary in the report. Never run `--all` after every
  edit, and never a gate per merge. When a reference frame changes, re-record
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
  reason; `--all` runs everything and CI always does. Do not work around a
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
  wasm-bindgen 0.2.129, TypeScript 7.0.2, bun 1.4.2) and where each is
  pinned: `docs/ci.md` §Toolchain, the one place they are stated. A version
  move changes that table and its pin in the same commit.

- Rust: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
  (or `scripts/check.sh quick`, which runs the same with nextest).
- Engine tests: `og test` in `tests/` (logic, multicore, gfx against committed
  reference frames, perf against baselines). A gfx test compares pixels; a
  perf test compares against a baseline with a tolerance. A test that ran no
  checks fails.
- API: the generated `.d.ts` and the wiki are build outputs checked for drift
  in CI; hand edits to generated files fail the build.
- Every change that alters rendering, animation, physics or the tick carries a
  scripted `og run --commands` run whose frame was read and described in the
  change.
