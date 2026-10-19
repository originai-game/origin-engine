# og run --device: game

2026-10-19, chrome (Google Chrome for Testing, headless), webgl2. The publish build served at http://127.0.0.1:64907/?webgl=1 (LAN http://192.168.6.243:64907/), opened cold, the gate tapped, then 5.01 s of play timed in the Worker.

## Scores

| | Score | Measure |
|---|---|---|
| First frame | 96 | ms from navigation to the first frame (1000 ms: 100, 4000 ms: 0; budget 2000 ms) |
| Frame time | 0 | p95 display interval of the played frames (17 ms: 100, 33.3 ms: 0) |

## Fixes

| Measure | Where | Found | Fix |
|---|---|---|---|
| frame time | webgl2 | p95 display interval 37.27 ms (p99 50, max 63.21): frames are missed | og-game §14: fewer draws and particles, less per-tick TypeScript; og polish names the step and render cost |

## Device

| | |
|---|---|
| User agent | Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/150.0.0.0 Safari/537.36 |
| Platform | macOS |
| Tier | webgl2 |
| WebGPU adapter | apple, metal-3 |
| WebGL2 renderer | ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Pro, Unspecified Version) (Google Inc. (Apple)) |
| DPR, viewport, screen | 1, 1280×577 CSS px, 800×600 |
| Canvas | 1280×577 device px |
| Cores, memory | 12, 16 GB |
| Isolated, secure | yes, yes |

## Frames

| | count | mean | p50 | p95 | p99 | max |
|---|---|---|---|---|---|---|
| Display interval (ms) | 338 | 14.86 | 8.34 | 37.27 | 50 | 63.21 |
| Engine work (ms) | 338 | 1.08 | 1.13 | 1.41 | 1.58 | 1.66 |

First frame 1125 ms, ready 944 ms, play started 1455 ms after navigation; 338 frames and 300 ticks in 5.01 s (67.5 fps, 59.9 ticks/s).

## Served

| Path | COOP | COEP | CORP |
|---|---|---|---|
| / | same-origin | require-corp | cross-origin |
| /origin-engine_bg.wasm | same-origin | require-corp | cross-origin |
| /basis/basis_transcoder.wasm | same-origin | require-corp | cross-origin |

