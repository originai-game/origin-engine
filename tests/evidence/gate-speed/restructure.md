# Test restructuring, 2026-11-06

Base: `5387f79db`, including the genre UI relocation. Package-owned kit and
style suites remain independent; their frame cuts belong to kit-frames-cut.

## Nightly and binary consolidation

**Current decision:** LLVM on every target and profile. The owner withdrew
the alternative backend after the first milestone. Its configuration,
component and overrides are removed. The dated nightly, four frontend
threads, Linux mold and Windows rust-lld remain. Metadata embedding and
layout now use defaults pending controlled experiments. Windows remains
unverified. The following records the first milestone's experiments, not
the current backend configuration.

Pinned `nightly-2026-11-05` (rustc 1.101.0-nightly, 2d20b42b8). Dev/test
default to Cranelift, four frontend threads and separate `.rmeta` metadata.
Shipping release and wasm remain LLVM. Linux uses mold. Build scripts and
the previously optimized codec/hot-loop packages retain LLVM.

**arm64 macOS must use LLVM throughout.** The Cranelift Node addon exceeded
the Mach-O section limit; JPEG and physics aborted on unsupported `sqxtn`
and `fmaxnm` ARM intrinsics. Package overrides fixed those failures but
mixed-backend UI/cook tests hung in HashMap insertion through SIMD generic
code. The whole-target LLVM fallback passed the same 75 selected controls
(determinism, snapshots, cooked assets, sound chunks and glTF frames).
Linux Cranelift and mold have not been executed on this Mac.

Windows MSVC is included in the dated nightly's target list. Its dev/test
configuration uses Cranelift with the explicit package fallbacks,
`-Zthreads=4`, and rust-lld's lld-link frontend; shipping remains LLVM.
**Windows is unverified:** no studio-win build was run from this lane.

Cargo metadata found 37 default integration binaries before, not 318: the
October consolidation already made most source files modules. There are
now 23 default integration binaries, each `tests/it/main.rs`, plus the
three optional package-owned kit targets. Core, blob, geometry and
localization contracts moved inline without removing assertions.

## Measurements and limits

These are single runs on a shared Mac, not controlled speed-up claims.
The lane's existing dependency cache was retained. No run below is a
clean dependency-cold build. Raw logs are in the lane's
`target/evidence/test-restructure/`.

| Command | Stable baseline | Nightly with Mac LLVM fallback |
|---|---:|---:|
| `cargo build --workspace --locked`, initial cached build | 108.81 s | 27.57 s after test compilation |
| Same command, no changes | 0.51 s | 0.23 s |
| `cargo nextest run --workspace --no-run --locked` | 63.22 s | 207.15 s after backend change |
| Selected 75 determinism/cook/render controls | — | 26.46 s wall, all passed |
| Full consolidated nextest suite | invalid baseline below | 354.98 s wall; 2787 passed, 26 ignored |

The build rows have different cache states and cannot establish a speed-up.
The earlier stable nextest run took 311.544 s (2792 passed, one failed,
30 skipped); its wasm subprocess encountered the subsequently edited
nightly config, so it is not a clean baseline either. Cranelift's failed
builds and mixed-backend hangs are excluded from timing comparisons, not
counted as successful fast runs.

Before category cuts, compared with the rebased base: 2820 textual Rust
test attributes, 241 playtests, 260 engine reference PNGs (89,081,222 B).
The imported test-cleanup removes the redundant baked-batches reference:
259 remain (88,873,738 B). `scripts/test-inventory.py --base 5387f79db`
verified that every retained reference PNG is byte-identical.

## Nightly lint repair, before category cuts

Rebased onto `2fee10f2b`, including the package-owned frame cuts. The
toolchain/consolidation milestone now passes the requested
`cargo clippy --workspace --all-targets --keep-going -- -D warnings` on
the pinned nightly (6.76 s incremental). No lint allowances were added.
The remaining Cargo manifest warning identifies `og-motion`'s test-only
pollster dependency; it is not a rustc/Clippy warning.

The full, untrimmed nextest suite passed: **2787 passed, 26 ignored**,
332.373 s execution, 394.70 s including incremental compilation. This is
a single shared-runner validation, not a controlled speed-up comparison.
Existing GPU comparisons passed without recording any references. The
pending category cuts were set aside throughout this validation.

## Engine category cuts

The retained integration modules cover each existing feature file with a
small representative set rather than every variant. Determinism, snapshot
restore and byte-contract checks remain. Variant-heavy GPU/layout cases,
one-off regressions and repeated constructor/validation cases were removed
by category. Audio mix and runtime extraction cost assertions are ignored
`perf_*` tests, selected only by the wave-end perf runner.

