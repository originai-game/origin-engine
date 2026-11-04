# Temporal upscalers: TAAU and FSR 3.1

Under TAA the engine resolves the jittered frame, drawn at the render
scale, to the output with one of two upscalers: `RenderSettings.upscaler`
(`og-render`'s `RenderQuality::upscaler`), `taau` or `fsr3`.
`RenderCapabilities.upscalers` names the ones a device draws: both on every
host and tier. This page is the evidence behind the choice of default and
of the presets (Wave B, 2026-11-05).

## The two

- **TAAU** (`taa.wgsl`, R2): one pass. The sample nearest each output
  pixel, weighted by its distance, blended into an output-sized history
  that is clamped to the current samples' neighbourhood and resampled
  with the bilinear or Catmull-Rom filter (`RenderOptions.taaHistory`),
  then `taaSharpen`'s pass.
- **FSR 3.1** (`fsr_*.wgsl`, `post/fsr.rs`): AMD FidelityFX Super
  Resolution 3.1.6's upscaler, ported to WGSL (MIT,
  `third_party/fidelityfx/`, whose README lists every deviation): eight
  compute passes for dilated depth and motion with depth-reconstructed
  disocclusion, a luma pyramid and auto exposure, shading-change
  detection, reactivity and thin-feature locks, luma instability, and
  the accumulation (Lanczos-2 upsample and history, variance-box
  rectification in a reversibly tonemapped YCoCg), then RCAS
  (`taaSharpen`'s strength). Transparent surfaces feed its reactive mask
  through the order-independent transparency's coverage.

FSR runs in fp32 without subgroups, float atomics or device-wide
coherence: it needs neither `shader-f16` nor `subgroups`, which Safari
and Firefox lack (Chrome has both), so one shader serves every host. The
cost of that choice is AMD's fp16 speed-up on GPUs that would have it.

## The web's options ("pmndrs web")

Surveyed 2026-11-04 (sources cloned and read):

- pmndrs/postprocessing has no TAA or upscaler (only SMAA; TAA is an open
  issue).
- **pmndrs/upscaler** (MIT, v0.5.0) is itself a simplified WGSL
  derivative of FSR 3.1.5: fewer passes (one fused shading-change
  dispatch, single-workgroup exposure), its own measurement 0.63 ms
  against 1.10 ms for the full graph at ratio 2, 1080p, Apple. As a cut-down
  FSR 3.1 it cannot beat FSR 3.1 on quality, so it is not ported; its
  WebGPU answers (integer atomics on float bits for the depth scatter)
  informed the port.
- three.js `TAAUNode` is a single-pass TAAU of the same class as ours.
- Arm ASR is FSR 2.2 with fp16; Snapdragon GSR 2 (BSD-3) is a two-pass
  clip-and-accumulate, a candidate only for a low-end phone tier in the
  compatibility phase.

None beats both TAAU and FSR 3.1, so none is ported.

## Quality

`fsr::compare` (crates/og-tests/tests/fsr.rs, ignored, run by hand on
Metal): 640x360 output, each scene against its own ground truth drawn at
4 samples a pixel (render scale 2, no antialiasing) through the same
frames. `mad` is the mean absolute byte difference of the last frame
(lower is better), `psnr` its PSNR in dB, `crop` the mean difference of
the crop judged at 4x, `flicker` the frame-to-frame change over the last
five frames that the ground truth does not have (crawl, shimmer and
smear; lower is better).

| scene | scale | TAAU mad / psnr / crop / flicker | FSR 3.1 mad / psnr / crop / flicker |
|---|---|---|---|
| thin-static | 0.5 | 2.219 / 30.44 / 3.431 / 0.141 | 2.631 / 29.14 / 4.700 / 0.055 |
| thin-static | 0.67 | 1.762 / 31.18 / 2.667 / 0.187 | 1.960 / 30.33 / 3.679 / 0.063 |
| thin-static | 0.77 | 1.490 / 32.17 / 2.353 / 0.205 | 1.720 / 30.45 / 3.656 / 0.075 |
| thin-pan-disocclusion | 0.5 | 5.393 / 22.30 / 10.741 / 3.977 | 3.350 / 26.16 / 6.195 / 3.143 |
| thin-pan-disocclusion | 0.67 | 4.257 / 23.39 / 8.896 / 3.760 | 2.647 / 27.00 / 5.696 / 2.815 |
| thin-pan-disocclusion | 0.77 | 3.787 / 23.94 / 8.083 / 3.604 | 2.281 / 27.61 / 5.368 / 2.604 |
| grass-pan | 0.5 | 4.583 / 26.41 / 1.740 / 6.093 | 4.663 / 26.23 / 1.805 / 6.168 |
| grass-pan | 0.67 | 4.086 / 27.29 / 1.412 / 6.038 | 4.184 / 27.06 / 1.496 / 6.073 |
| grass-pan | 0.77 | 3.878 / 27.68 / 1.264 / 5.996 | 3.993 / 27.38 / 1.354 / 6.013 |
| particles | 0.5 | 2.247 / 28.18 / 2.423 / 0.535 | 2.372 / 28.15 / 3.372 / 0.532 |
| particles | 0.67 | 2.252 / 28.33 / 2.852 / 0.533 | 2.272 / 28.40 / 3.140 / 0.539 |
| particles | 0.77 | 2.285 / 28.17 / 2.908 / 0.531 | 2.272 / 28.32 / 3.069 / 0.540 |
| smoke-transparency | 0.5 | 1.269 / 35.54 / 2.877 / 0.335 | 1.415 / 35.54 / 3.128 / 0.265 |
| smoke-transparency | 0.67 | 0.961 / 36.71 / 2.102 / 0.329 | 1.080 / 36.73 / 2.161 / 0.253 |
| smoke-transparency | 0.77 | 0.816 / 37.61 / 1.726 / 0.329 | 0.889 / 37.98 / 1.647 / 0.247 |

What the crops show (render scale 0.5):

- **Moving camera with disocclusion** (the thin geometry panning, the
  sphere rolling across it): TAAU smears the wires sideways into doubles
  and stair-steps the sphere's edge; FSR keeps the wires single and
  sharp, the sphere's edge smooth and the checker crisper, though its
  thinnest vertical wires break into dashes. FSR's clearest win: 3.6 dB to
  3.9 dB at every scale and a fifth to a quarter less flicker.
- **Thin geometry, still**: TAAU lies nearer the ground truth, FSR is
  crisper and two to three times steadier. FSR converges slowly from a
  reset (a cut): AMD's per-frame weight (0.74/16 of the Lanczos sum against
  a history weight of at most 1) gives it a time constant near 65 frames
  on sub-pixel wires at 0.5, which show beads for the first second that
  fade by the fourth; TAAU settles in about 96 frames. Measured on the
  thin geometry at 0.5: TAAU 3.79 → 2.25 by frame 96, FSR 4.10 → 3.02 at
  64, 2.36 at 256.
