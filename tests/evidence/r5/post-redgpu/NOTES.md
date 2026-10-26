# r5/post-redgpu

RedGPU's (github.com/redcamel/RedGPU, MIT) post effects and wind, ported
as ideas into og-render, in the order agreed in thread
T-e2d98462-ee03-480f-be7f-60fb037b5898: the colour adjustments, the
Khronos PBR Neutral tone curve, lens distortion, wind sway and the screen
blurs. Nothing of RedGPU is a dependency: each effect is og-render data,
an IDL field and a reference frame, on both tiers. Base `6e36ee62`.
Frames are lavapipe's (llvmpipe, LLVM 15.0.6, Mesa 25.0.7) in an orb; the
metal references are owed to the Mac lane (`OG_GFX_CAPTURE=<scene>`).

## 1. Colour adjustments (`PostEffects.temperature` … `threshold`)

- **What landed.** og-render `ColorAdjust` (`PostEffects::color_adjust`):
  white balance (temperature, tint), brightness and contrast, colour
  balance per tonal range with Preserve Luminosity, hue, saturation,
  vibrance, greyscale, invert, threshold, applied in that order to the
  display colour after the tone curve and the grading LUT. No shader
  changed: the renderer bakes the grading table (at its intensity, mixed
  in linear light as post.wgsl mixes it) and the adjustments into one 32³
  table on the CPU (`ColorLut::adjusted`) when either changes, written in
  place into one texture, so the tonemap's bindings stay; the tonemap
  then samples it at intensity 1. Held still, they cost nothing per
  frame; a frame that changes them pays the bake, measured in a release
  build on one core of the orb's 2.6 GHz Xeon at 0.4 ms with one
  adjustment on, 1.7 ms for the white balance alone, 3.4 ms with six on
  and 4.3 ms with a grading table too (the first cut took 6.4 ms and 8.0
  ms: sRGB curves as tables, the white balance and hue matrices made once
  per bake, the grading table read at its texels). The IDL's PostEffects gains the fields (every one off at 0,
  `preserveLuminosity` true); og-runtime builds `ColorAdjust` from them.
- **Decisions.** RedGPU's maths is kept where it is right (contrast's
  `1 / (1 - c)`, the colour balance's shadow/mid/highlight weights and
  luminance ratio, the hue rotation about the grey axis, vibrance's
  logistic saturation guard and skin-tone guard). Its colour temperature
  is not: it drops Tanner Helland's `/ 100` on the kelvin, so green and
  blue clamp to 1 below 6600 K and red falls far too fast above it; the
  white balance is Unity's (`ColorBalanceToLMSCoeffs`, a von Kries scale
  in LMS from a white moved along the daylight locus), the API's
  temperature and tint being Unity's divided by 100. Saturation is one
  linear scale of the chroma (`1 + s`), not RedGPU's two curves
  (`1 / (1.001 - s)` above 0). The LUT runs before the adjustments, so
  `saturation -1` is grey whatever the LUT.
- **Tests.** og-tests `effects::colour_adjustments_bake_after_the_lut`:
  saturation -1 greys the red bar at its luma (231, 89, 63 → 117, 117,
  117); hue 120 turns it green (63, 231, 89), and after a red/blue swap
  LUT the same turn gives red (231, 63, 89), so the LUT comes first and
  both are in one table; a renderer whose scene changes its adjustments
  between frames draws what a fresh renderer draws (the in-place bake);
  warm and cool white balance and tint move a mid grey the right way
  (warm 0.550, 0.497, 0.405; cool 0.380, 0.519, 0.705; tint +0.5 0.549,
  0.479, 0.568). New gfx scene `post-adjust`.
- `crates/og-tests/references/vulkan-cpu/post-adjust.png` —
  `OG_GFX_CAPTURE=post-adjust cargo test -p og-tests --test tests
  gfx::post_adjust` (sha256 aca1e950d6a1fcce). Read: the spike's robots
  and crates on the checker floor through the teal-orange LUT at 0.6,
  then `temperature 0.06, contrast 0.15, vibrance 0.25` and shadows
  pushed to teal: against `gltf-reference` the purple robots turn a
  deeper blue-violet, the far floor and the sky's haze go from grey to a
  warm sand, the near floor stays neutral and the crates' yellow is
  richer; nothing clips.
