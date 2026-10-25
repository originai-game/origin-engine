# og run --device: game

2026-10-25, chrome (Google Chrome for Testing, headless), webgpu. The publish build served at http://127.0.0.1:65300/, opened cold, the gate tapped, then 10.01 s of play timed in the Worker.

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
| Display interval (ms) | 1201 | 8.33 | 8.34 | 8.34 | 8.34 | 8.57 |
| Engine work (ms) | 1201 | 0.85 | 0.83 | 1.78 | 2.07 | 2.71 |

First frame 300 ms, ready 188 ms, play started 514 ms after navigation; 1201 frames and 600 ticks in 10.01 s (120 fps, 59.9 ticks/s).

## Served

| Path | COOP | COEP | CORP |
|---|---|---|---|
| / | same-origin | require-corp | cross-origin |
| /origin-engine-lanes_bg.wasm | same-origin | require-corp | cross-origin |
| /basis/basis_transcoder.wasm | same-origin | require-corp | cross-origin |