The engine keeps 54 reference PNGs: 37 renderer feature representatives,
12 UI frames and five existing DX12/Vulkan references for retained scenes.
Those five preserve documented backend differences at unchanged tolerance;
they are not additional feature scenarios. The cooked reference matrix
names the renderer features explicitly. Package-owned kit/style PNGs are
outside this selection.
GPU capability failures and unchecked playtest frames now fail rather
than pass, and the CPU-Vulkan tolerated-divergence exception is removed.

`scripts/test-inventory.py --base 5387f79db` reports:

| Engine measure | Before | After |
|---|---:|---:|
| Textual Rust `#[test]` attributes | 2820 | 1606 |
| Rust integration-source lines | 136651 | 79588 |
| Engine reference PNGs | 260 | 54 |
| Engine reference bytes | 89081222 | 14863841 |

The source reduction is 41.8%. The inventory's package-frame reduction
belongs to the separately landed kit stream, not this cut. Playtest cuts
are not included yet. Every crate that previously had Rust tests retains
tests; macro-generated tests are counted by nextest, not the textual count.

Validation on the pinned nightly: **1608 passed, 28 ignored**, 80.679 s
nextest execution and 85.05 s including incremental build. The GPU-slot
queue preceded the timed command. This single shared-runner run is not a
controlled comparison with the 332.373 s pre-cut run above.

Quick passed in 54.70 s with unchanged-input GPU and drift caching:
1015 passed, 7 ignored. The preceding uncached quick run passed Rust, fmt,
clippy and drift but failed three create-package checks because the copied
native `og` still rejected the newer `mood` grammar. Refreshing native
artifacts from this checkout fixed all three (28 create checks passed).
Neither that failed run nor the cached quick time measures a full-lane
speed-up. Gate missing-wasm guards also passed every case.

### One retained reference was stale

`crates/og-tests/references/metal/stylise-print.png` was recorded on Metal
at `fc9a08bb6` from source `845bd197f`, not on Chrome. It predates the
intentional hatch wash/fade change `7ee65c714`. The newly retained feature
comparison exposed the same historical difference noted at wave-end:
5.7386% outliers, RMSE 0.05095. Inspection found the same scene and geometry
with lighter hatch strokes. It was re-recorded on Metal, losslessly
recompressed and then passed the normal comparison. No tolerance changed.
This is the sole intentional retained-PNG change; do not describe all
retained frames as unchanged. Raw validation is in
`target/evidence/test-restructure/{print-reference,reduced-nextest-final}.log`.

## Template and lane cuts

Removed 149 replay playtests and 121 template/example reference PNGs.
Each affected project retains one smoke; 14 frame-bearing projects retain
two reference comparisons each. Arena retains its solo smoke and its
two-peer duel: create-game ships the latter for online-lockstep choices. Game,
game-2d and runner belong to the invariants stream; ui-kit and styles
belong to the kits stream and were not changed here. The six examples'
orphaned screen-schema helpers and snapshots were deleted with their
screen suites. Pixel-position pins in match3 and fx-showcase were removed.
Meridian uses its existing input-only published script (10.1 simulated
seconds), replacing the 121-second full replay.

Measured against 5387f79db, before rebasing these cuts beyond 92fe84b92,
repository-wide playtests fall from 241 to 92. Together with the separately
landed kit cut, total references fall from 1364 to 195 (54 engine, 141 package/example). All retained
template/example PNG bytes and tolerances are unchanged.

Native frame-smoke validation on the same engine: seven suites pass; seven
retain pre-existing reference failures. The original scripts were replayed
against the same game bundles and original reference bytes as controls.
They fail on the same retained comparisons, and their final simulation
hashes equal the reduced scripts' hashes. This is not a green template
suite and not a measured speed-up. Timings below are single runs under
variable shared load; they measure the playtest subprocess, not the lane.

| Smoke | Seconds | Result |
|---|---:|---|
| arena (simulation-only) | 0.484 | Pass |
| brasswick | 25.243 | Pass |
| fx-showcase | 3.944 | Pass |
| hollowbell | 17.178 | Existing title/combat mismatch |
| idle | 1.275 | Existing title/orbit mismatch |
| kestrel-stair | 22.389 | Pass |
| lantern-vale | 28.223 | Existing title/lantern mismatch |
| match3 | 0.660 | Pass |
| meridian-loop | 14.251 | Pass |
| rhythm | 4.428 | Existing title mismatch |
| roguelike | 1.695 | Existing title/dive mismatch |
| rumpus-isle | 23.699 | Existing title/bump mismatch |
| third-person | 5.470 | Pass |
| topdown | 0.778 | Pass |
| tower-defense | 3.866 | Existing title/wave mismatch |

Kestrel's strike initially failed after removing earlier captures
(RMSE 0.01108); its original script passed. Keeping its two earlier
captures as unreferenced render-history warm-ups restores the comparison
at the existing tolerance. Only title and strike remain stored references.
Rumpus bump's measured outliers vary slightly (0.363–0.365%) between the
original and reduced capture schedules; both fail the existing RMSE bound.
No template reference was re-recorded or tolerance widened.