- `color-adjust-run.png` — `og run templates/fx-showcase --size 1280x720
  --commands "0.5:station:materials;2.0:screenshot:materials.png;
  2.05:set:og.PostEffects.temperature=0.3;2.05:set:og.PostEffects.vibrance=0.6;
  2.05:set:og.PostEffects.contrast=0.25;2.05:set:og.PostEffects.balanceShadows=-0.06,0,0.1;
  2.2:screenshot:materials-adjust.png;… 2.25:set:og.PostEffects.hue=120;
  2.25:set:og.PostEffects.saturation=0.5;2.4:screenshot:materials-hue.png;…
  2.45:set:og.PostEffects.threshold=0.5;2.6:screenshot:materials-threshold.png;2.65:quit"`
  (each step resets the previous fields to 0), the four frames at half
  size (rev 6e36ee62+dirty, 2026-10-26, sha256 5e66db39077093a0). Read:
  top left the materials board as it is (blue-grey backdrop); top right
  warmer, more contrast and vibrance (the backdrop a warm grey, the gold
  row richer, the reds deeper, teal in the shadows); bottom left hue 120
  and saturation 0.5 (the blue row magenta, the reds green, the gold
  row teal); bottom right threshold 0.5, the board as black and white
  silhouettes. The HUD and the og.ui badge are untouched in all four:
  the overlay draws after the post stack.

## 2. Khronos PBR Neutral (`RenderSettings.toneMapping = "neutral"`)

