# R3: dynamic global illumination

## Batch 1: screen-space GI

`og.GlobalIllumination` (`enabled`, `intensity`, `maxDistance`,
`thickness`, `steps`, `rays`, `occlusion`, `resolution`, and `allowed`,
which `og.render.applyQuality` writes, false in the low preset) draws in
the AO passes' place
(`og-render/src/ssgi.rs`, `shaders/ssgi.wgsl`, `docs/design.md` §2.6).
Off by default: no existing frame moves. It is a lazy resource
(og-runtime `lazy.rs`): a world that never names it keeps its type table
and simulation hash, so no hash pin moves (the first head put the fields
on PostEffects and RenderSettings and moved five og-runtime pins).

- No lit binding added. The AO targets become RGBA16F while GI is on:
  r holds the occlusion, gba the bounce. The lit passes read them through
  the effect group's existing `ssao_tex`, and `Fx.ao.w` says the gba is
  there.
- Each pixel casts cosine-weighted rays from its prepass normal through
  the view depth. A hit on a surface facing the ray reads the previous
  frame's lit image at the hit's reprojected uv (prepass velocity). Under
  TAA that image is TAA's history; without TAA the main pass copies `hdr`
  into a persistent ping-pong (`Persistent::gi`). That gives one bounce a
  frame.
- Under TAA the noise moves every frame (IGN offset by 5.588238 px a
  frame) and TAA's accumulation is the denoiser. Without TAA the pattern
  is fixed and the 4×4 blur, weighted by depth and normals, smooths it.
  Half resolution reuses the AO's depth-aware upsample over four channels.
- Occlusion: the rays' hit share (`giOcclusion`), or the scene's own GTAO
  or SSAO term when `PostEffects.ao` is on as well.
- The bounce adds `kd · albedo · bounce` (pbr_fs, physical). Scattered
  copies keep the AO alone.

Checks (orb, no GPU): `cargo test -p og-runtime` (the hash pins), `cargo check -p og-render` (native and
wasm32-unknown-unknown), `cargo check -p og-runtime`, `cargo clippy -p
og-render -p og-runtime -p og-tests --all-targets -D warnings`, fmt,
`cargo test -p og-render --lib` (ssgi, post::plan, quality), `cargo test -p
og-render --test tests shaders` (naga: WGSL, MSL, SPIR-V, HLSL), `og api
gen`.

gfx tests (`crates/og-tests/tests/gfx/overlay_effects.rs`, new frames):

- `gi-room` (no TAA, 4 frames): an open-topped room under a high sun from
  the front right. It has a red wall on the left, a green wall on the
  right, a white back wall and floor, a white block (left) and a sphere
  (right), under a dim sky. Expected: a red tint on the floor along the
  red wall and on the block's left face, and a green tint on the floor
  near the green wall and on the sphere's right side. The shaded faces
  (the block's front and the green wall's inside) should be lifted by the
  sunlit floor's bounce. Contact corners should be darker than the open
  floor. There should be no fireflies and no halo at the walls'
  silhouettes against the sky.
- `gi-room-taa` (TAA, 32 frames): the same scene, smoother, with no
  visible noise pattern.
- `gi_bleeds_each_walls_colour_towards_its_side`: no reference. Against
  the same room without GI, the frame's left half gains more red than
  green and its right half more green than red.

Commands (a native dev build), fx-showcase with GI off, on, doubled, then
switched off by the quality setting:

```sh
og run templates/fx-showcase --headless --size 1280x720 --out /tmp/r3-gi \
  --commands "3:screenshot:off;3.1:set:GlobalIllumination.enabled=true;4:screenshot:gi;4.1:set:GlobalIllumination.intensity=2;5:screenshot:gi-x2;5.1:set:GlobalIllumination.allowed=false;6:screenshot:quality-off;7:quit"
```

`off` and `quality-off` should match. `gi` should show warm and coloured
bounce in the creases and under objects, and `gi-x2` stronger bounce.
