# Feature area 1: global illumination and sky occlusion

Branch `feat/gi-area`, from origin/main 9bb81cb41. Before this, every
scene's ambient light was the environment's irradiance cube, flat
wherever it was read: no sky occlusion and no bounce off screen. The
screen-space GI of R3 (`../r3-gi/NOTES.md`) bleeds only what the frame
shows, and gi-room on Metal shows no visible bleed.

## What landed

- **Probe GI (DDGI)**: `og-render/src/gi_probes.rs` and `shaders/gi_*.wgsl`.
  - Ported from Wicked Engine's DDGI (MIT, `third_party/wicked-ddgi`).
    NVIDIA's RTXGI is under the NVIDIA RTX SDK licence, not MIT, so it
    is only cited: the visibility bias follows its paper and
    documentation, and none of its code is used.
  - Cascades of 16 × 8 × 16 probes that scroll with the camera, each
    cascade at twice the previous spacing. Cascade c relights 1/2^c of
    its probes a frame.
  - Each probe stores:
    - L1 harmonics, deringed when they are written.
    - A 12² octahedral map of distance moments. Lit fragments read it
      through a Chebyshev visibility test, with a wrap backface weight
      and a weight crush.
    - A relocation offset and an inside-geometry flag.
  - Radiance is accumulated with Wicked's multiscale mean estimator.
  - Everything lives in storage buffers: the lit pipelines have no
    texture slot left, so the probes share the light volumes' buffer at
    frame binding 10.
- **Software tracing in the Lumen style**: `og-render/src/gi_scene.rs` and
  `shaders/gi_trace.wgsl`.
  - WebGPU has no hardware ray tracing, so rays go through a two-level
    binned-SAH BVH built on the CPU:
    - One bottom level per mesh or terrain. A mesh is simplified by
      vertex clustering to at most 8192 triangles; a terrain is sampled
      at up to 129 heights a side.
    - A top level over the instances.
  - Each hit is shaded with each material's mean albedo and emission,
    the sun and one random shadowed local light, the hemisphere light and
    the probes' previous bounce, for infinite bounces over frames.
  - It runs on the baseline WebGPU path. No limit is requested: only
    storage buffers within the default eight a stage are used.
- **GTAO bent normals (XeGTAO, MIT, `third_party/xegtao`)**: `fxpost.wgsl`
  `gtao_bent`, `fs_gtao_bent`.
  - When the probe GI and GTAO are both on, the AO targets are RGBA16F:
    r holds the AO and gba the world bent normal, blurred and upsampled
    over all four channels. `fx.ao.w` is 2 in that case.
  - The lit shaders read the probes along the bent normal
    (`pbr.wgsl` `ambient_normal`).
  - The screen GI's bounce and the contact shadows' masks use the same
    channels. Either one turns the bent normals off; when that happens
    the probes are read along the surface normal.
- **Reflection occlusion**: `gi_reflected` in `gi_probes.wgsl`. The
  environment's reflection is scaled by how much of the environment's
  light the probes see along the reflected ray. Where a reflection probe
  covers the point it is kept, because its capture already holds what
  stands around it.
- **API**: `GlobalIllumination.method` (`screen` by default, then `probes`
  and `both`), plus `probeSpacing`, `probeCascades` and `probeRays`. The
  defaults are zero values that make sense, so no existing frame moves.

## Radiance Cascades, measured against it

Sannikov's Radiance Cascades split each ray into intervals. Cascade c
holds probes 2^c apart, with each probe's directions multiplied by
4 (2D) or 8 (3D) per cascade, and intervals 4× longer. The saving
assumes that a ray's cost grows with its length, as it does in a distance
field or a screen march.

There is no GPU on the orb, so the measurement is on the CPU, with the
BVH layout the shader traverses (a scratch port of `gi_trace.wgsl`'s
traversal). The scene is gi-room's geometry plus a roof (6544 vec4, 105
KB of BVH), with 20 000 random rays from inside it:

| interval (m) | nodes/ray | triangles/ray | hit |
|---|---|---|---|
| 0.125 | 11.8 | 0.2 | 0.03 |
| 0.25 | 12.2 | 0.5 | 0.07 |
| 0.5 | 13.0 | 0.9 | 0.16 |
| 1 | 14.5 | 1.9 | 0.31 |
| 2 | 16.9 | 3.5 | 0.55 |
| 4 | 19.8 | 5.6 | 0.82 |
| ∞ | 20.7 | 6.2 | 0.92 |

