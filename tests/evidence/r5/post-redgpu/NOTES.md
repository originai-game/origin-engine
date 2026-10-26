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
