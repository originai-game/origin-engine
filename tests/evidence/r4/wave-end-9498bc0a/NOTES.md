# Mac wave-end, 9498bc0a (2026-10-26)

**FAILED:** the determinism fixture does not compile. No source, baseline
or reference was edited. This is a failed wave receipt, not closure of
row 53. Machine: macrunner, M4 Pro, macOS 27.2; Chrome for Testing
150.0.7871.115. Revision: `9498bc0a48846ae38a54256e7bfbeaf8b5d1d0a5`.

## Commands and outcomes

Run serially; 60-second pauses before each measurement/gate:

```sh
# packages/engine: fresh release addon and all shipping wasm variants
OG_RELEASE=1 sh scripts/build.sh
# repository root: refresh the gate artifacts web-reference's build.sh copies
OG_WASM_PROFILE=wasm scripts/check.sh --only wasm-opt
scripts/check.sh web-reference
# packages/engine
bun tests/web/gpu-timing.ts
# repository root, no timer shim
bun tools/bench/gpu-passes.ts --mode webgl2 --out .amp/in/wave-9498bc0a/gpu-passes-webgl2.json
bun tools/bench/gpu-passes.ts --mode webgpu --out .amp/in/wave-9498bc0a/gpu-passes-webgpu.json
env -u OG_PERF_RECORD bun tools/bench/threejs-compare.ts --only complex --rounds 2 --out tests/evidence/r4/bench-complex/mac-3
scripts/wave-end.sh --stages gate,web
# After preserving the failed wave: collect the cancelled size measurements
OG_WASM_PROFILE=wasm scripts/check.sh --only size-gate
```

Only the requested gate (`--all`) and web-reference stages were selected;
conformance, release packing/verification and optional lavapipe stages
were not run. `summary.txt` and `wave-end.log` are exact copies of the
script's files under `target/evidence/wave-end/`, preserved before any
follow-up. The standalone size rerun does not turn the failed wave green.

| Step | Result |
|---|---|
| Release build | PASS |
| Gate wasm build / optimize | PASS, 211 s |
| Initial web-reference | PASS, 526 s; no out-of-tolerance frame |
| Standalone gpu-timing | PASS, 5 checks, real extension, no shim |
| GPU pass profiles | Completed on both tiers, 21 pass names each |
| Complex benchmark | Completed all ten runs; WebGPU budget PASS |
| Wave gate | FAIL, 283 s: determinism E0308 |
| Wave web | PASS by path-aware reuse of the initial web-reference pass |
| Separate size gate | PASS, 22 s (23 s including reused dependencies) |

Wave gate fmt, clippy, wasm-build, test, wasm-opt and drift passed. Rust
tests passed in 240 s. Determinism failed after 17 s:

```text
tools/wasm-determinism/src/lib.rs:187
put(&mut h, &c.samples);
expected &[f32], found &Samples
error[E0308]: mismatched types
```

The gate cancelled size-gate and never reached bun-test. There is no
determinism hash result for this revision. See `determinism.log`.

## WebGL2 calls and real GPU timer availability

`gl-calls.ts`: **5/5 checks PASS**, 1,344.933 calls/frame (rounded 1,345),
below 1,500; 41 draws/frame. `gl-calls.json` retains counts by GL function.

`gpu-timing.ts` reports WebGL2 `gpuTimestamps=true`, the browser's native
extension and nonzero `main`/`tonemap` timings matching CPU pass names.
The no-extension case is skipped because this Chrome exposes the extension.

The reference profiles have 21 timed pass names: 600 GPU trace frames
for 600 requested WebGL2 frames, and 601 for WebGPU (asynchronous readback
can include a prior frame). The extension reports TIMESTAMP_EXT=0 bits,
TIME_ELAPSED_EXT=64 bits. Raw reported frame sums:

| Tier | Mean ms | p50 ms | p95 ms |
|---|---:|---:|---:|
| WebGL2 | 36.4824 | 36.030 | 54.852 |
| WebGPU | 3.9283 | 3.934 | 5.167 |

**The Metal WebGL2 sums are not an uninstrumented pass-cost breakdown.**
ANGLE flushes a Metal command buffer at every TIME_ELAPSED begin/end,
including scheduling in each interval. Tiny bloom passes consequently
show a roughly 2.2–2.46 ms floor. This is the documented behavior in
`tests/evidence/debt/webgl2-gpu-timing/NOTES.md`, “Caveat for reading Metal
numbers”; these receipts establish real query availability and pass
coverage, not that bloom costs that much in a shipping frame. Publish
benchmark frames have GPU timers disabled.

## Complex benchmark and wasm size

The full table and visual checks are in `../bench-complex/NOTES.md`,
“Mac-3 numbers”; raw data and all five PNGs are in `../bench-complex/mac-3/`.
Two-round median/mean engine time: WebGPU **5.228 ms**, WebGL2 **7.592 ms**.
Three per-tile/engine: **5.838× WebGPU PASS**, **1.315× WebGL2** reported;
WebGL2 per-part **3.193×**. The 3.42% lower WebGL2 time versus Mac-2's
7.861 ms is smaller than this run's spread, so no isolated patch speedup
is claimed. One-minute run-start load: 3.71–5.62.

Shipping `wasm` profile, brotli-q11, each below the unchanged 4,194,304-byte
per-download budget (`size-rerun.log`):

| Download | Brotli bytes | Result |
|---|---:|---|
| 3D lanes | 3,232,640 | PASS |
| 2D lanes | 2,992,605 | PASS |
| 3D publish | 3,204,738 | PASS |
| 2D publish | 2,962,362 | PASS |

Load before the initial browser lane was 9.64 after the build; before the
standalone timer test 4.42, GL2 profile 3.93, WebGPU profile 3.17, complex
benchmark 3.71, wave 3.75 and size-only rerun 11.24. Gate durations and
compressed byte counts are not idle runtime benchmarks. No load-based
speedup claim or baseline adjustment is made.
