# og run --device: game

2026-10-25, android Pixel 10 (58051FDCR007QL), webgpu. The publish build served at http://localhost:49824/, opened cold, the gate tapped, then 5.01 s of play timed in the Worker.

## Scores

| | Score | Measure |
|---|---|---|
| First frame | 77 | ms from navigation to the first frame (1000 ms: 100, 4000 ms: 0; budget 2000 ms) |
| Frame time | 100 | p95 display interval of the played frames (17 ms: 100, 33.3 ms: 0) |

## Fixes

No finding crossed a threshold.

## Device

| | |
|---|---|
| User agent | Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/155.0.0.0 Mobile Safari/537.36 |
| Platform | Android, mobile |
| Tier | webgpu |
| WebGPU adapter | img-tec, d-series |
| WebGL2 renderer | ANGLE (Imagination Technologies, PowerVR D-Series DXT-48-1536, OpenGL ES 3.2) (Google Inc. (Imagination Technologies)) |
| DPR, viewport, screen | 2.625, 857×355 CSS px, 924×412 |
| Canvas | 2252×932 device px |
| Cores, memory | 8, 8 GB |
| Isolated, secure | yes, yes |

## Frames

| | count | mean | p50 | p95 | p99 | max |
|---|---|---|---|---|---|---|
| Display interval (ms) | 275 | 18.61 | 8.49 | 16.99 | 17.04 | 2637.98 |
| Engine work (ms) | 275 | 4.02 | 4.22 | 7.21 | 11.62 | 19.36 |

First frame 1677 ms, ready 1204 ms, play started 2269 ms after navigation; 275 frames and 153 ticks in 5.01 s (54.9 fps, 30.5 ticks/s).

## Served

| Path | COOP | COEP | CORP |
|---|---|---|---|
| / | same-origin | require-corp | cross-origin |
| /origin-engine-lanes_bg.wasm | same-origin | require-corp | cross-origin |
| /basis/basis_transcoder.wasm | same-origin | require-corp | cross-origin |

