# og session baseline

Baseline source: `2fee10f2b3c2a774546ae420fa0e5a5d90beaf25`, the fetched
`origin/main` selected by `lane.sh take lane-8 og-session`. The brief named
the earlier `7a38bcab0`. The only runtime changes in this measurement are
additional trace spans. No acceleration is implemented or selected yet.

## Measurement contract

`measure.py` measures wall time around each actual command, saves both
output streams and the exit code, and hashes its output files. GPU lock
waiting, fixture preparation, and output hashing are outside the timer.
Initial GPU invocations took all slots through `scripts/lock.sh`, releasing
them between invocations. The bounded Meridian samples take one slot, as
requested by the coordinator so other streams can validate. Other agents compile on this shared
12-core macrunner. Load averages accompany the raw records. A slow cold
sample and a fast warm sample do not establish a cache speed-up by themselves.

The baseline uses the development profile that this checkout's agent
commands run, Bun 1.4.4, and the Metal backend. It is not a measurement of
a released archive. Rust is 1.99.1 (`58cf8f9c3`), with LLVM 23.1.1. These
samples precede the move to `nightly-2026-11-05` in `41b1c2635`; the session
experiments must not attribute that toolchain change to the session.
Both the CLI and addon were built from this lane with
`cargo build -p og-host-node -p og-cli`. No wasm build is included in the
timed region: `og build` exports the prebuilt web engine, as a game author
would. Record its prebuilt engine identity separately from the native head.

Cold means a fresh process with this disposable project's `.og`, `cooked`,
`og-out`, and `dist` removed. Warm means another fresh process immediately
after that command, retaining the project outputs. Neither means flushed
OS page caches or deleted Metal driver caches. `check`, `api`, `compare`,
and `frame` have no persistent project cache to warm. Their two columns
are successive process invocations, not an invented cache-hit distinction.

`og build` also has an existing package-level cache at
`packages/engine/dist/.build-cache.json`, keyed by content hash. It stores
Brotli size measurements and the web engine's probe metadata. This cache
is retained, not cleared with project outputs. The first observed build
can fill it; later project-cold builds can hit it. Report that first-fill
sample separately from repeated project-cold samples. It is not evidence
that a future session removed compression work.

Sources and assets are copied without edits, except the bounded fixture's
explicit two-screen `ui` block in `project.ogt`. Meridian's relative
`../../packages` imports and the installed engine package are preserved
with symlinks. Two fixture-setup preflights were excluded: one lacked the
relative packages tree, and the other lacked the installed package for an
unmapped `ui-check` import. Those type errors were harness errors, not fast
successful engine runs.

The short `run` workload is identical on both projects:
`0.2:dump_world:world.json;0.2:screenshot:frame.png;0.3:quit`, headless at
1280 × 720, with type checking enabled. `test` and the template's UI commands
use the project's full default workload. Meridian's bounded UI commands use
title and settings, captured after 0.8 seconds, with every default breakpoint,
locale, and text-scale variant: 48 check cells and 32 preview cells. The
only added commands are `0:resume` and `0.1:open:settings`. These deliberately
do not replay the long races that reach late screenshots in the full matrix.
`compare` decodes and compares the first reference image
to itself and writes its comparison sheet; it is the exact-match workload,
not a claim about mismatch performance. `frame measure` computes contrast,
highpass, mean, and ramp on that image. `api find camera` is the API lookup.

`--trace` runs are separate from the end-to-end samples. New spans cover
type checking, native bundling, process startup and imports, addon load,
game import, engine creation, renderer attachment, game start, and capture
(extraction, GPU readback, PNG encoding). Existing spans cover asset reads,
simulation ticks, render encoding, render passes, and GPU work. Nested spans
overlap: summing tick, load, capture, and host totals double-counts work.

## What the commands actually repeat

- `cook` is already content-keyed, despite calling the key a "stamp".
  `og-cook/src/cook/mod.rs` hashes relative paths, source bytes, import
  settings, and sorted external glTF inputs with FNV-1a. Output stamps also
  include the cooker version and output-specific settings and schema inputs.
  It reads sources again during scanning and keeps the manifest per project.
  The experiments must distinguish avoiding these repeated reads and sharing
  outputs between worktrees from merely introducing content hashing again.
- `run` cooks, checks, bundles, starts Bun, loads the addon, creates an
  engine and renderer, and drives frames from tick zero.
