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
