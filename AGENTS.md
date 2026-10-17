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

## Validation

- Rust: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
- Engine tests: `og test` in `tests/` (logic, multicore, gfx against committed
  reference frames, perf against baselines). A gfx test compares pixels; a
  perf test compares against a baseline with a tolerance. A test that ran no
  checks fails.
- API: the generated `.d.ts` and the wiki are build outputs checked for drift
  in CI; hand edits to generated files fail the build.
- Every change that alters rendering, animation, physics or the tick carries a
  scripted `og run --commands` run whose frame was read and described in the
  change.
