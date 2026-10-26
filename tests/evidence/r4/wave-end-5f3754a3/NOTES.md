# Mac wave-end at 5f3754a3 — PASS

2026-10-26, macrunner (M4 Pro, macOS 27.2), measure worktree
`/Volumes/app/origin-engine-measure`, reset to
`5f3754a377e1272b20da76b48c17898611fe3424` before the run. No source,
baseline or reference was edited. Only these receipts were added afterward.

After fetching, creating the receipt branch at that revision and resetting
the clean measure lane, the command was:

```sh
sleep 60
uptime
CI=1 scripts/wave-end.sh --stages gate,web
```

`CI=1` forces every gate step and fresh browser references, with no
path-aware reuse of the earlier web pass. The script serializes the full
gate (`scripts/check.sh --all`) then `scripts/check.sh web-reference`
under the machine and target locks. Only the requested gate/web stages
were selected; conformance and release pack/verify were not run. The
one-minute load before the run was 4.03; these are correctness/size checks,
not idle runtime performance measurements. No complex benchmark or
standalone GPU timing rerun was performed.

## Every requested stage passed

Total: **1,273 s**. `summary.txt` and `wave-end.log` are exact copies of
the script's outputs from `target/evidence/wave-end/`. The individual
step logs are copied from `target/evidence/gate/`.

| Stage / step | Result | Seconds |
|---|---|---:|
| Full gate | PASS, 9/9 | 789 |
| fmt | PASS | 3 |
| clippy | PASS | 20 |
| wasm-build | PASS | 332 |
| test | PASS | 321 |
| wasm-opt | PASS | 235 |
| size-gate | PASS | 35 |
| drift | PASS | 28 |
| bun-test | PASS | 183 |
| determinism | PASS | 238 |
| Fresh web-reference | PASS | 484 |

The gate's steps may overlap internally; durations are not additive.
No native, template or browser reference failed or moved outside its
existing tolerance. No frame was re-recorded. The browser log retains
the per-frame outlier percentages and RMSE values; passing does not mean
every browser image is pixel-identical to Metal.

The reverted `gl-calls.ts` check is absent. The retained `gpu-timing.ts`
ran only as part of web-reference: native extension, WebGL2
`gpuTimestamps=true`, named nonzero `main` and `tonemap` timings, all five
checks passed, no shim. The no-extension case is skipped on this Chrome
because the native extension is present.

## Shipping wasm sizes

Fresh shipping `wasm` profile, brotli-q11; each download is below the
unchanged **4,194,304 B** limit. See `size-gate.log`.

| Download | Wasm bytes | Brotli bytes | Result |
|---|---:|---:|---|
| 3D lanes | 11,358,907 | 3,232,252 | PASS |
| 2D lanes | 10,483,642 | 2,988,274 | PASS |
| 3D publish | 11,066,631 | 3,202,050 | PASS |
| 2D publish | 10,199,001 | 2,959,886 | PASS |

## Determinism

**Every build agrees.** `determinism.log` records each native/wasm build
and its results; a build without a physics dimension reports `-` for it.

| Digest | Value |
|---|---|
| math_digest | `6117185439544271997` |
| bouncing_boxes_hash | `15551159556835976588` |
| physics3d_hash | `10053948282410406339` |
| physics2d_hash | `3002410719564870113` |
| anim_hash | `12806465523503570878` |

The animation hash matches the coordinator's orb result. This run
supersedes the failed determinism-compilation wave at `9498bc0a` without
altering or concealing that earlier receipt.
