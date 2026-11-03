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
  --commands "3:screenshot:off;3.1:set:og.GlobalIllumination.enabled=true;4:screenshot:gi;4.1:set:og.GlobalIllumination.intensity=2;5:screenshot:gi-x2;5.1:set:og.GlobalIllumination.allowed=false;6:screenshot:quality-off;7:quit"
```

`off` and `quality-off` should match. `gi` should show warm and coloured
bounce in the creases and under objects, and `gi-x2` stronger bounce.

## Batch 2: realtime light volumes

`og.RealtimeLightVolume` (`updatesPerFrame`, default 2, at most 8;
`intensity`), a lazy component beside an `og.LightVolume`, relights
that box's probes while the game runs (`og-render/src/probe_grid.rs`,
`shaders/probe_grid.wgsl`, `docs/design.md` §2.6). It is off unless a
game adds it, so no existing frame moves, and lazy, so no hash pin moves.

- No binding is added to a lit stage: the realtime probes are written
  into the light volumes' storage buffer (frame binding 10), after the
  baked probes. Their header entry has bit 21 of the packed counts set.
  `ambient_irradiance` blends the realtime volumes over the baked ones
  over the environment, each by its edge weight. Lightmapped surfaces
  keep their lightmaps. The visibility-buffer resolve shades through the
  same `lit_fragment` and `ambient_irradiance`, so it gets the probes too.
- A probe is relit by a 16² cube capture through the reflection-probe
  capture path. The captures are appended to the frame's probe list
  after the sampled reflection probes (`ProbeFrame::sampled`), so they
  are never reflected or used as boxes. A compute pass (`cs_project`,
  one 64-thread workgroup per probe) then projects the six faces onto L1
  SH (irradiance over π, `[c1, c0]` per channel) where the reflection
  probe filter would run. The captures sample the probes, so each sweep
  adds a bounce.
- Up to 8 probes a frame over every volume, in a stride-7919 order that
  spreads them over the box. Volumes take turns being first. A volume is
  sampled once every probe has been relit (its first sweep runs at 8 a
  frame); until then its box takes the baked probes or the environment.
  `updatesPerFrame: 0` relights once and keeps the probes.
- The quality's GI switch (`scene.quality.global_illumination`: the low
  preset, `GlobalIllumination.allowed`, auto quality's low step) stops
  the relighting, and the box falls back to baked probes.
- Fix for the gap the coordinator named: a low preset applied before
  the world names `og.GlobalIllumination` now keeps GI off once the
  world does name it. The preset's switch waits in `FxState::gi_preset`
  (presentation state, not the world) until a later preset writes the
  field.

Checks (orb, no GPU): `cargo test -p og-runtime` (the hash pins, and
`render_quality`'s two new tests: the low preset before GI is named, and
a realtime volume reaching the renderer with its LightVolume's box);
`cargo test -p og-render --lib` (`probe_grid`'s four plan tests,
lightmap, probes); `cargo test -p og-render --test tests shaders` (naga);
`cargo check -p og-render` (native and wasm32); clippy `-D warnings` on
og-render, og-runtime and og-tests; fmt; `og api gen` / `og api check`;
`og check templates/fx-showcase`.

gfx tests (`crates/og-tests/tests/gfx/overlay_effects.rs`):

- `light-volume-room` (new reference, 48 frames, no TAA): a closed room
  6.4 m on a side. The left wall is red, the right green, the rest white,
  with a white block (left) and a sphere (right). One warm point lamp
  with shadows sits under the ceiling near the red wall, with almost no
  environment light (a black background, intensity 0.05). A realtime
  volume fills the room (5 × 3 × 5 probes, 8 a frame, about five
  sweeps). Expected:
  - The faces away from the lamp should be lifted from near black: the
    block's front and right, the sphere's underside, the floor in the
    block's shadow, the ceiling.
  - Reddish light should fall on the left side and greenish on the right
    side (the sphere's right flank, the floor along the green wall).
  - The light should vary smoothly between probes, with no blocky cells
    and no black probe.
  - There should be no light leaking through the walls' outer faces.
- `a_realtime_light_volume_takes_the_lamp_s_colour`: no reference. With
  the volume against without it, the frame gains more than one level a
  channel. With the lamp turned blue, the added light's blue-to-red
  ratio is more than 1.5 times the warm lamp's.

Commands (a native dev build): the fx-showcase baked station (the
lightmapped Cornell box, whose LightVolume lights the rolling ball). The
template gains two commands: `realtime:on|off` adds or removes
og.RealtimeLightVolume on `cornell-probes`, and `lamp:warm|blue` sets the
ceiling spot's colour.

```sh
og run templates/fx-showcase --headless --size 1280x720 --out /tmp/r3-probes \
  --commands "0.5:station:baked;3:screenshot:baked;3.1:lamp:blue;5:screenshot:baked-blue;5.1:realtime:on;8:screenshot:realtime-blue;8.1:lamp:warm;10:screenshot:realtime-warm;10.1:set:og.GlobalIllumination.allowed=false;12:screenshot:quality-off;13:quit"
