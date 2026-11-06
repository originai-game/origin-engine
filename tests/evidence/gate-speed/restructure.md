# Test restructuring, 2026-11-06

Base: `5387f79db`, including the genre UI relocation. Package-owned kit and
style suites remain independent; their frame cuts belong to kit-frames-cut.

## Nightly and binary consolidation

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