- `test` checks and bundles the game **once per suite invocation**. By default
  it calls the incremental cook twice: before checking, then inside `prepare`.
  `playtest-run.ts` starts `bun run.ts` directly for each run and `then`
  step. It does not repeat `og run`'s cook and type check per playtest.
  Each playtest also has its own driver process and test-module bundle.
- `ui check` and `ui preview` already share one Bun process across cells.
  They import a fresh game module and create/destroy an engine and renderer
  per cell.
- `compare` and `frame` only process PNG files. They do not cook, check,
  bundle, load the addon, or replay a game. `api` reads the API catalogue.
- The current template discovers two screenshot-defined screens, producing
  48 default check cells, not the brief's older 216-cell matrix.

The cold cook reports 25 source assets, 26 outputs, and 3,633,355 source
bytes for the template. Meridian has 45 sources, 64 outputs, and 13,444,918
source bytes. Its default test runs 11 playtests with 183 checks; its UI
matrix discovers 14 screens. These fixture copies retain the measured
workload even when the separate test-restructure stream reduces main's suites.

## A world snapshot is not a JavaScript heap snapshot

`bun tests/evidence/og-session/heap-probe.ts` reproduces this without a GPU:

| Observation | Hash |
| --- | --- |
| Initial world | `b58aa0fde92abb50` |
| Restored world | `b58aa0fde92abb50` |
| First tick, module counter = 1 | `a72f442acfd0dd62` |
| Repeated tick after restore, module counter = 2 | `eaa424b44cc185a1` |

The restore itself is exact. The next tick diverges because restoring the
world does not rewind the callback's module-scope counter. The probe is an
explicit baseline limitation experiment, not a regression test that the
new session should preserve.
`docs/design.md` §3.2 already forbids global mutable game state. The probe
shows that this rule is not enforced for an ordinary module-scope counter.

The two measured projects mostly already keep gameplay state in the world:

- The template stores `Run.started/chimed`, `Spin`, guard alarms, transforms,
  and physics state in resources/components. Its source has no top-level
  `let` or `var` gameplay variable.
- Meridian stores `Race`, `Prefs`, `Tuning`, `Racer`, `TapeChunk`, `Sfx`,
  and `Mark` in the world. Replay input bytes are `TapeChunk` columns, not
  a JS array retained between ticks. `track.ts` builds static `SAMPLES`,
  `MAP`, and related tables at import time; `tape.ts`'s `INDEX` Map is
  subsequently only read. No top-level `let` or `var` was found in its source.

That inspection is not proof for arbitrary JavaScript. Existing examples
demonstrate compatibility hazards:

- `examples/ui-motion/src/main.ts:31`: the mutable `const state` object
  stores screen, card order, and whether timelines were defined. Lines
  170–181 retain and clear a `pending` animation request outside the world.
- `examples/autorig/src/main.ts:58–70`: `shown` retains entity IDs and `rigs`
  caches rig handles. Reusing this module across worlds can address another
  world's entities or assets.
- `examples/art-pixel-topdown/src/main.ts:20`, `art-pixel-side`, and
  `art-paint-2d` use module counters for spawned names. Creating another
  world from the same module does not reset those counters.
- `examples/art-pixel-3d/src/scene.ts:28–42` keeps a mutable `sprites` map.

Engine-host state also needs an explicit boundary: runtime callback and
declaration registries, record caches, the local platform's store and fake
response queues, the audio sink, and renderer history are not interchangeable
with a world blob. `world.snapshot`, the probe's in-process snapshot, and
`render_snapshot` have different contracts. The latter includes inputs and
the probe pointer, drops already-run commands on restore, and resets capture
history. `og render --jobs` already uses that path with eight warm-up frames;
its existence does not prove exact arbitrary-tick session forks.

## Constraints carried into the experiments

The research artifact, revisions 1 and 2, replaces the brief's initial
candidate list:
https://ampcode.com/artifacts/og-session-agent-2026-11-06-5479e52a-a56b-4f1f-bd75-34a9c48e21b6

Treat a run as a recording of engine/build identity, bundle and cook hashes,
seed, commands, external inputs, and snapshot history. Snapshot reuse must
pass a SyncTest-style restore-and-re-step check. A failed or unsupported
fork falls back to an equivalent fresh replay, never a false pass. A future
recording supports coarse snapshots plus exponentially spaced fine snapshots,
edit-and-continue at a recorded tick, and timeline comparisons. Per-system
cache reuse needs a complete dependency key, not merely unchanged source text.

