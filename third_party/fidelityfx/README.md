# FidelityFX Super Resolution 3.1 upscaler

Source: AMD GPUOpen FidelityFX SDK, https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK,
revision `7539ae8b6568ad29ed78a753404351daf3b5cca7` (SDK 2.4.0, FSR upscaler 3.1.6),
`Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/`. MIT: `LICENSE.txt`
is the notice every one of those headers carries.

The port is `crates/og-render/shaders/fsr_*.wgsl` (one WGSL module per AMD
pass, each prefixed with `fsr_common.wgsl`) driven by
`crates/og-render/src/post/fsr.rs`. It is selected with
`Upscaler::Fsr3` (`render.ogt` `Upscaler`) under TAA and replaces the
engine's TAA resolve and its sharpening pass.

## Kept as AMD wrote it

Prepare inputs (dilated depth and motion, previous-depth reconstruction),
the luma pyramid's exposure and frame info, shading change, prepare
reactivity (dilated reactive, new locks, disocclusion), luma instability,
accumulate (history reprojection with the reference Lanczos-2, the
approximate Lanczos-2 upsample, the variance box rectification, locks,
the reversible tonemap in YCoCg) and RCAS. The jitter's phase count and
Halton sequence are `ffxFsr3UpscalerGetJitterPhaseCount` and
`ffxFsr3UpscalerGetJitterOffset`.

## Deviations, all for core WebGPU (no fp16, no subgroups, no float atomics, no device-wide coherence)

- **fp32 only.** AMD's half-precision path (`FFX_HALF`) is not ported, so
  the module needs no `shader-f16`; the same code runs on Chrome, Safari,
  Firefox and every native backend.
- **No wave operations.** SPD's single dispatch relies on a globally
  coherent mip and an atomic counter for its last workgroup; the luma
  pyramid is two dispatches instead (`cs_reduce`, `cs_finish`), whose
  averages are over the render's pixels exactly. The shading change
  pyramid's three mips the shading change pass reads are written by one
  workgroup per tile in workgroup memory.
- **Previous-depth scatter** uses a `u32` `atomicMax` on the inverted f32
  bits (0 is far) instead of `InterlockedMax` on a typed UAV. The idea of
  scattering through integer atomics on float bits follows pmndrs/upscaler
  (MIT, © 2026 Dennis Smolek, https://github.com/pmndrs/upscaler, head
  `2020ea2`), whose code is not copied.
- **Storage formats** are the core WebGPU ones: AMD's `R16`/`RG16`/`R11G11B10`
  intermediates are `r32float`, `rg32float`, `rgba16float` and `rgba8unorm`.
- **Inputs are the engine's.** Motion is the engine's (camera motion through
  the depth, or the prepass's motion vectors), negated to FSR's convention;
  the reactive mask is `min(1 − OIT revealage, 0.9)`; there is no
  transparency-and-composition mask. The colour is HDR scene-linear. The
  output's alpha (the engine's tone-mapped fraction) is blended along the
  motion and clamped to the current samples' range, as `taa.wgsl` does.
- **Frame time** comes from the tick count, never the wall clock, so the
  upscaler stays deterministic.
- **Loops written out, shared taps in workgroup memory.** naga emits
  every WGSL loop as a guarded `while (true)` that Metal does not unroll,
  so a private array indexed in a loop (a 3x3 of samples, a 4x4 of
  history) went to stack memory. Every fixed-trip loop is instead a run of
  calls with constant offsets, with the same arithmetic in the same order
  except where noted here. The history's Lanczos-2 is separable, with one
  set of four weights per axis made from three sines, where AMD
  evaluates the kernel per tap. The accumulation prepares the current
  colour under a workgroup's 8x8 output pixels once, into an 11x11 tile
  that the nine taps of each pixel read. The shading change pyramid runs
  one thread per level-0 texel (16x16 a group). The output matches the
  looped form to within float reordering: at most 0.006 levels of 255
  on average per channel, with a few pixels up to 29.
- **Luma instability in the reactivity pass.** AMD's luma instability
  is a pass of its own that reads the dilated reactive masks back
  bilinearly at the jittered place; here the reactivity pass keeps the
  luma history and its instability from the pixel's own masks as it
  writes them, a sub-pixel apart, saving the pass and its reads.
- **The history's colour is the engine's previous output.** AMD keeps
  an internal history (colour, lock in alpha) beside the output it
  writes; the engine's output already persists as its TAA history (the
  same colour), so the accumulation writes the output once and keeps only
  the lock, in an `r32float` of its own, read back bilinearly (AMD's
  rides the colour's Lanczos and deringing; either stays in the middle
  2x2's range). The new locks are cleared only where the reactivity pass
  set one. The pass's stores fall from 20 to 12 bytes a pixel.
- **The history read in 8 fetches.** The reference Lanczos-2 over the
  4x4, deringed to the middle 2x2, reads the middle 2x2 as loads (the
  deringing box needs them), each edge's middle pair, whose weights are
  both positive, as one filtered fetch at their ratio, and leaves out the
  four corners (together at most 1/64 of the weight, at a half-texel
  offset on both axes), renormalising the rest.
- **A narrower upsample kernel where the history is whole.** AMD's
  Lanczos-2 upsample reaches one output pixel from each sample, so the
  accumulation converges to the image filtered by that kernel, which
  drops a thin feature's contrast (poles and wires a render pixel wide
  came out lighter than a supersampled frame, 0.8 to 1.7 dB under TAAU
  in `fsr::compare`). The accumulate pass multiplies AMD's kernel bias
  (the factor on each sample's offset) by up to 2 (`KERNEL_NARROWING`:
  the first zero half an output pixel off) and scales the frame's
  weight by the square of that, keeping AMD's convergence rate; the
  narrowing eases back to AMD's kernel as the history weight falls
  (motion, disocclusion, a cut), where few frames are blended and a
  narrow kernel keeps each frame's aliasing. Where the biased kernel's
  radius leaves out the 3x3's outer ring, its weights are not made.
- **Fewer reads.** The reactivity pass loads the luma's 3x3 once for the
  thin-feature locks and the instability's luma at the jittered place
  (bilinear from the same nine), and skips the reactive mask's dilation
  when no OIT drew. The instability is stored in `rgba8unorm` and read
  with one filtered fetch. The accumulation takes the engine's alpha
  from its workgroup tile and the history's middle 2x2 it already loads,
  and reads the new lock once.
- Frame generation, the debug view and the auto-generated reactive and T&C
  masks are not ported.
