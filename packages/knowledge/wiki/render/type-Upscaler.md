<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Upscaler"></a>

### `Upscaler` (enum)

The temporal resolve under TAA (RenderSettings.upscaler), as og-render's Upscaler: what turns the jittered frame, drawn at the render scale, into the output. RenderCapabilities.upscalers names the ones this device draws. Without TAA neither runs: the frame is scaled bilinearly.

A string: one of the variants below. The zero value is `"taau"`.

| Variant | Doc |
|---|---|
| `"taau"` | The engine's TAA resolve with temporal upscaling: one pass, a neighbourhood clamp of the history and the RenderOptions.taaHistory filter, then the tonemap's sharpening at RenderOptions.taaSharpen. |
| `"fsr3"` | AMD FidelityFX Super Resolution 3.1's upscaler (MIT), in fp32 without subgroups so it runs alike on every host: depth-based disocclusion, thin-geometry locks, luma instability, Lanczos upsampling, then the tonemap's sharpening at taaSharpen's strength; transparent surfaces feed its reactive mask. |
| `"auto"` | fsr3 while the scene is drawn smaller than the output (render scale below 1, automatic quality's included), where it keeps the thin and moving detail taau loses; taau at the output's size, which costs less. A device without fsr3 (RenderCapabilities.upscalers) draws taau. |

```ogt
enum Upscaler {
  doc "The temporal resolve under TAA (RenderSettings.upscaler), as og-render's Upscaler: what turns the jittered frame, drawn at the render scale, into the output. RenderCapabilities.upscalers names the ones this device draws. Without TAA neither runs: the frame is scaled bilinearly."
  variant taau "The engine's TAA resolve with temporal upscaling: one pass, a neighbourhood clamp of the history and the RenderOptions.taaHistory filter, then the tonemap's sharpening at RenderOptions.taaSharpen."
  variant fsr3 "AMD FidelityFX Super Resolution 3.1's upscaler (MIT), in fp32 without subgroups so it runs alike on every host: depth-based disocclusion, thin-geometry locks, luma instability, Lanczos upsampling, then the tonemap's sharpening at taaSharpen's strength; transparent surfaces feed its reactive mask."
  variant auto "fsr3 while the scene is drawn smaller than the output (render scale below 1, automatic quality's included), where it keeps the thin and moving detail taau loses; taau at the output's size, which costs less. A device without fsr3 (RenderCapabilities.upscalers) draws taau."
}
```

<sub>Module `render` line 101</sub>
