# E5 feature costs

Each row is a scene measured twice: once with the feature and once without
it. Render rows come from `og-gfx --costs` (metal: best of 2 sweeps × 5 runs
of 100 frames) and `og-gfx --costs-e5 --frames 20` (vulkan-cpu: lavapipe,
best of 5 runs of 20 frames, the E5 rows only). Both run at 1280×720.
Physics rows come from `cargo run --release -p og-physics --example
soft_costs`, which times one feature alone per 60 Hz tick (best of several
runs over 600 ticks).

Machines: Apple M4 Pro, macOS, Metal, native aarch64. vulkan-cpu is
llvmpipe (LLVM 19.1.7, 256-bit) in the og-lavapipe container (Debian
trixie), x86_64 under Docker's emulation on the same M4 Pro. The vulkan-cpu
and x86_64 physics numbers include that emulation, so they are an upper
bound for a real x86_64 CI runner and are best read relative to each other.
Both machines were shared with another agent's work, so expect about ±10 %.

## Rendering

| Scene: feature | metal ms | Δ metal | vulkan-cpu ms | Δ vulkan-cpu | draws |
|---|---|---|---|---|---|
| `atmosphere-fog`: fog (distance and height) | 0.720 | +0.006 | 50.971 | +0.242 | 25 |
| `atmosphere-dusk`: sky look drawn per pixel (vs the cubemap) | 0.710 | -0.010 | 52.934 | +0.546 | 25 |
| `water-3d`: planar water (refraction, foam, SSR on the full tier) | 0.996 | +0.656 | 116.719 | +75.160 | 21 |
| `water-2d`: 2D water line | 0.117 | +0.001 | 15.358 | +2.707 | 5 |
| `fluid-2d`: 2D fluids as metaballs (2 fluids, 1,500 particles) | 0.160 | +0.044 | 23.428 | +18.824 | 5 |
| `shader-hooks`: material shader hooks (3 dissolving, 900 grass blades) | 0.718 | -0.007 | 87.621 | -3.952 | 31 |
| `weather-rain`: rain preset (particles, fog, dimming) | 0.733 | +0.031 | 58.959 | +6.480 | 26 |

Reading the table:

- Fog, the per-pixel sky and shader hooks are within noise on the GPU. A
  hooked material swaps the identity hooks for its own, so it costs nothing
  measurable over the plain material it replaces.
- Planar water is the expensive feature. It needs the depth prepass, a copy
  of the scene drawn so far, and a reflection march. On Metal that is
  0.66 ms, which fits a 60 Hz budget. On a software rasteriser it doubles
  the frame, so a game that targets low-end devices should keep water small
  in view or use the 2D line.
- Metaballs fill a field target before the main pass. That is cheap on a GPU
  and costs fill rate on lavapipe.
- For reference, the scene with every pre-E5 feature off draws in 1.196 ms
  on metal (`gltf-reference`; the full table is in og-gfx `--costs`).

## Physics (per 60 Hz tick, over an empty world)

| Feature | profile | M4 Pro (aarch64) ms | x86_64 Linux (emulated) ms |
|---|---|---|---|
| og.Fluid2D, 1,000 particles (2 substeps) | 2D | 1.154 | 1.478 |
| og.Fluid2D, 2,000 particles | 2D | 2.979 | 3.731 |
| og.Fluid2D, 4,000 particles | 2D | 6.842 | 8.508 |
| og.WaterSurface 64×64, 8 floating boxes | 3D | 0.012 | 0.020 |
| og.WaterSurface 128×128, 8 floating boxes | 3D | 0.030 | 0.047 |
| og.WaterLine, 96 columns | 2D | 0.002 | 0.004 |
| 10 og.Rope, 16 segments | 3D | 0.028 | 0.034 |
| 4 og.Cloth, 12×12 | 3D | 0.113 | 0.178 |
| 8 og.CharacterController | 3D | 0.046 | 0.064 |
| 8 og.PlatformerController | 2D | 0.011 | 0.015 |
| 4 og.Vehicle (raycast cars) | 3D | 0.007 | 0.010 |

The 2D fluid is the only physics feature that is not small. Use about
1,000 to 2,000 particles in a 16.7 ms tick, depending on the rest of the
game. 3D SPH was not attempted: 3D water is a height field (og.WaterSurface)
with buoyancy and splashes.

## Wasm size

The publish variant (`scripts/check.sh wasm-build wasm-opt size-gate`, `-Oz`
then brotli q11) is listed below. The gate is 4,194,304 bytes; the round
stays under 75 % of it with no feature behind a cargo flag.

| | bytes after `wasm-opt -Oz` | brotli q11 | of the 4 MB budget |
|---|---|---|---|
| before E5 (db38b41) | | 2,518,068 | 60 % |
| E5 physics alone (870c65a) | | 2,621,192 | 62 % |
| E5 (this round) | 9,891,675 | 2,664,999 | 63 % |

The round adds 146,931 bytes brotli: about 103 KB for physics (joints,
controllers, vehicles, Verlet, fluid, water, ragdoll) and about 44 KB for
rendering, the runtime and the atmosphere module. naga's back ends for the
shader cook are in og-cook's native `cook` feature only, as the cook is.
