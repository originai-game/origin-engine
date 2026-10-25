# Final-wave Metal verification

Candidate: `70c120c0b2653aab0d013403ab02641d9154a8d1`.
Mac main untouched; lane branch `r4/metal-candidate-4`.

## Step 1: pass

Metal functional: five steps pass, 204 seconds wall time. fmt 2s, clippy
15s, test 169s, drift 54s, bun-functional 133s. Rumpus: 30 checks pass in
5.989s; all eleven frames have zero outliers and RMSE 0.00000. No reference
was changed. The package's cross-host tests retain the functional lane's
stale-wasm skips.

## Step 2: four new release baselines recorded

Built the release test binaries before timing. The first recorded invocation
used the default test concurrency (`baselines/`). Repeated with
`RUST_TEST_THREADS=1` so the selected perf tests cannot contend; the committed
baselines use this serial repeat (`baselines-serial/`). Both commands exited
zero. Each batch waited for a full minute below one-minute load 8; the serial
repeat stayed at **4.10–4.25** throughout.

```sh
RUST_TEST_THREADS=1 OG_PERF_RECORD=1 cargo test -p og-runtime --release -- --ignored perf --nocapture
RUST_TEST_THREADS=1 OG_PERF_RECORD=1 cargo test -p og-tests --release -- --ignored perf_materials_physical perf_showcase_capture --nocapture
```

| Baseline | Best ms |
|---|---:|
| extract-sprites-20k | 5.537 |
| extract-models-20k | 1.999 |
| materials-physical | 0.828 |
| showcase-capture | 8.108 |

These are the existing harness's best-run baselines, not p95. Showcase
capture passes its built-in <30ms check. All seven gfx runs are in the JSONs.
The broad runtime command also recorded three existing baselines; those
files were restored, not silently raised. Its sphere extraction measured
0.798ms versus the existing 0.531ms baseline; this is a possible regression
to investigate, not a new accepted baseline. Static/moving raycast measured
0.307/1.140ms. Raw outputs retain all numbers.

## Step 3: wave end passes, including both browser tiers

Ran `scripts/wave-end.sh --stages gate`, then `web`, `conformance`, and
`release`, serially, stopping between stages on failure. This is the script's
default stage order, split to honor the coordinator's stop-on-failure rule
(the default invocation would keep going after a failed stage).

| Stage | Result | Seconds |
|---|---|---:|
| Full gate (`check.sh --all`) | PASS, 9/9 | 778 |
| Browser reference tiers (`check.sh web-reference`) | PASS, WebGPU and WebGL2 | 493 |
| Checker conformance | PASS | 14 |
| Release pack + local/container install verification | PASS | 623 |

The full gate ran 1,320 Rust tests successfully (18 skipped), and all 328
engine Bun tests passed with no skipped tests. Full per-step timings are in
`full-summary.txt`. Browser comparisons all pass their existing tolerances;
no reference was re-recorded. This does not claim byte identity for all
browser frames or an isolated opt-level s-versus-3 comparison.

Determinism passes across native debug/release, lanes off/on, wasm scalar,
SIMD and both lanes profiles. **anim_hash 15495097786948894722** is recorded
in docs/phase-1.md with the receipt. Native lanes performed 1,067 parallel
calls across four threads. Release packaging/verification was local only;
no npm package was published or deployment performed.

## Step 4: captures pass; WebGL2 comparison fails, further timing stopped

Same capture runners as the earlier measurement, 30 measured calls after
60 warm frames and three discarded captures, 1280×720. Release native addon;
web uses the diagnostic non-publish Worker path. Every PNG decoded with the
correct dimensions; native template and web Meridian outputs were inspected.
All four maxima, not just their p95s, are within their respective limits.
Full idle minute before the batch; load **4.18–4.21** throughout.

| Scene | Host | p50 ms | p95 ms | Max ms | Limit ms | Verdict |
|---|---|---:|---:|---:|---:|---|
| Template | native Metal | 5.518 | 6.124 | 6.978 | 30 | PASS |
| Meridian | native Metal | 9.773 | 18.036 | 19.167 | 30 | PASS |
| Template | Chrome WebGPU | 15.560 | 15.995 | 15.995 | 60 | PASS |
| Meridian | Chrome WebGPU | 23.780 | 31.975 | 32.330 | 60 | PASS |

The WebGL2 comparison uses the unchanged runner, 600 measured frames after
60 warm frames, two rounds per mode. Each mode gets its own idle minute;
all loads **2.90–3.63**. Per-run JSONs and commands are retained.

| Scene | Mode | p50 ms | p95 ms | Sustained ms | FPS |
|---|---|---:|---:|---:|---:|
| Reference | engine WebGL2 | 1.483 | 3.075 | 1.996 | 501 |
| Reference | three.js WebGL2 instanced | 2.988 | 3.583 | 2.540 | 394 |
| 20k spheres | engine WebGL2 | 3.525 | 4.288 | 3.674 | 272 |
| 20k spheres | three.js WebGL2 instanced | 2.375 | 2.515 | 2.385 | 419 |

Reference frame-rate ratio: **2.540 / 1.996 = 1.273×, FAIL versus ≥1.5×**.
The sphere ratio is 0.649× (no separate numeric sphere budget). These are
the harness's sustained wall-time rates, not GPU timestamp measurements;
do not substitute the ratio of p50 frame times for the specified frame-rate
comparison. No budget or baseline was adjusted to make this pass.

Stopped after the already-running four-mode comparison batch, per the
coordinator's failure rule. **Lantern's new release render p95 and the
separate Chrome first-frame/frame-time checks were not run.** The passing
browser-reference stage is not a substitute for those timing checks.

## Step 5 number available from the completed shipping-profile size gate

3D lanes: **3,370,739 bytes Brotli, 80.3647% of 4,194,304 bytes**, PASS.
Raw optimized wasm: 12,052,826 bytes. The packaged 3D lanes module and the
size gate's artifact have identical SHA-256:
`d7beaa92824490109cfdfff8bfd01db6d643db61eea3d8135c80e2b9934a46f5`.
No new size run was started after the WebGL2 failure.

`bun tests/evidence/r4/final-metal/summarize.mjs` verifies receipt counts,
load qualification and comparison inputs, regenerating `summary.json`.
Evidence PNGs are presentation copies, resized to ≤640px and <150KB after
validation; raw numeric results are unchanged. No claim of a complete wave:
the remaining timings require a coordinator decision after the FAIL.

## Follow-up: remaining timings completed

The coordinator authorised continuing on `7ca4e251`, without treating the
WebGL2 comparison miss as a blocker. The [remaining timing receipts](outstanding/README.md)
record five release-unpaced rounds per game, five Chrome runs per game and
tier, and three serial sphere-baseline repeats. Lantern passes in all five
runs. Chrome passes the existing device timing checks, with one retained
display-gap outlier and no matched pre-size-2 comparison. Sphere frame cost
repeatedly exceeds its unchanged baseline. No source or reference changed.
