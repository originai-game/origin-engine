# Phase 2, animation and physics round: evidence

Branch `p2-anim-physics`, 2026-10-19, Apple M4 Pro (shared with other
agents' builds; load average 18 to 44).

| File | What it shows |
|---|---|
| `stations-four-tiers.png` | fx-showcase's four new stations (retarget, ragdoll2d, cloth, fluids) on metal, vulkan-cpu, webgpu and webgl2, from their committed references |
| `fluid-groups-metal-vulkan-cpu.png` | og-tests' fluid-groups scene (ten fluids, two groups) on metal (top) and lavapipe (bottom) |
| `test-measurements.txt` | what the new tests measured: retargeted feet (height error and slide 0 with foot IK, 11.5 mm off with forward kinematics alone; the cooked strider and stout at leg ratios 1.267 and 0.689), fluid layering and blending, cloth self-collision against none |
| `costs.md` | CPU cost per tick of self-colliding cloth, fluid groups and 2D and 3D ragdolls; the metaballs' GPU cost for ten fluids |
| `cross-target-determinism.md` | `tools/wasm-determinism/run.sh`: host debug and release and wasm32 scalar and simd128 agree on the physics scenario (3D and 2D, with the new cloth, fluid group and 2D ragdoll) and on the walk with its retargeting |
| `break-and-restore.md` | every new test broken once and seen to fail, then restored |
| `danger-tokens/` | og-ui's danger button on the platform's new tokens (cf932f8): before and after crops of every re-recorded reference, and the 96-row contrast table |

Tiers for the new frames against metal (RMSE; no frame has outliers above
0.001 %):

| Frame | vulkan-cpu | webgpu | webgl2 |
|---|---|---|---|
| retarget | 0.00121 | 0.00027 | 0.00011 |
| ragdoll2d | 0.00223 | 0.00005 | 0.00005 |
| cloth | 0.00134 | 0.00025 | 0.00125 |
| fluids | 0.00058 | 0.00020 | 0.00007 |
| og-tests fluid-groups | 0.00097 | (no browser harness, owed row 22) | |

How they were made:

- `metal`: `OG_FX_CAPTURE=<station>.png og test templates/fx-showcase`;
  og-tests' references with `OG_GFX_CAPTURE=fluid-groups`.
- `vulkan-cpu`: the og-lavapipe Docker image (llvmpipe on LLVM 19.1.7)
  running og, og-gfx and the addon cross-built with `cargo zigbuild
  --release --target x86_64-unknown-linux-gnu.2.31`. There fluid-2d and
  water-2d reproduce their committed references at RMSE 0, a second
  fluid-groups run is byte-identical, and all 13 fx-showcase suites pass.
- `webgpu`, `webgl2`: `packages/engine/tests/web/reference.ts [--webgl]
  --record --script 0.5:station:<name> --shots <name>@<tick>` in Chrome for
  Testing 150 with the gate's publish wasm; afterwards `reference-e5.ts`
  passes 12 of 12 stations on both tiers.

Not attempted: 3D SPH (owed row 29), left for the next round.