Mutable module-state detection must catch object/Map/closure mutations, not
just top-level `let`. A new restriction cannot silently change existing games'
results. The no-session path remains the correctness reference. The result
model must carry the separate invariants stream's violations and coverage
buckets without changing existing playtest scripts or probe protocol 5.
That stream provides `assertions({ take: true })` to return and clear a
run's violations and coverage while retaining its static assertion catalog.
Independent playtests and sibling branches need this boundary: coverage from
one branch must not satisfy another branch's requirements.

## Results

These are single samples per command and temperature, not medians of repeated experiments.
The coordinator stopped additional baseline measurements to prioritize the
session rebuild. Missing measurements below are not inferred from another
project. The retired command's samples are excluded.

| Command | Template cold / warm ms | Meridian cold / warm ms |
| --- | ---: | ---: |
| `cook` | 801.3 / 38.3 | 3867.2 / 65.1 |
| `check` | 27.9 / 28.5 | 31.1 / 30.0 |
| `run` with capture and dump | 1321.7 / 631.4 | 4645.8 / 822.5 |
| `test`, full measured suites | 10234.1 / 9487.8 | 491863.6 / 643680.0 |
| `ui check` | 117581.9 / 67409.9 | 124213.2 / 201015.8, bounded |
| `ui preview` | 96465.5 / 41433.5 | 100441.6 / 122488.5, bounded |
| `build` | 62900.5 / 229.9 | Not measured |
| `compare`, identical PNGs | 1665.4 / 1650.0 | Not measured |
| `frame measure` | 1219.0 / 1215.2 | Not measured |
| `api find camera` | 53.7 / 40.8 | Not measured |

The bounded Meridian UI check costs 2.59 s/cell cold and 4.19 s/cell warm,
including preparation. Multiplying by 336 gives **14.5 and 23.5 minutes**.
Those are extrapolations for equally short screens, not measurements or
predictions for the full default matrix's long replay prefixes. The shared
runner's one-minute load rose from 26.5 to 41.0 across the pair. The warm
sample being slower is not evidence of a warm-cache regression.

The separate template trace attributes the following milliseconds:

| Phase | Cold | Warm |
| --- | ---: | ---: |
| Cook | 744.529 | 28.375 |
| Type check | 20.988 | 21.267 |
| Bundle | 13.725 | 13.779 |
| Process start and imports | 15.771 | 13.348 |
| Addon load | 28.109 | 5.809 |
| Game import | 35.701 | 10.481 |
| Engine creation | 75.170 | 7.806 |
| Renderer attachment | 176.685 | 124.477 |
| Game start | 82.719 | 59.907 |
| Run loop, inclusive | 264.323 | 229.173 |
| Simulation ticks | 34.843 | 18.387 |
| Render encoding | 162.200 | 143.218 |
| Asset reads | 0.890 | 0.819 |
| Capture, inclusive | 50.074 | 50.772 |

The rows overlap. Read `results.json` for the recorded identities, outcomes,
output hashes, load observations, and trace totals. Further cook scan/read/
parse profiling and broad toolchain candidates were cancelled before measurement.
No claim about their individual costs or a replacement compiler follows.

The bounded result and the long suite identify the first work: reuse the
engine and renderer, restore verified prefixes, and avoid drawing simulation
ticks that do not need a frame. Type checking and bundling remain unchanged
until a measurement shows a material end-to-end benefit.

The template's default UI check finishes its 48 cells but exits 3 in both
samples. Its error is `feedback`: the storybook style has no hover sound.
It also reports four warnings. These are completed failing checks, not
successful UI-check samples or setup failures. They remain in the timing
table, with the exit code visible.

The bounded Meridian check likewise completes all 48 cells and exits 3:
four contrast, three focus-reach, and three text-scale errors, plus 25
warnings. These are findings in the measured baseline fixture, not session
regressions. Both previews exit successfully. No further baseline runs are
pending.

The first Meridian warm suite overlaps a 3.84-second trace-test verification
launched by this thread without acquiring the GPU lock. That was a measurement
error. Keep the sample as observed, flag it as contaminated, and repeat it
before using it to compare implementations. No process was stopped.
