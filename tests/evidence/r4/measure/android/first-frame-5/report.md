# og run --device: game

2026-10-25, android Pixel 10 (58051FDCR007QL), webgpu. The publish build served at http://localhost:49895/, opened cold, the gate tapped, then 5.01 s of play timed in the Worker.

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
| Display interval (ms) | 278 | 18.28 | 8.49 | 8.59 | 17.06 | 2637.55 |
| Engine work (ms) | 278 | 3.73 | 3.86 | 6.75 | 11.19 | 20.89 |

First frame 1691 ms, ready 1238 ms, play started 2337 ms after navigation; 278 frames and 150 ticks in 5.01 s (55.5 fps, 29.9 ticks/s).

## Served

| Path | COOP | COEP | CORP |
|---|---|---|---|
| / | same-origin | require-corp | cross-origin |
| /origin-engine-lanes_bg.wasm | same-origin | require-corp | cross-origin |
| /basis/basis_transcoder.wasm | same-origin | require-corp | cross-origin |

