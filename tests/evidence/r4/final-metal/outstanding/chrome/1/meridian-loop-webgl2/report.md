# og run --device: meridian-loop

2026-10-25, chrome (Google Chrome for Testing, headless), webgl2. The publish build served at http://127.0.0.1:65061/?webgl=1, opened cold, the gate tapped, then 10 s of play timed in the Worker.

## Scores

| | Score | Measure |
|---|---|---|
| First frame | 84 | ms from navigation to the first frame (1000 ms: 100, 4000 ms: 0; budget 2000 ms) |
| Frame time | 100 | p95 display interval of the played frames (17 ms: 100, 33.3 ms: 0) |

## Fixes

No finding crossed a threshold.

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
| Display interval (ms) | 1196 | 8.36 | 8.34 | 8.34 | 8.34 | 16.67 |
| Engine work (ms) | 1196 | 1.57 | 1.58 | 2.44 | 2.67 | 2.85 |

First frame 1482 ms, ready 421 ms, play started 1645 ms after navigation; 1196 frames and 599 ticks in 10 s (119.6 fps, 59.9 ticks/s).

## Served

| Path | COOP | COEP | CORP |
|---|---|---|---|
| / | same-origin | require-corp | cross-origin |
| /origin-engine-lanes_bg.wasm | same-origin | require-corp | cross-origin |
| /basis/basis_transcoder.wasm | same-origin | require-corp | cross-origin |

