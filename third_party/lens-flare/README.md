# lens-flare

The lens flare (`og.LensFlare`; `crates/og-render/shaders/lens_flare.wgsl`,
`crates/og-render/src/post/flare.rs`, the starburst in
`crates/og-render/shaders/post.wgsl`) ports two sources.

## Filament (screen-space ghosts, halo, blur, starburst)

google/filament at revision `5ce4cbf19051729d74db9ba33ad5bd04cf6b4f37`
(2026-11-03), Apache-2.0, Copyright The Android Open Source Project. The
licence is in `filament/LICENSE`. Filament's flare is itself after John Chapman's
"Screen Space Lens Flare" (2017,
https://john-chapman.github.io/2017/11/05/pseudo-lens-flare.html).

The files ported to WGSL:

- `filament/src/materials/flare/flare.mat`: `sampleColor` (chromatic
  aberration along the line to the centre), `ghosts` (taps along that line,
  wrapped with `fract`, each over the threshold and clamped to 1), `halo`
  (the frame warped outwards by a fixed length, kept on an aspect-corrected
  ring), `ring` and `smoothstep01`.
- `filament/src/PostProcessManager.cpp`: `flarePass`, which reads the
  bloom's first level and draws at half its size, and `gaussianBlurPass`'s
  coefficients for kernel width 9 and sigma 10/6: a centre tap and two
  bilinear pairs each side.
- `filament/src/materials/separableGaussianBlur.fs`: the separable blur's
  taps.
- `filament/src/materials/colorGrading/colorGrading.mat`: `starburst` (two
  copies of a random row read by the angle round the centre and turned with
  the camera's forward axis) and the flare's composite before the tone
  curve. Filament fills its starburst row, a 256 × 1 R8 texture, with
  0.5 + 0.5 × uniform random values when it starts. Here the row is
  procedural: the engine's own hash, read with the same linear filtering and
  repeat. No texture or asset is copied.
- `shaders/src/common_math.glsl`: `acosFastPositive`.
- `filament/include/filament/Options.h`: `BloomOptions`' flare defaults
  (4 ghosts, spacing 0.6, threshold 10, halo radius 0.4, thickness 0.1,
  threshold 10, chromatic aberration 0.005, the starburst on, strength
  0.1). These are `LensFlare::default()`.

## Wicked Engine (light occlusion from depth)

turanszkij/WickedEngine at revision `e8be9c7732e992a2365d1041921fd2d6e5479150`
(2026-11-04), MIT, Copyright (c) 2026 Turánszki János. The licence is in
`wicked/LICENSE`.

Ported: `WickedEngine/shaders/lensFlareVS.hlsl`'s occlusion, the share of a
grid of depth-buffer samples around the light's position on screen that lie
at or behind the light. It runs on the GPU with no readback, and the sun
lies at the far plane. Also ported is `wiRenderer.cpp` `DrawLensFlares`'s
skip of a sun below the horizon.

## Changes from the sources

- The grid is 9 × 9 samples over the light's disc on screen, computed once
  per light into a texel of an 8 × 1 texture, not in every flare vertex.
  Samples off the frame are not counted.
- Wicked draws textured sprites along the axis. Here, the sun and the lights
  instead feed the ported Filament ghosts and halo as discs of their
  radiance, each thresholded and clamped on its own and then scaled by its
  visible share. Around each source, the bloom taps are masked out (the sky
  round the sun, a lamp's bulb, and the bloom's spill of them onto an
  occluder's edge), so it flares once.
- A source's ghosts are sprites of a smooth aperture (a flat disc with a
  soft rim): the sun's 8 % of the frame's height in radius, a light's
  three times its disc's radius up to that and at least three flare
  texels, drawn where Filament's taps would mirror the source and
  magnified as they would be, up to 1.5 times. Each holds a share of the
  source's exposed radiance over the threshold (0.56 % for the sun, 0.28 %
  for a light) as the brightness of a ghost of the sun's radius, with the
  energy kept over that radius, so a smaller ghost is as bright and
  carries less light; linear in the source's light,
  neither clamped nor split into channels. A ghost that would lie on its
  own source's glare is left out. Each ghost takes one coating tint (amber,
  teal, violet), the chromatic aberration defaults to 0.001 (Filament:
  0.005) and the halo radius to 0.28 (0.4): art direction.
- A spot only flares from inside its cone. Fog dims a source as media.wgsl
  `fog_amount` does. The sun fades out over the horizon instead of being
  skipped at it.