- **Grass under a moving camera**: even, TAAU ahead by at most 0.12.
- **Particles and transparency** (the night camp's sparks; lit smoke and
  debris): even on error; FSR's reactive mask keeps the smoke 20 % steadier.
  Both lose the smallest sparks at 0.5.

## Cost

`fsr::cost` (`OG_FSR_COST_SCENE=thin`), Metal on macrunner (Apple M4
Pro), the thin geometry, `taaSharpen` 0.5, each output size and scale run
alone under `og_gpu_acquire_all`: the frame's GPU time, the median of 64
frames after 16, in milliseconds. The frame's total, not the `taa`
pass's: Apple's pass timestamps move work between neighbouring passes
(`main` read 0.97 ms under TAAU and 0.29 ms under FSR in the same
scene), while their sum holds. In the near-grass scene, whose frame is
26 ms to 45 ms, the same differences are smaller and noisier (+0.1 ms to
+4.9 ms).

| output | scale | TAAU frame | FSR 3.1 frame | FSR's extra |
|---|---|---|---|---|
| 1920x1080 | 0.5 | 1.699 | 3.104 | +1.41 |
| 1920x1080 | 0.67 | 1.926 | 3.955 | +2.03 |
| 1920x1080 | 0.77 | 1.307 | 4.525 | +3.22 |
| 2560x1440 | 0.5 | 2.180 | 5.438 | +3.26 |
| 2560x1440 | 0.67 | 2.129 | 6.923 | +4.79 |
| 2560x1440 | 0.77 | 2.289 | 7.960 | +5.67 |

## The choice

FSR 3.1 is the better picture: clearly so whenever the camera or an
object moves, the case games spend most of their frames in, and steadier
everywhere. TAAU is cheaper, converges faster after a cut and is as good
on grass and particles. So:

- **Default and medium: TAAU.** The default is also every native run's
  resolve at render scale 1 (`og run`, `og test`, the references); FSR
  there would cost every game 1.4 ms to 3 ms at 1080p before its cost is
  brought down.
- **High: FSR 3.1** (`og.render.applyQuality(world, "high")`, the
  `upscaler` row of its table). A game sets `RenderSettings.upscaler`
  itself at any preset.
- **Low: TAAU.**
- **Dynamic resolution (G1-4):** automatic quality lowers the render scale
  with the upscaler kept (FSR's state is the output's size, so a step
  keeps its history: `fsr::a_render_scale_step_keeps_fsrs_history`), and
  once its preset steps cap the quality at medium or low it draws TAAU
  (a cap takes the cheaper upscaler), the history starting again at
  that step.

Open: FSR's cost. On Apple the port's frame is 2.5x the full AMD graph
pmndrs/upscaler measured at the same ratio (1.10 ms at 1080p, ratio 2),
and it grows with the render scale, so the render-resolution passes
(input preparation with its depth scatter, the two pyramids, reactivity)
are where to look first, pmndrs' fused passes the measured reference.
Once FSR costs near TAAU, it becomes the default. The browser tiers run
the same shaders and are measured at the wave's end.
