# Temporal upscalers: TAAU and FSR 3.1

Under TAA the engine resolves the jittered frame, drawn at the render
scale, to the output with one of two upscalers: `RenderSettings.upscaler`
(`og-render`'s `RenderQuality::upscaler`), `taau` or `fsr3`.
`RenderCapabilities.upscalers` names the ones a device draws: both on every
host and tier.

## The two

- **TAAU** (`taa.wgsl`): one pass. The sample nearest each output pixel,
  weighted by its distance, blended into an output-sized history that is
  clamped to the current samples' neighbourhood and resampled with the
  bilinear or Catmull-Rom filter (`RenderOptions.taaHistory`), then
  `taaSharpen`'s pass.
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
and Firefox lack, so one shader serves every host, at the cost of AMD's
fp16 speed-up where a GPU would have it.

Other web upscalers (pmndrs/upscaler, a cut-down FSR 3.1; three.js
`TAAUNode`, a TAAU of the same class as ours; Arm ASR; Snapdragon GSR 2)
beat neither of the two, so none is ported.

## The choice

FSR 3.1 is the better picture whenever the camera or an object moves, and
steadier everywhere; TAAU is cheaper (FSR costs about 1 ms to 2 ms more at
1080p and 1440p), converges faster after a cut and is as good on grass
and particles. So:

- **Default and medium: TAAU.** The default is also every native run's
  resolve at render scale 1 (`og run`, `og test`, the references).
- **High: FSR 3.1** (`og.render.applyQuality(world, "high")`). A game sets
  `RenderSettings.upscaler` itself at any preset.
- **Low: TAAU.**
- **Dynamic resolution:** automatic quality lowers the render scale with
  the upscaler kept (FSR's state is the output's size, so a step keeps its
  history), and once its preset steps cap the quality at medium or low it
  draws TAAU (a cap takes the cheaper upscaler), the history starting
  again at that step.

`fsr::compare` and `fsr::cost` (`crates/og-tests/tests/fsr.rs`, ignored,
run by hand on Metal) measure quality against a 4-samples-a-pixel ground
truth and the frame's cost; their numbers go in reports.
