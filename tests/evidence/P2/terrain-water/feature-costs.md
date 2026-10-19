# Phase 2 terrain and water: feature costs

Each row is a scene measured twice, with the feature and without it, by
`og-gfx --costs-p2` (crates/og-tests/src/bin/og-gfx.rs): best of 5 runs
of 100 frames on metal (release build) and of 10 frames on vulkan-cpu, at
1280×720. A frame's time is the wall time of rendering it and waiting for
the GPU, divided over the run.

Machines: Apple M4 Pro, macOS, Metal, native aarch64, release build.
vulkan-cpu is llvmpipe (LLVM 19.1.7, 256-bit) in the og-lavapipe container,
x86_64 under Docker's emulation on the same M4 Pro, og-gfx cross-built with
`cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31`. The
machine was shared with four other agents' builds and tests throughout
(load averages 18 to 55), so expect ±0.3 ms on metal and ±40 ms on
lavapipe; two metal runs are given to show the spread.

| Scene: feature | metal ms (run 1 / run 2) | Δ metal | vulkan-cpu ms | Δ vulkan-cpu | draws |
|---|---|---|---|---|---|
| `terrain-near`: terrain from 1.7 m (2 km, 4 layers, clipmap) | 0.936 / 1.460 | +0.152 / +0.311 | 284.7 | +160.3 | 38 |
| `terrain-mid`: terrain from 120 m | 1.758 / 1.666 | +0.270 / +0.080 | 326.3 | +170.5 | 36 |
| `terrain-far`: terrain from 1.5 km | 2.190 / 2.078 | +0.543 / +0.508 | 262.8 | +105.4 | 24 |
| `terrain-mid`: the terrain casting shadows (its shadow passes) | 1.756 / 1.678 | +0.218 / +0.347 | 263.9 | +45.4 | 36 |
| `water-planar`: planar reflection at half resolution (vs the screen-space march it replaces) | 1.736 / 1.576 | -0.138 / -0.177 | 273.1 | +1.2 | 49 |
| `water-shadows`: shadows received on water | 2.582 / 2.572 | -0.060 / +0.086 | 471.1 | +46.7 | 44 |
| `water-underwater`: caustics on the bed | 1.333 / 1.299 | +0.015 / +0.135 | 183.7 | -32.7 | 37 |
| `water-underwater`: the underwater view (fog, the surface from below; vs no water) | 1.312 / 1.303 | +0.361 / +0.443 | 228.6 | +12.1 | 37 |
| `terrain-grass`: grass scatter (36,000 five-blade tufts, shadowed) | 5.808 / 5.504 | +4.523 / +4.056 | 474.1 | +189.9 | 43 |

Reading the table:

- The terrain costs 0.1 to 0.55 ms on the M4 Pro, most from afar, where
  the coarse levels cover the whole island and every layer shows. Its
  shadow passes are about a fifth of a millisecond.
- The planar reflection is cheaper than what it replaces: with it the
  water samples the mirrored scene instead of marching the depth prepass
  for screen-space reflections, and the mirrored pass at half resolution
  costs less than that march. On WebGL2, which has no march, it is an
  extra scene pass.
- Shadows on water and caustics are within the noise on the GPU (one
  shadow lookup per water pixel, two cell-noise patterns per lit pixel
  below the surface).
- The grass is the expensive feature, and it is CPU time: `og-gfx
  terrain-grass --frames 300` reports 4.27 ms of CPU submit in a 4.28 ms
  frame, the 36,000 instance transforms rebuilt and uploaded every frame
  and drawn in the three shadow cascades too. The og-game skill tells
  games to keep `density × distance²` modest.
- On lavapipe everything is fill rate: the terrain's per-pixel layer
  blending doubles a frame, and so does the grass.
