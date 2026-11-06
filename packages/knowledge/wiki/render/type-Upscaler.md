<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Upscaler"></a>

### `Upscaler` (enum)

The temporal resolve under TAA (RenderSettings.upscaler), as og-render's Upscaler: what turns the jittered frame, drawn at the render scale, into the output. RenderCapabilities.upscalers names the ones this device draws. Without TAA neither runs: the frame is scaled bilinearly.

A string: one of the variants below. The zero value is `"taau"`.

| Variant | Doc |
|---|---|
| `"taau"` | The engine's TAA resolve with temporal upscaling: one pass, a neighbourhood clamp of the history and the RenderOptions.taaHistory filter, then RenderOptions.taaSharpen. The cheapest; softer on thin geometry and ghosts a little more on disocclusion. |
| `"fsr3"` | AMD FidelityFX Super Resolution 3.1's upscaler (MIT), in fp32 without subgroups so it runs alike on every host: depth-based disocclusion, locks that keep thin geometry and edges through motion, luma instability, Lanczos upsampling and RCAS sharpening (taaSharpen's strength). Transparent surfaces (the order-independent transparency's coverage) feed its reactive mask, so particles and glass do not smear. More GPU time than taau (docs/design.md §11 has the measurements). |

```ogt
enum Upscaler {
  doc "The temporal resolve under TAA (RenderSettings.upscaler), as og-render's Upscaler: what turns the jittered frame, drawn at the render scale, into the output. RenderCapabilities.upscalers names the ones this device draws. Without TAA neither runs: the frame is scaled bilinearly."
  variant taau "The engine's TAA resolve with temporal upscaling: one pass, a neighbourhood clamp of the history and the RenderOptions.taaHistory filter, then RenderOptions.taaSharpen. The cheapest; softer on thin geometry and ghosts a little more on disocclusion."
  variant fsr3 "AMD FidelityFX Super Resolution 3.1's upscaler (MIT), in fp32 without subgroups so it runs alike on every host: depth-based disocclusion, locks that keep thin geometry and edges through motion, luma instability, Lanczos upsampling and RCAS sharpening (taaSharpen's strength). Transparent surfaces (the order-independent transparency's coverage) feed its reactive mask, so particles and glass do not smear. More GPU time than taau (docs/design.md §11 has the measurements)."
}
```

<sub>`api.ogt` line 3837</sub>
