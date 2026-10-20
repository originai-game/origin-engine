# P4 physical materials: evidence

Branch `r-materials` from main b62dd65, files written at 370007a, 2026-10-20,
Apple M4 Pro (metal), the og-lavapipe container (vulkan-cpu: Mesa llvmpipe,
LLVM 19.1.7, binaries cross-built with `cargo zigbuild --release --target
x86_64-unknown-linux-gnu.2.31`), Chrome for Testing 150 (webgpu, webgl2).

| File | What it is and how it was made |
|---|---|
| `materials-physical-tiers.png` | og-tests' `materials-physical` frame on metal (top left), vulkan-cpu (top right), webgpu (bottom left), webgl2 (bottom right), each halved. Metal and vulkan-cpu are the committed references; the browser frames are `bun tests/web/reference-gfx.ts [--webgl] --scene materials-physical,materials` in packages/engine. Every lobe draws on every tier: webgl2 matches metal at RMSE 0.00000, webgpu at 0.00003, vulkan-cpu at 0.0018 (7 outlier pixels). |
| `reference-gfx-webgl2.json`, `reference-gfx-webgpu.json` | those browser runs' reports (`materials` too: RMSE 0.00000 and 0.00001). The first WebGL2 run failed the physical pipeline (`gl_1`: a reserved GLSL name naga made from a second `gl` local); fixed in a46413b, which also makes og-render's tests/webgl2.rs catch it. |
| `og-gfx-metal-before.txt`, `og-gfx-metal-after.txt` | `cargo run -p og-tests --bin og-gfx -- --all --capture DIR` with `crates/` at b62dd65 and at this branch: each scene's RMSE, max channel difference and outliers against its committed reference. Identical lines for all 48 earlier scenes (the non-zero ones, terrain and water among them, were so on main); the 48 captured PNGs compared pixel by pixel are byte-identical before and after. |
| `og-gfx-vulkan-cpu-before.txt`, `og-gfx-vulkan-cpu-after.txt` | the same with og-gfx cross-built at b62dd65 and at this branch, run in the og-lavapipe container: identical lines, and the 48 earlier frames byte-identical before and after. |
| `fx-showcase-metal-main-engine.txt` | `og test templates/fx-showcase` with this branch's template on main's engine (crates/ at b62dd65, dev addon and og): the earlier stations' RMSE against their references are the same numbers as on this branch's engine; the new `physical` station fails there (RMSE 0.075: no physical lobes). |
| `fx-showcase-vulkan-cpu.txt` | `OG_FX_CAPTURE=physical.png og test .` in templates/fx-showcase inside the og-lavapipe container (og and the addon cross-built): all 20 suites pass; the `physical` frame recorded is RMSE 0.00128 against metal with no outliers. |
| `size-gate-before.json`, `size-gate-after.json` | `scripts/check.sh wasm-build wasm-opt size-gate` with `crates/` at b62dd65 and at this branch: publish 3,205,718 to 3,224,361 B brotli (+18,643 B, 76.43 % to 76.87 % of 4,194,304), lanes 3,209,121 to 3,226,482 B (+17,361 B). |
| `check-summary-concurrent.txt` | the last concurrent path-aware `scripts/check.sh` (load average 45 to 60 from other work on the machine): fmt, clippy, determinism pass, wasm-build, wasm-opt, size-gate and drift skipped as passed on this tree, bun-test failed by four 5-second test timeouts in packages/engine's headless render tests (render.test.ts, asset-stream.test.ts; each takes about 1 s alone) and test was cancelled with it. On the same tree, each step passed on its own: `OG_GATE_JOBS=1 scripts/check.sh` (fmt, clippy, wasm-build, test 575 of 575, wasm-opt, size-gate), `scripts/check.sh determinism` (215 s) and `scripts/check.sh bun-test` (44 s); drift passed in the first full run. |

The tests: og-render `tests/webgl2.rs` (the physical and pack shaders
translate to GLSL ES 3.0 with no reserved names; 14 textures), og-render's
physical.rs unit tests (the pack plan), og-cook `tests/physical.rs` (the
seven KHR extensions through the glTF bridge and the cook), og-tests
`tests/physical.rs` (the reference and nine behaviour tests, passing on
metal and in the lavapipe container), fx-showcase `tests/physical.og.ts`.
