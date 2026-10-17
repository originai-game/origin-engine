# M7 platform notes

What phase 1 M7 changed in the OriginGame platform repository
(`origingame/origingame`), why, and what is still owed by the platform's
own goals. Everything listed here is additive and ships unwired: no mode,
executor class, scaffolder or tool catalog selects the new substrate yet.
The platform plan is `origingame/goals/origin-engine/plan.md` §三.

## Engine repository (this one)

| Path | Change |
|---|---|
| `packages/knowledge/scripts/mount.ts` | New. `bun packages/knowledge/scripts/mount.ts <origingame checkout> [--check] [--commit <rev>]` materialises `knowledge.json`'s mounts into `packages/game-knowledge/production`: skills → `game/skills/<name>/`, the wiki → `game/skills/og-game/reference/`, `templates/game` + `origin-engine.json` + `distribution.json` → `game-scaffold/origin-engine-ts/`. It validates first (an invalid package mounts nothing), owns exactly `game/skills/{og-build,og-core,og-game}/` and `game-scaffold/origin-engine-ts/`, writes only differing bytes, removes stale files there, never writes template binaries, and `--check` exits 1 naming each added, changed and removed path. `--commit` mounts a `git archive` of a commit instead of the working tree. |
| `packages/knowledge/scripts/sync-rules.ts` | The skills and overlay this package mounts are no longer counted as platform skills it could collide with (otherwise the first mount would make the package invalid). `rules/platform.json` is unchanged. |
| `packages/knowledge/scripts/lib.ts` | `TEMPLATE_SKIP` adds `og-out` and `.og` (the template's own `.gitignore`): a template that was run once no longer mounts its run output and bundled entry. |
| `packages/knowledge/skills/og-build`, `og-game` `SKILL.md` | Descriptions scoped to projects with a `project.ogt` (not a Godot `project.godot`): every Game's knowledge inventory lists all skills, and the platform's Godot substrate is also called Origin Engine. |
| `packages/knowledge/tests/mount.test.ts` | New: the CLI against a stand-in platform tree (byte equality, no binaries, other skills/overlays untouched, idempotence, `--check` on an edited, a missing and a stray file, an invalid package, a non-checkout path). |

The mount in the platform tree is from **c20e014** (`--commit c20e014`):
the wiki was being regenerated in the working tree at the same time. The
two description edits above are not in that mount; after they are
committed, re-run `mount.ts <checkout> --commit <new rev>` (it will change
the two `SKILL.md` files and whatever the regenerated wiki changed).

## Platform repository: files changed

| Path | Change | Why |
|---|---|---|
| `packages/game-knowledge/production/game/skills/og-build/`, `og-core/`, `og-game/` (with `og-game/reference/*.md`) | New, written by `mount.ts` | The engine knowledge as platform packs (`skill:og-*`) |
| `packages/game-knowledge/production/game-scaffold/origin-engine-ts/` | New overlay, written by `mount.ts` (13 text files; 6 binaries named in its `distribution.json`) | The project template as `scaffold:origin-engine-ts` |
| `packages/game-knowledge/scripts/production.test.mjs` | Skill count 17 → 20 and the three names in the expected list | The test pins the skill inventory |
| `packages/game-knowledge/scripts/production-bundle.test.mjs` | File count 122 → 155 and skills 17 → 20 | The test pins the payload: +31 mounted text files, +2 files `1e03549a6` added without updating the count (the overlay's brand `icon.svg` and `splash.png`) |
| `packages/game-knowledge/production/README.md` | One paragraph: the overlay and skills are mounted from the engine package, never hand-edited; nothing selects the overlay yet | Ownership of the new paths |
| `agent-runner/crates/runner-tools/src/og.rs` | New adapter, feature `origin-engine-ts` | See below |
| `agent-runner/crates/runner-tools/src/lib.rs` | `#[cfg(feature = "origin-engine-ts")] pub mod og;` | |
| `agent-runner/crates/runner-tools/src/toolchain.rs` | Feature-gated `og: Option<PathBuf>` (`ORIGIN_OG_BIN`, else PATH) and capability `engine-ts` | Detection once at start, like every other binary |
| `agent-runner/crates/runner-tools/Cargo.toml` | `[features] origin-engine-ts = []` (off by default) | The gate |
| `agent-runner/crates/runner-tools/build.rs` | `ORIGIN_RUNNER_OG_TESTS=1` → `cfg(origin_og_tests)` | The real-toolchain test gate every other integration test uses |
| `agent-runner/crates/runner-tools/tests/og.rs` | New integration test (needs the env gate and the feature; a missing feature is a compile error, a missing `og`, engine checkout or ffmpeg a failure) | Runs `templates/archetypes/runner` through the adapter |
| `agent-runner/README.md` | Rows for `og`, `og::start`, `og::dump`, the `engine-ts` tag and the test command | |

### The runner adapter

`og::start` (`engine_start`'s shape plus `width`/`height`) starts
`og run <project> --headless --port <p> --size <w>x<h> --out
.origin/frames/og-run` as the thread's supervised service **`engine`**, the
same service the Godot warm process is, and requires og-probe's `hello`
(`substrate: "origin-engine"`, protocol 5, wire 1). og-probe speaks the same
probe wire, so every existing warm-process tool runs on it without a line
changed and writes where it always writes (`.origin/frames/<toolUseId>/`):
`engine_probe` (`hello`, `advance`, `input`, `state`, `pause`, …),
`engine_capture`, `engine_sweep`, `engine_stop`, `stage_capture`,
`stage_record` (tick-stepped, ≤ 30 fps), `level_report`, `playtest_replay`.
The adapter adds only `og::dump` (`dump` world/entity/ui saved as JSON next to
the frames). `feel_compare` and `flow_map` still start temporary Godot
processes from a `project.godot`; they need an `og` branch when the Godot
path retires.

## Owed by the platform's active goals

These belong to goals that own the files (`executor-hardening`,
`model-and-tool-dispatch`); M7 did not touch them.

1. **The `engine` E2B executor class without Godot or Chromium**
   (`agent-server/scripts/e2b/build-templates.ts`, executor-hardening): bun,
   the `og` binary and `@origingame/engine` (native addon for
   `linux-x64`, built with `cargo zigbuild --release -p og-host-node --target
   x86_64-unknown-linux-gnu`), Mesa lavapipe for wgpu's Vulkan backend,
   ffmpeg; no Godot, no Chrome/agent-browser; ≤ 600 MB. The runner is built
   with `--features origin-engine-ts` for this class and finds `og` on PATH
   (or `ORIGIN_OG_BIN`).
2. **The mode table without substrate** (`agent-server/modes`,
   model-and-tool-dispatch): no `substrate` field or substrate-conditional
   sections; the executor class fixed by the mode row. Until then nothing
   routes a thread to `engine-ts`.

## Further asks (not owned by M7)

- **Runner dispatch** (`runner-tools/src/production.rs`): with the feature
  on, `engine_start` goes to `og::start` when the project has a
  `project.ogt` (Godot otherwise), `engine_*`/`stage_*` specs are reported
  when `tc.og` exists (today they need Godot or Chromium,
  `Toolchain::frames`), and a `engine_dump` spec calls `og::dump`. The plan's
  end state renames the capability `engine` to mean `og`; M7 used
  `engine-ts` so the Godot capability keeps its meaning meanwhile.
- **Origin's `prepare-knowledge.mjs`** (`origin/`) returns a fixed
  substrate list (`canvas2d`, `three`, `origin-engine`); it copies the new
  overlay already but does not name it.
- **The template** (`templates/game`, engine repository): it has no
  `.gitattributes` routing media to Git LFS and no `.origin/setup`, both of
  which the platform's other overlays carry (a scaffolded Game would commit
  its 2.5 MB font and models to Agent Service Git directly, and a sandbox
  snapshot would not pre-cook). Its `package.json` depends on
  `@origingame/engine@0.0.1`, which is not published yet, so a scaffolded
  project needs `OG_ENGINE_PACKAGE` until it is.
- **A pre-existing platform test failure** unrelated to M7: in a checkout
  with Git LFS smudged, `game-scaffold/origin-engine/origin/brand/splash.png`
  is a real PNG and `readProductionBundle` refuses it (three tests in
  `production-bundle.test.mjs` fail before and after M7). With the LFS
  pointer (CI without smudge) they pass.
