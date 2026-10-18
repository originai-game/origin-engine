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

- **One target directory.** Everyone, agents included, builds into `target/`
  and lets cargo's lock serialise; nobody sets `CARGO_TARGET_DIR=target/agent-*`
  or copies the tree to build "in isolation". Isolation of changes is a Git
  worktree with its own `target/`, used only when two streams must build
  incompatible trees at once, and deleted with the worktree.
- **One profile per lane.** Everything in the gate that runs native code
  (tests, `og` for drift, cook, the addon the bun tests load) is the `dev`
  profile, whose heavy dependencies are already optimised per package in
  `Cargo.toml`. `release` (thin LTO, one codegen unit) is for what ships: the
  wasm the size gate measures, `packages/engine` packs, the perf lane. A step
  that compiles the same crate in a second profile is a defect.
- **Tests never run `cargo`.** A Rust test uses `env!("CARGO_BIN_EXE_og")`; a
  bun or browser test uses the `og` the gate's `build.sh` put in
  `packages/engine/native/` (`tests/support/og-bin.ts`); a cook made once per
  project is reused by every test that needs it. A test that shells out to
  `cargo run` is waiting behind the build lock and may rebuild the world.
- **Two lanes.** `scripts/check.sh quick` (fmt, clippy, drift, the logic
  tests of every crate but og-tests, the knowledge and create packages) is
  the loop while working and the PR signal, under a minute warm.
  `scripts/check.sh` (full) adds the gfx, perf-shaped and browser-shaped
  suites, the wasm build, the size gate and determinism, and is what a
  commit to main must pass. Changes to rendering, animation, physics, the
  tick or the export run the full lane before they are called done.
- **Parallel test binaries.** The test steps use `cargo nextest` when it is
  installed; test binaries that write a shared directory (a cook in place)
  are serialised through `.config/nextest.toml` test groups, never by
  sleeping or retrying.
- **No cold rebuilds for size.** `build.sh` under the gate (`OG_GATE=1`)
  takes the wasm the wasm-build and wasm-opt steps already produced; only a
  standalone `bun run build` compiles it itself.

## Validation

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
