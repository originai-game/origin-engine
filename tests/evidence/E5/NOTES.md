# E5 evidence: atmosphere, water, material shaders, physics beyond rigid bodies (2026-10-19)

Branch `e5`, db38b41..HEAD. The full gate (`scripts/check.sh`) passed at bcbae90 in
132 s (path-aware, after `scripts/check.sh quick` passed on the same tree):
fmt 0 s, clippy 1 s, wasm-build 6 s, test 33 s (390 tests, 5 ignored perf
tests skipped), wasm-opt 1 s and size-gate 0 s (reusing the gate's wasm,
built from scratch in 30 s and 44 s earlier the same morning), drift
skipped (passed in the quick run 23 s before), bun-test 46 s and
determinism 45 s. Outside the gate: `scripts/check.sh --all web-reference`
passed in 34 s (5 + 8 stations on WebGPU and WebGL2). The fx-showcase
playtests also passed on lavapipe (`fx-showcase-vulkan-cpu.txt`).

| File | What it shows |
|---|---|
| `og-tests-metal.png`, `og-tests-vulkan-cpu.png` | the nine new og-tests reference scenes on each native tier: atmosphere-fog, atmosphere-dusk, atmosphere-night, water-3d, water-2d, fluid-2d, shader-hooks, weather-rain, weather-snow |
| `fx-showcase-metal.png`, `fx-showcase-vulkan-cpu.png`, `fx-showcase-webgpu.png`, `fx-showcase-webgl2.png` | the eight new fx-showcase stations on the four tiers: sky, weather, water, shaders, controller, joints, rope, fluid |
| `fx-showcase-vulkan-cpu.txt` | the template's nine playtest suites on lavapipe (og-lavapipe container, x86_64 Linux addon). The eight E5 stations reproduce their recorded references at RMSE 0, and the five earlier stations stay within RMSE 0.0039 |
| `feature-costs.md` | render costs of each feature on metal and lavapipe, physics costs per tick on the M4 Pro and on x86_64 Linux, and the wasm size |
| `determinism.md` | the 3D and 2D worlds with every physics feature: hashes at ticks 0, 600 and 1200 and after a restore. They are identical on aarch64 macOS and x86_64 Linux |
| `break-and-restore.md` | every new test broken once and seen to fail, then restored |

Tiers against `metal` for the E5 stations (RMSE, outliers above 32/255):

| Station | vulkan-cpu | webgpu | webgl2 |
|---|---|---|---|
| sky | 0.0011 | 0.0004 | 0.0027 (0.01 %) |
| weather | 0.0012 | 0.0005 | 0.0002 |
| water | 0.0021 | 0.0004 | 0.0216 (0.93 %) |
| shaders | 0.0021 | 0.0010 | 0.0010 |
| controller | 0.0012 | 0.0004 | 0.0002 |
| joints | 0.0012 | 0.0003 | 0.0002 |
| rope | 0.0014 | 0.0004 | 0.0002 |
| fluid | 0.0008 | 0.0002 | 0.0001 |

WebGL2 water reflects the prefiltered sky cubemap instead of marching the
depth prepass, so it is the one frame that differs noticeably. Each tier
compares against its own references, and every tier reproduces them
exactly.

How the tiers were made:

- `metal`: `OG_FX_CAPTURE=1 og test templates/fx-showcase` and og-tests'
  `OG_GFX_CAPTURE` on the M4 Pro.
- `vulkan-cpu`: the same commands in the og-lavapipe Docker image (Debian
  trixie, Mesa llvmpipe on LLVM 19.1.7). It runs og-gfx and the addon
  cross-built with `cargo zigbuild --target
  x86_64-unknown-linux-gnu.2.31`, and it reproduces the committed vulkan-cpu
  references of earlier rounds at RMSE 0.
- `webgpu` and `webgl2`: `packages/engine/tests/web/reference-e5.ts
  [--webgl] --record` in Chrome for Testing.

Not attempted or owed (`docs/phase-1.md` §Status 2026-10-19, E5 round):

- 3D SPH: 3D water is a height field.
- Ragdoll bodies hold the rest pose at weight 0, because the runtime does
  not feed them animation targets.
- Hooks skip skinned meshes, and shadows and the prepass are not displaced
  by the hooks.
- Water has screen-space reflection only, no shadows and no underwater
  view.
- Fluids: at most four drawn per frame, and they do not interact.
- No cloth self-collision and no 2D ragdolls.
- No moon disc.
- The runner and topdown archetypes do not use the new controllers yet.
