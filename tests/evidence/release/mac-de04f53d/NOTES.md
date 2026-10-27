# macrunner metal lane at de04f53d (0.6.1 candidate): PASS

2026-10-27 (NZDT), macrunner, thread
T-01a17bee-78ce-74b9-93cf-559c06e76fc1. Commit tested: `de04f53d`
(origin/main: 0.6.0, the five RedGPU render ports, the debug og-gfx stack
fix, and the merge of `webgl2-cluster-table` b6340a10). Nothing was fixed,
re-recorded or re-tolerated; this branch only adds this directory.

## Machine

| | |
|---|---|
| Mac | Mac16,8, Apple M4 Pro, 24 GB |
| GPU | Apple M4 Pro, 16-core GPU, Metal 4 (wgpu adapter "Apple M4 Pro", tier `metal`) |
| macOS | 27.2 (26B5091g) |
| Chrome | Chrome for Testing 150.0.7871.115, headless, the harness's default (`tests/web/cdp.ts`, `--use-angle=metal`); WebGL2 adapter "ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Pro, Unspecified Version)" |
| Toolchain | rustc 1.99.1, bun 1.4.4, node v26.3.0 |

## 1. Functional lane, main checkout at de04f53d

`scripts/check.sh functional`: **gate PASSED (5/5)**: fmt 2 s, clippy
22 s, test 201 s (nextest 1393 run, 1393 passed, 19 skipped; all 68
`gfx::` tests among them), drift 76 s, bun-functional 118 s (16 + 302 + 49
pass, 0 fail). `functional.log`. No failures, so nothing new since 2901621b
(`../mac-95104f3e/`).

Each og-tests scene's metal numbers (`cargo nextest run -p og-tests
--no-capture`, 309/309 passed; `metal-rmse.txt`, 98 scenes): every lit
scene this change touched is at RMSE 0.00000 against its metal reference
(max channel diff ≤ 1): point-spot-lights, rect-lights, lights-256,
light-cookies, materials-physical, probe-interior, probe-two-rooms,
terrain-far, terrain-generated, terrain-grass, terrain-grass-wind. The
non-zero ones (terrain-near 0.00657, water-planar 0.00491, terrain-mid
0.00362, particles 0.00230, sprites-20k 0.00229, probe-outdoor 0.00173,
water-shadows 0.00169, particles-cpu 0.00166) were rerun at 95104f3e in
lane-3: the same numbers to five decimals, so they predate this change.

## 2. web-reference, measure lane pinned at de04f53d

`scripts/check.sh --only wasm-opt` (wasm-build 217 s, wasm-opt 115 s;
`wasm.log`), then `scripts/check.sh web-reference`: **PASSED (559 s)**.
The served `dist/web/origin-engine_bg.wasm` carries b6340a10's
`frame_lights`. `web-reference.log`.

| Tier | frames with RMSE | FAIL | max RMSE | vs 95104f3e wave |
|---|---:|---:|---:|---|
| webgpu | 533 | 0 | 0.00921 (ghost) | every frame's RMSE identical to 5 decimals |
| webgl2 (ANGLE on Metal) | 530 | 0 | 0.00901 (controller) | every frame's RMSE identical to 5 decimals |

1143 `ok` lines, no FAIL; the 95104f3e wave's 1167 counted 24 lines of
conformance and determinism from its wave-end log, so the web-reference
set is the same 1143. fx-showcase 5/5, E5 26/26, looks 37/37, og-tests
gfx 132/132, og-ui kit 16/16 playtests, Ember Vale 3, Brasswick 9,
Meridian Loop 10, lane-panic 23/23, a11y 26/26, GPU timing, per tier.

The five RedGPU scenes, as at 95104f3e (≤ 0.00376):

| Scene | webgpu RMSE / outliers | webgl2 RMSE / outliers |
|---|---|---|
| post-adjust | 0.00376 / 0.004 % | 0.00365 / 0.004 % |
| post-neutral | 0.00000 / 0 | 0.00002 / 0 |
| post-lens | 0.00352 / 0.007 % | 0.00334 / 0.006 % |
| terrain-grass-wind | 0.00085 / 0.002 % | 0.00285 / 0.014 % |
| post-blur | 0.00000 / 0 | 0.00003 / 0 |

WebGL2 lit frames read (`webgl2-lit.jpg`: point-spot-lights, rect-lights,
materials-physical, terrain-near): light pools, area-light reflections,
material responses and terrain shadows all present; webgl2 RMSE 0.0027,
0.00002, 0.00005, 0.00661.

## Not done

Safari (no harness tier), vulkan-cpu (set aside), the rest of the wave's
end (determinism, size gate, release pack) were not asked for and not run.
