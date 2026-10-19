# Phase 2 terrain and water: frame times

The budget is og-game §14's: step plus render under half a 60 Hz frame
(8.3 ms) on a desktop. Every number below is under it. The machine was shared
with four other agents' work (load average 18 to 55), so these are upper
bounds.

## Native, metal (og-gfx, release, 1280×720, 300 frames)

`og-gfx terrain-near terrain-mid terrain-far terrain-grass water-planar
water-shadows water-underwater --frames 300`: the mean frame including the
GPU's work, and the CPU's share of it.

| Scene | frame ms | CPU submit ms | first frame ms (pipelines, uploads) |
|---|---|---|---|
| `terrain-near` | 1.457 | 0.927 | 120.8 |
| `terrain-mid` | 1.721 | 0.930 | 86.6 |
| `terrain-far` | 1.916 | 1.003 | 95.3 |
| `terrain-grass` | 4.282 | 4.273 | 81.6 |
| `water-planar` | 1.639 | 1.065 | 91.6 |
| `water-shadows` | 2.485 | 1.175 | 109.7 |
| `water-underwater` | 1.413 | 0.952 | 107.5 |

## Chrome, the fx-showcase terrain station (WebGPU and WebGL2)

`bun tests/web/reference.ts [--webgl] --script "0.5:station:terrain"
--shots terrain@330 --time 300` (Chrome for Testing, headless, 1280×720,
DPR 1, the dev web export): the station's frame matched its reference at
RMSE 0, then 300 more frames, each one tick and one render through the
engine Worker; "with GPU" is the 300 frames plus one capture (whose
readback waits for the GPU) over 300. The water station is the same
measurement on an E5 station for comparison.

| Tier | Station | median ms | p95 ms | with GPU ms |
|---|---|---|---|---|
| webgpu | terrain | 1.34 | 2.60 | 3.08 |
| webgpu | water (E5) | 0.96 | 1.93 | 3.17 |
| webgl2 | terrain | 2.43 | 4.17 | 4.08 |
| webgl2 | water (E5) | 1.98 | 6.25 | 3.28 |

The terrain station is a 256 m terrain with four layers, a lake with a
half-resolution planar reflection, shadows and caustics, the character
and the HUD.
