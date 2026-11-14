<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GraphicsQuality"></a>

### `GraphicsQuality` (enum)

A graphics preset: a row of RenderSettings' quality fields that og.render.applyQuality writes (its doc has the table). A settings page offers the three.

See `render §1`.

A string: one of the variants below. The zero value is `"medium"`.

| Variant | Doc |
|---|---|
| `"medium"` | The full frame: every quality field at its default. |
| `"low"` | The cheapest frame: no rect-light shadows, two shadowed lights, half-size shadow maps, plain fog, hard particles, no SSR, ambient occlusion or GI, FXAA for MSAA; for a game that turns one back on, half-resolution AO and half the reflection and fog steps. |
| `"high"` | The full frame refined: FSR 3.1 as the temporal upscaler, shadow maps at twice their side, ambient occlusion at full resolution whatever PostEffects.aoResolution asks, twice PostEffects.ssrSteps and twice the volumetric fog's slices (or raymarch steps). |

```ogt
enum GraphicsQuality {
  doc "A graphics preset: a row of RenderSettings' quality fields that og.render.applyQuality writes (its doc has the table). A settings page offers the three."
  variant medium "The full frame: every quality field at its default."
  variant low "The cheapest frame: no rect-light shadows, two shadowed lights, half-size shadow maps, plain fog, hard particles, no SSR, ambient occlusion or GI, FXAA for MSAA; for a game that turns one back on, half-resolution AO and half the reflection and fog steps."
  variant high "The full frame refined: FSR 3.1 as the temporal upscaler, shadow maps at twice their side, ambient occlusion at full resolution whatever PostEffects.aoResolution asks, twice PostEffects.ssrSteps and twice the volumetric fog's slices (or raymarch steps)."
  guide "render §1"
}
```

<sub>Module `render` line 622</sub>