```

- `baked`: the station as before.
- `baked-blue`: the box's surfaces are lit blue directly, but the ball
  keeps the bake's warm ambient light (the baked probes are stale).
- `realtime-blue`: the ball's shaded side turns bluish, still red on its
  left and green on its right. The box itself does not change, since its
  lightmap comes first.
- `realtime-warm`: close to `baked`, the realtime probes standing in for
  the baked ones.
- `quality-off`: the realtime volume stops and the box falls back to its
  baked probes, close to `baked`.

## Batch 3: reflections from the probes

The realtime light volumes now relight reflections too.
`volume_reflected` (lightmap.rs's probe WGSL) wraps `reflected_light` in
the split-sum specular of `pbr_fs.wgsl` and `physical.wgsl`.

- Inside a realtime volume, the prefiltered reflection (the environment's
  or a ReflectionProbe's) is scaled per channel by the probes' irradiance
  for the normal over the environment irradiance it replaces (clamped to
  16). Toward roughness 1 (smoothstep from 0.4) the reflection blends to
  the probes' harmonics evaluated along the reflected ray. Everything is
  blended in by the volume's edge weight.
- `ambient_irradiance` leaves the trilinearly interpolated realtime
  harmonics, their weight and the environment irradiance in private
  variables, so the reflection costs no second walk over the volumes.
- Outside every realtime volume, and on lightmapped fragments, the
  reflection is unchanged. Baked volumes do not touch reflections, so
  existing frames do not move.
- No binding is added. R2+R8 (who own the IBL path) were told before the
  edit.

Checks (orb, no GPU): naga validates `cargo test -p og-render --test
tests shaders`; `cargo test -p og-render --lib`; `cargo check -p
og-render` passes native and wasm32; clippy, fmt, `og api check`.

gfx tests:

- `light-volume-mirror` (new reference, 48 frames): the batch 2 room
  with a chrome sphere (metallic 1, roughness 0.3) in the middle of the
  frame. Expected: the sphere reflects the room, with the red wall on its
  left, the green wall on its right, and the lamp's warm light above. It
  should not reflect the dark environment, and there should be no seams
  or blotches between probes.
- `a_realtime_light_volume_relights_a_mirror_s_reflection`: no
  reference. The sphere's middle gains brightness with the volume (it
  has no diffuse light, so the gain is the reflection's). Its left half
  gains more red than green and its right half more green than red. On
  batch 2's code the gain is zero.
- `light-volume-room` moves slightly from batch 2 (its matte surfaces'
  reflections are now the probes'). If it was recorded on batch 2's
  head, re-record it.

### Batch 3, second head: macrunner's findings fixed

macrunner held 585fc47cf. Two findings, both confirmed in the code:

1. The mirror reflected the sky, not the room. `volume_reflected`
   scaled the environment cube by an irradiance ratio, and the probes'
   SH along the ray only entered from roughness 0.4 up, so a 0.3
   chrome sphere never got it. It now reflects the probes' L1 radiance
   along `r` (`c0 + 1.5 c1·r`, the band going to 1.0 as roughness
   rises) at every roughness. Only inside a ReflectionProbe's box do
   smooth surfaces keep that capture, scaled by the irradiance ratio.
2. `light-volume-room`'s floor in the block's shadow was navy: it was
   outside the volume. The box (y 0.15–2.75, x/z ±2.7) ends 0.15–0.3
   short of the floor, walls and ceiling, and a volume's weight fades
   to 0 at its faces, so those surfaces took the environment's sky
   ambient. A realtime volume is now whole up to its faces and fades
   out over one spacing beyond them, so the room's surfaces take the
   probes. Baked volumes keep their old fade, so no baked frame moves.

Frames: `light-volume-room` and `light-volume-mirror` should both be
drawn from this head. The floor in the block's shadow should take the
room's warm, red and green light, not navy. The mirror should show the
red wall's colour on its left, green on its right, and the lit ceiling
above, as a blurred image of the room. The mirror test now reads two
bands left and right of the sphere's middle (0.05h–0.12h out), where
the normals face the walls, not the central square.
