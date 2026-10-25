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