With a BVH, a cascade-0 interval still costs 57% of a full ray (11.8
against 20.7 nodes): the top level is descended whatever the interval.
The short intervals that make RC cheap in a distance field buy little
here.

Per-frame budgets over the same 16 × 8 × 16 m region:

- **DDGI**, the room's settings (0.75 m, 2 cascades, 64 rays, the second
  cascade at half rate): 2048 × 64 + 1024 × 64 = 196 608 rays, about
  4.1 M node visits.
  - Memory: 1216 B of probe data and 1728 B of estimator per probe,
    6.0 MB for two cascades.
  - Lit fragments read 8 probes per cascade.
- **3D world-space RC** with a cascade-0 spacing of 0.25 m, which RC needs
  at least: its probes carry no visibility, so they must be dense enough
  not to leak. Settings: 16 directions and a 0.125 m interval at cascade
  0, then per cascade 4× the directions, 4× the interval, and probes 2×
  apart. Sannikov's penumbra condition in 3D asks for 8× the directions;
  4× is the cheaper, leakier setting, so these figures are a lower bound.
  Probes per cascade: 55 296, 6912, 864, 108, 14.
  - Rays: 884 736 + 442 368 + 221 184 + 110 592 + 55 296 = 1.71 M, 8.7×
    DDGI's.
  - Node visits, at the table's cost per interval: 10.6 + 5.8 + 3.7 +
    2.3 + 1.1 = 23.5 M, 5.8× DDGI's.
  - Radiance storage is 13.7 MB at RGBA16F, and RGBA16F needs no
    capability. Each merge reads 8 probes × 4 directions.
  - It updates every frame where DDGI amortises. Spread over frames
    instead, it would converge no faster than DDGI.
- **Screen-space RC** (2.5D, as in Path of Exile 2) at 1280 × 720:
  - 640 × 360 probes × 4 directions per cascade, the same count at every
    cascade, 6 cascades: 5.5 M depth-buffer march steps.
  - It sees only what the frame shows. The engine's SSGI (R3) already
    covers that, and the screen half of `method: "both"` is that path.
  - RC would give sharper penumbrae for emissive screen content, but not
    the off-screen bounce and sky occlusion this area exists for.

**Decision: Radiance Cascades are not ported.**

- World-space 3D RC costs about 6× DDGI's tracing over the same region,
  and over twice the memory, with no visibility test.
- Screen-space RC duplicates SSGI's reach.
- What would change this:
  - A distance-field scene representation, where interval cost does grow
    with length (a later Lumen-style global SDF).
  - A 2D lighting path. `og-render`'s 2D lights (Light2d) are the natural
    home for flatland RC, and that is a separate feature area.

GPU timings are macrunner's to take. The `gi-probes` pass is a separate
compute pass, so its cost shows on its own in the frame's GPU timer
breakdown.

## Tests that fail on the old code

- `probe_gi_bleeds_each_walls_colour` (og-tests gfx):
  - Against the same room with no GI, the left half gains more red than
    green and the right half more green than red, with a mean lift above
    2 levels.
  - On the old code the scene has no probes: no lift, so it fails.
- `probe_gi_occludes_the_sky_under_a_roof` (og-tests gfx, gi-shelter):
  - The floor under the porch roof must be below 0.8× the brightness of
    the environment-lit floor, while the open ground stays within 10%.
  - The old code lights both alike, so it fails.
- `gi_room_probes_matches_its_reference` and
  `gi_shelter_matches_its_reference`: new frames, recorded by macrunner.
- Unit tests:
  - The BVH matches brute force.
  - Depth bound, clustering and terrain.
  - The octahedral fold equals Wicked's border table.
  - The scrolling grid's slots and keys.
  - The cascades' update slices.
  - The plan gives the AO targets GI_FORMAT with bent normals.

## Checks on the orb (no GPU)

- `cargo check -p og-render`, native and wasm32-unknown-unknown.
- `cargo check -p og-runtime` and `cargo check -p og-tests --tests`.
- `cargo test -p og-render --lib` and `cargo test -p og-render --test
  tests shaders` (naga: WGSL, MSL, SPIR-V, HLSL).
- `cargo test -p og-runtime`.
- clippy `-D warnings` and fmt.
- `og api check`.
