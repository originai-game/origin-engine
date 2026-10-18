# E4 evidence: 2D and game feel (2026-10-18)

Branch `e4-2d-feel`, b6da6a1..HEAD. The full gate (`scripts/check.sh`) passed
at 738a66a: fmt 1 s, clippy 0 s (cached), wasm-build 7 s, test 90 s,
wasm-opt 26 s, size-gate 17 s, drift 1 s, bun-test 44 s, determinism 43 s.

| File | What it shows |
|---|---|
| `tilemap-perf.md`, `tilemap-256-metal.png` | a 256×256 four-layer map at 1280×720: 0.223 ms per frame on the M4 Pro (budget 1 ms); the whole map in view 2.58 ms |
| `flipbook-events.json` | the frame events a flipbook clip reports, tick by tick, in loop, ping-pong and once modes |
| `tween-hit-stop-determinism.json` | tweens of every kind, presets, a hit stop, slow motion and camera feel: the same hashes at five checkpoints on the native and web hosts and twice natively; a snapshot taken inside the hit stop replays to the same end; without the hit stop the hash differs (asserted by `packages/engine/tests/hosts.test.ts`) |
| `archetypes-vulkan-cpu.json` | the three archetypes' playtests on lavapipe (E2B): every suite passes, frames against `metal` within RMSE 0.0031, the recorded `vulkan-cpu` references reproduced at RMSE 0 |
| `break-and-restore.md` | every new test broken once and seen to fail, then restored |

Topdown reference frames (`templates/archetypes/topdown/references/`):
`metal/`, `vulkan-cpu/`, `webgpu/` and `webgl2/` each hold `start.png`,
`crit.png`, `dash.png` and `east.png`. Only two native tiers exist (og-render
selects Metal or Vulkan natively; there is no native GL or DX12 tier on this
hardware), so the browser tiers from `packages/engine/tests/web/reference.ts
--project templates/archetypes/topdown` are the third and fourth: within
RMSE 0.00009 of `metal`.

Wasm: the publish variant is 2,517,667 bytes brotli-q11, 60 % of the 4 MB
budget (57 % before E4). By code bytes in the unoptimised module,
og-api's feel runtime is about 1.5 % (≈39 KB brotli), og-tilemap 0.8 %
(≈21 KB) and og-tween 0.6 % (≈14 KB). The Tiled and LDtk importers are in
og-cook's native `cook` feature only.