Raw controls and candidate results are in
`target/evidence/test-restructure/smokes/*{-baseline,}.log`. The examples'
documented `tools/make-assets.ts` generators supplied their ignored music
files before validation; no generated tracked asset changed. Engine
typecheck and gate missing-wasm guard cases pass. Browser command plans
were checked with `--print`, but browser pixels, functional/full and perf
were not rerun: this checkout has no current optimized wasm manifest.

The Windows-only test lane and wasm import/export routing are removed,
as are the retired lavapipe routing stages. The same named lane selects
the same tests on every machine; GPU slot locks still queue rendering.
Independent kits/ui and kit-web routes remain. Budgets are targets in
docs/ci.md, not measured guarantees or skip permissions.

Remaining build-speed experiments are deferred to a later head at the
owner's request. Completed probes do not change compiler settings:
`embed-metadata=no` was slower (median 14.295 s versus 14.039 s across
five forced og-cook compiles each); `build-dir-new-layout` is already
enabled by Cargo 1.101 (the flag is a stabilized no-op). Spotlight was
already disabled on /Volumes/app. No dependency or workspace-hack change
is included in this deletion head. Windows remains unverified.

The template/lane head rebased cleanly onto 8be30462a, including the landed
engine cut f86cbac8d and invariants 443357b6b. Engine typecheck and shell
syntax checks pass after rebase. The combined inventory now has 1610
textual Rust tests, 79728 integration lines, 92 playtests and 194 reference
PNGs (54 engine, 140 package/example); the four added tests and runner's
one removed frame belong to the invariants stream. Smoke runtime evidence
above predates that rebase; it is not a claim to have rebuilt invariants.

Landing found that arena's duel is a shipped online-lockstep scaffold
contract. It was restored unchanged. After refreshing the native addon
and packed template overlay, all 28 create-package tests pass (1409
assertions, 12.43 s), including the published-package duel check. The cut
therefore removes 149 rather than 150 playtests.

## Missing adapters warn and leave checks unchecked (2026-11-06)

The owner's revised policy keeps simulation running when no GPU adapter
is available. Rust uses adapter-aware libtest discovery, Bun registers
GPU tests as skipped, and native commands report unchecked frames and
the adapter error. No reference PNG or tolerance changes in this head.

Validation on macrunner, nightly-2026-11-05, LLVM:

| Check | Result |
|---|---|
| Workspace nextest, `OG_RENDERER=none`, final tree | 1448 passed, 191 skipped, 0 failed; 38.506 s |
| GPU discovery reports | 177 GPU skips across six binaries, including explicitly ignored GPU perf tests; the remaining 14 skips are not GPU tests |
| Changed Bun test files, `OG_RENDERER=none OG_WASM_STALE=1 OG_GATE=1` | 24 passed, 35 skipped, 0 failed; 1210 assertions, 13.72 s |
| Metal Bun control: no-renderer, render, og-render | 15 passed, 0 failed; 127 assertions, 10.36 s |
| Direct libtest JSON, forced ignored motion GPU tests | 0 passed, 2 ignored, exit 0; adapter reason in both ignored events |
| Nextest forced unavailable GPU tests | Explicit unsupported-invocation error, 0 passed; rejected rather than misreported as passing |
| Workspace Clippy, all targets, `--keep-going -- -D warnings` | Pass |
| Native CLI/addon build and engine typecheck | Pass |
| Gate missing-wasm guard tests, shell syntax, diff whitespace | Pass |

Earlier controls before the final docs rebase and reporting changes:
the no-GPU baseline had 1449 passes, 163 failures and 28 ignored tests.
After skip registration it had the same 1449 passes, no failures and
191 skips. The Metal control ran 1611 tests successfully, with 29 skips
(28 existing ignores and the stale-wasm check excluded), in 93.377 s.
The later remove-polish landing removed one CLI test. These are correctness
controls, not same-load speed-up measurements.

An unconfigured broad Bun run before rebasing onto remove-polish was not
green: 523 passed, 33 skipped and 26 failed. It included stale-wasm host
comparisons, removed-polish expectations and a HUD text failure. Two
missing GPU registrations in reload tests were fixed; changed-file tests
above pass. This head does not claim a fresh wasm or a full functional gate.

Nextest cannot encode a runtime skip. Normal discovery works, but forcing
unavailable tests with `--run-ignored all/only` is explicitly rejected,
as approved by the owner. Direct libtest preserves ignored status even
with `--ignored`. GPU report files keep the gate from caching an unchecked
run as green. Windows and Linux adapter behavior remain unverified here.
Raw logs are in `target/evidence/test-restructure/`, named `no-gpu-final-workspace`,
`gpu-bun-changed-none`, `gpu-bun-final-metal`, `gpu-libtest-final`, and
`gpu-forced-nextest-final`.
