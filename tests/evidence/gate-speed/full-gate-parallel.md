# The full gate, serial and concurrent

`scripts/check.sh --all` on main, Apple M4 Pro (12 cores, 24 GB), sccache
warm, nothing changed between runs, 2026-10-19. Another agent's cargo build
of the engine and an iOS simulator were on the machine throughout; the
load average at the start and end of each run is given, and numbers move
by about 20 % with it.

## Before and after, per step (seconds)

| Step | brief (old script) | before, old script, b2ae4bd | one at a time (`OG_GATE_JOBS=1`, new script) | after, run 7 | after, run 8 |
|---|---|---|---|---|---|
| fmt | 1 | 2 | 1 | 1 | 1 |
| clippy | 4 | 1 | 1 | 1 | 1 |
| wasm-build | 44 | 15 | 9 | 4 | 7 |
| test (nextest, 437 tests) | 42 | 43 | 34 | 69 | 67 |
| wasm-opt | 37 | 44 | 45 | 69 | 68 |
| size-gate | 21 | 27 | 20 | 24 | 25 |
| drift | 4 | 1 | 1 | 2 | 2 |
| bun-test | 83 | 52 | 15 | 17 | 18 |
| determinism | 1 | 1 | 1 | 2 | 0 |
| **total** | **237** | **187** | **129** | **97** | **101** |
| load average (start → end) | | | 17 → 15 | 6 → 18 | 18 → 21 |

Runs 7 and 8 are two consecutive `--all` runs at e2ed217 (the final
script). In the concurrent runs a step's seconds overlap: wasm-build, then
test beside wasm-opt (both start at +7 s), then size-gate beside bun-test
(both start when wasm-opt ends). Every concurrent `--all` run of the round:

| Run | total | test | wasm-opt | size-gate | bun-test | load (start → end) |
|---|---|---|---|---|---|---|
| 3 | 99 | 66 | 67 | 24 | 17 | ~20 |
| 4 | 135 | 73 | 79 | 48 | 36 | → 32 |
| 5 | 84 | 54 | 54 | 22 | 14 | 9 → 20 |
| 6 | 121 | 75 | 77 | 37 | 28 | 20 → 24 |
| 7 | 97 | 69 | 69 | 24 | 17 | 6 → 18 |
| 8 | 101 | 67 | 68 | 25 | 18 | 18 → 21 |

The path-aware full lane with nothing changed (`scripts/check.sh`, the
GPU tests and the wasm steps and drift skipped): 39 s with bun-test
waiting for test, 30 s without (bun-test does not wait for test in the
final script). The quick lane: 30 s.

## Stability

Every run above passed. Each one gave 437 of 437 nextest tests passed
(5 skipped, as before; every og-tests gfx reference compared within
tolerance), 71 of 71 engine bun tests, the same determinism numbers
(math_digest 6117185439544271997, bouncing_boxes_hash 15551159556835976588
on host debug and release, wasm32 scalar and simd128) and the same publish
wasm (10,807,952 B after wasm-opt, 2,943,180 B brotli). No tracked file
changed. A failing step (determinism with a `node` that exits 1) stopped
the running test step (`CANCELLED test (1s): stopped when 'determinism'
failed`), the gate exited 1, no cargo or nextest process was left and the
gate lock was gone.

## Where the time went

- **bun-test.** `packages/engine`'s eighteen files ran one after another:
  31 s standalone, 47 s inside the old gate. Run in parallel workers
  (`bun test --parallel`, in the pinned bun 1.4.2) they take 11.6 s
  standalone (three runs: 11.68, 11.62, 11.49 s) and 11–14 s in the gate
  (22–30 s at the higher loads). The five slowest files, standalone, one
  at a time:

  | File | seconds | what it spends them on |
  |---|---|---|
  | tests/polish.test.ts | 9.6 | `og polish` playing the topdown archetype's seven playtests (7.5 s), then four small polish runs |
  | tests/template.test.ts | 6.0 | scaffolding a game, cooking it, its playtest and the frame and dumps |
  | tests/reload.test.ts | 4.0 | a 240-tick realtime run with a watched edit at 1.5 s |
  | tests/basis.test.ts | 3.0 | cooking two textures to Basis once (top-level, shared by the file's tests) |
  | tests/audio.test.ts | 2.7 | `og run --audio` twice, to compare the two WAVs |

  None rebuilds or respawns what it could share: the second audio run and
  the polish runs are what the tests assert on, basis already cooks once,
  reload is real time by design. polish's single `og polish` run is the
  longest file and so the floor of the parallel run.
- **test beside wasm-opt.** Both are CPU-bound (nextest's processes; two
  binaryen `-Oz` runs of a 15 MB module), so together they take 54–69 s
  where alone they take 34 s and 45 s: the overlap saves about 12 s, not
  their sum.
- **size-gate** is one `brotli -q 11` of the 10.8 MB module (single
  threaded, 20–25 s); it overlaps bun-test.
- **wasm-build** relinks og-host-web for both feature variants each run
  (4–15 s), because the two variants write the same uplifted file.

## The shipped wasm is the measured wasm

`OG_RELEASE=1 sh packages/engine/scripts/build.sh` (what `pack.sh` runs;
146 s here, release addon and `og` included), after build.sh and the gate
both called `scripts/wasm-opt.sh`, wrote
`packages/engine/dist/web-publish/origin-engine_bg.wasm` with SHA-256
`ee76bb5c6b378984…`, 10,807,952 B, `brotli -q 11` 2,943,180 B: the same
bytes as the gate's `target/og-wasm/opt/og_host_web-publish.wasm` and the
size-gate number. Before, build.sh's wasm-opt had no `--low-memory-unused`
and shipped 2,969,335 B brotli (measured by the platform's pin agent).
