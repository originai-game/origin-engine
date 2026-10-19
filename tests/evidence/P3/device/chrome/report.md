# og run --device: game

2026-10-19, chrome (Google Chrome for Testing, headless), webgpu. The publish build served at http://127.0.0.1:55167/ (LAN http://192.168.6.243:55167/), opened cold, the gate tapped, then 5 s of play timed in the Worker.

## Scores

| | Score | Measure |
|---|---|---|
| First frame | 100 | ms from navigation to the first frame (1000 ms: 100, 4000 ms: 0; budget 2000 ms) |
| Frame time | 100 | p95 display interval of the played frames (17 ms: 100, 33.3 ms: 0) |

## Fixes

No finding crossed a threshold.

## Device

| | |
|---|---|
| User agent | Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/150.0.0.0 Safari/537.36 |
| Platform | macOS |
| Tier | webgpu |
| WebGPU adapter | apple, metal-3 |
| WebGL2 renderer | ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Pro, Unspecified Version) (Google Inc. (Apple)) |
| DPR, viewport, screen | 1, 1280×577 CSS px, 800×600 |
| Canvas | 1280×577 device px |
| Cores, memory | 12, 16 GB |
| Isolated, secure | yes, yes |

## Frames

| | count | mean | p50 | p95 | p99 | max |
|---|---|---|---|---|---|---|
| Display interval (ms) | 601 | 8.33 | 8.33 | 9.82 | 10.26 | 10.31 |
| Engine work (ms) | 601 | 0.51 | 0.54 | 0.73 | 0.84 | 1 |

First frame 270 ms, ready 200 ms, play started 483 ms after navigation; 601 frames and 300 ticks in 5 s (120.2 fps, 60 ticks/s).

## Served

| Path | COOP | COEP | CORP |
|---|---|---|---|
| / | same-origin | require-corp | cross-origin |
| /origin-engine_bg.wasm | same-origin | require-corp | cross-origin |
| /basis/basis_transcoder.wasm | same-origin | require-corp | cross-origin |