- **What landed.** og-render `Tonemap::Neutral`, code 3 in the tonemap's
  `mode.x` and the views' resolve `params.y`; `post.wgsl` and
  `views.wgsl` gain `neutral()`, the Khronos curve (RedGPU's
  `khronosPbrNeutral`, three.js's `NeutralToneMapping`): the 0.04 of a
  dielectric's reflection comes off (`x - 6.25x²` under 0.08), a colour
  whose peak is then under 0.76 passes through, a brighter one is scaled
  so its peak rolls off as `1 - 0.24² / (peak - 0.52)` and mixed towards
  white by `1 - 1 / (0.15 (peak - new peak) + 1)`, then clamped. The
  IDL's `ToneMapping` gains `neutral` after `none`, so the existing
  indices (aces 0, agx 1, none 2) and every saved value keep their
  meaning; og-runtime maps index 3 and the name; a look's `toneMapping
  neutral` is checked against the same enum by the cook.
  `og migrate three` and the three-to-og guide map three.js's
  `NeutralToneMapping` to it.
- **Tests.** og-tests `effects::neutral_tone_curve_follows_khronos`: the
  colour bars and an over-bright bar (4, 2, 1) against the curve computed
  in the test from Khronos's definition, through the frame's tonemap and
  through a view's resolve over the whole frame (0.8, 0.1, 0.05 → 227,
  73, 34 against 227, 72, 34; 0.5 grey → 181; 4, 2, 1 → 253, 209, 182 on
  both paths). With the views' branch broken (`> 3.5`) the view half
  fails (204, 117, 94: AgX), so the test reads that path. New gfx scene
  `post-neutral`.
- `crates/og-tests/references/vulkan-cpu/post-neutral.png` —
  `OG_GFX_CAPTURE=post-neutral cargo test -p og-tests --test tests
  gfx::post_neutral` (sha256 a02dbc5b75baf944). Read beside metal's
  `bloom` (ACES) and `post-agx`: the orange sphere stays orange-red where
  ACES turns it yellow-white, the blue stays blue (ACES: cyan-white), the
  green and magenta keep their hue into vivid halos, the white sphere's
  core rolls to white, and the backdrop is a deeper navy (the curve's
  offset darkens the near-blacks); no banding in the halos.
- `neutral-run.png` — `og run templates/fx-showcase --size 1280x720
  --commands "0.5:station:materials;2.0:screenshot:materials-aces.png;
  2.05:set:og.RenderSettings.toneMapping=neutral;2.2:screenshot:materials-neutral.png;
  2.25:station:lights;4.0:screenshot:lights-neutral.png;
  4.05:set:og.RenderSettings.toneMapping=aces;4.2:screenshot:lights-aces.png;4.25:quit"`,
  ACES left and Neutral right at half size (rev 6ba3ab70+dirty,
  2026-10-26, sha256 c2998557a565ec1e). Read: on the materials board the
  backdrop goes from ACES's washed pale grey to a lavender blue, the
  emissive sphere (1, 0.45, 0.1 at intensity 3) stays orange where ACES
  skews it to pale yellow,
  the blue and red spheres and the gold row are deeper; under the night
  lights the floor's pools are more saturated with smaller white
  centres. The HUD is the same in both.

## 3. Lens distortion (`PostEffects.lensDistortion`, `lensDistortionCenter`)

- **What landed.** og-render `LensDistortion { amount, center }`
  (`PostEffects::lens_distortion`) and a `lens` pass in the chain after
  the tonemap (`FramePass::Lens`, `stylise.wgsl`'s `fs_lens`, its
  parameters in the chain's uniform): each pixel samples the image at
  `center + offset × (1 + amount r²) × fit`, bilinear (nearest under a
  pixel grid), black beyond the image. The IDL's PostEffects gains the
  two fields; og-runtime builds the struct when the amount is not 0.
- **Decisions.** The plan's "one pass in fxpost.wgsl" (before the
  tonemap) became a chain pass after `paint` and `lines`, the two passes
  that read the depth-normal prepass at each pixel, so ink, hatching and
  halftone bend with the image (before them, in fxpost or the tonemap,
  they would be drawn from the unbent depth over a bent image), and
  before `quantize`, so a palette stays exact after the bilinear
  resampling; the overlay draws after the chain and is not bent.
  RedGPU measures the radius in uv units, an ellipse on a 16:9 frame;
  here it is in pixels, 1 at a centred lens's corners. RedGPU leaves
  black corners under a barrel; here the scale `1 / (1 + amount R²)`, R
  the farthest corner's radius, keeps that corner on the image, so a
  barrel shows no black edge wherever its centre is (a pincushion only
  pulls the image in). RedGPU's centre is an offset in CSS pixels times
  the device pixel ratio; here it is a fraction of the frame.
- **Tests.** og-tests `effects::lens_distortion_bends_by_its_formula`:
  two white markers on a dark 320×180 frame, one on each axis, land
  where the formula, solved in the test by bisection along the ray from
  the centre, puts them, within 0.15 px: barrel 0.5 moves the left one
  from x 88 to 64.7 and the upper one from y 45 to 26.3, pincushion -0.5
  to 80.5 and 43.5; with the centre on the upper marker it stays put and
  the other goes to (61.2, 106.8) by the fit to the farthest corner, and
  no corner is black. The lens is one graph pass, none at amount 0.
  Measured against wrong implementations: the radius in uv units
  (RedGPU's) puts the left marker at 61.4 (fails), the fit to a centred
  lens only leaves two corners black with the centre moved (fails).
  New gfx scene `post-lens`.
- `crates/og-tests/references/vulkan-cpu/post-lens.png` —
  `OG_GFX_CAPTURE=post-lens cargo test -p og-tests --test tests
  gfx::post_lens` (sha256 690be64227ffe9e8). Read beside metal's
  `gltf-reference`: the robots and drones magnified at the centre, the
  floor's far edge and the horizon bowed upwards into arcs, the floor's
  side edges curved, the corners still showing floor and sky (fitted,
  no black).
- `lens-run.png` — `og run templates/fx-showcase --size 1280x720
  --commands "0.5:station:materials;2.0:screenshot:plain.png;
  2.05:set:og.PostEffects.lensDistortion=0.4;2.2:screenshot:barrel.png;
  2.25:set:og.PostEffects.lensDistortion=-0.4;2.4:screenshot:pincushion.png;
  2.45:set:og.PostEffects.lensDistortion=0.4;2.45:set:og.PostEffects.lensDistortionCenter=0.3,0.6;
  2.6:screenshot:barrel-off-centre.png;2.65:quit"`, the four frames at
  half size (rev e75d9d4c+dirty, 2026-10-26, sha256 204296f94f27088f).
  Read: top left the board as it is; top right barrel 0.4, the board
  magnified with its rows bulging and the backdrop reaching the corners;
  bottom left pincushion -0.4, the end spheres stretched outwards and the
  rows bowed in at their ends; bottom right barrel 0.4 centred at (0.3,
  0.6), the lower left spheres swollen and the right side squeezed, no
  black edge. The HUD and the og.ui badge are not bent in any.

## 4. Wind sway (`og.Wind`, `TerrainScatter.sway`)

- **What landed.** og-render `Wind { strength, direction, speed, gust }`
  (`RenderScene::wind`) and `TerrainScatter::sway`; `scatter_sway` in
  `scatter_common.wgsl` moves each vertex of a scattered copy in the
  vertex stage of the lit, prepass and shadow passes, before a material
  shader's hooks. The IDL gains the atmosphere resource `Wind` (off at
  strength 0; direction 1 0 0, speed 3 m/s, gust 0.6) and
  `TerrainScatter.sway` (1; 0 for stones); og-runtime reads both.
- **Decisions.** RedGPU sways a copy about its upright stem with a wave
  that travels upwind; here the mean is a lean downwind (`strength` m of
  tip per metre of height) varied by `1 + gust × w`, `w` two octaves
  (wavenumber 2π/10 m and 2.3 times it) travelling downwind at `speed`,
  offset per copy by a hash of its position, so gusts sweep across the
  field and neighbours differ. A vertex moves by `lean × h × min(h / top,
  1)` (the root still, the tip furthest) and drops to keep its distance
  from the root. The phases are reduced in f64 on the CPU from the tick
  (no f32 time drift in a long session) and the previous frame's are kept
  for the motion vectors; a chunk's culling bounds grow by the furthest
  tip. Nothing in the simulation reads it (og.Rope, og.Cloth and
  og.Weather keep their own wind).
- **Tests.** og-tests `terrain::wind_sways_scattered_copies_downwind`:
  30 plates held 2 m up on stems, seen from above. In a steady wind
  (strength 0.5, direction 3 5 4) each moves (0.6, 0.8) m within 0.026 m
  at ticks 0 and 37 (the horizontal direction normalised, the lean times
  the height); strength 0 and sway 0 draw pixel-identical frames to no
  wind; in gusts (gust 1) every plate moves downwind by 0 to 2 m, 24 of
  30 move on by tick 20 and they spread over 1.86 m. With motion blur and
  a still camera a frame drawn after tick 0 smears the plates (3051
  pixels differ from a renderer's first frame of tick 30); with the
  previous position drawn from this frame's phases it is 0 and fails. A
  first cut of that check passed nothing: plates at emissive 8 with no
  tone curve stay at 255 red however blurred, and the blur's 20 px
  dilation taps step over a 7 px plate; it now dims them to 0.8 and
  blurs over 16 px. The earlier scatter tests (`terrain::scatter*`, the
  fork-field scatter tests, `light::terrain_and_scatter`) pass, and
  `terrain-grass` still matches its reference.
- `crates/og-tests/references/vulkan-cpu/terrain-grass-wind.png` —
  `OG_GFX_CAPTURE=terrain-grass-wind cargo test -p og-tests --test tests
  terrain::the_grass_scene_in_the_wind` (sha256 382cc260dcc38283). Read
  beside metal's `terrain-grass`: the same meadow and boulders, every
  tuft bent towards the right and a little towards the camera, their
  tips lower and the blades longer on screen, with bands where a gust
  leans them further; the boulders (plain instances) and the mountains
  unchanged.
- `wind-run.png` — no template scatters over a terrain, so the run is on
  a scratch copy of fx-showcase (not committed) whose P2 terrain station
  grows tufts of three crossed blades (`og.TerrainScatter` density 6
  within 40 m on the grass layer) with the camera 2.6 m over the meadow:
  `og run /tmp/fxs-wind --size 1280x720 --commands
  "0.5:station:terrain;3.0:screenshot:calm.png;3.05:set:og.Wind.strength=0.45;
  3.05:set:og.Wind.gust=0.8;3.05:set:og.Wind.direction=1,0,-0.3;
  3.5:screenshot:wind-a.png;3.75:screenshot:wind-b.png;4.0:screenshot:wind-c.png;4.05:quit"`,
  the four frames at half size (rev 950fd568+dirty, 2026-10-26, sha256
  f025f4e62c96d14c). Read: top left calm, the blades upright; the other
  three in the wind, the blades leaning away and to the right, the
  near ones lying lower, by different amounts across the field and from
  frame to frame as the gusts pass. The HUD and the og.ui badge are
  unchanged.
