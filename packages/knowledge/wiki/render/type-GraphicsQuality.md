<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GraphicsQuality"></a>

### `GraphicsQuality` (enum)

A graphics preset: a row of RenderSettings' quality fields that og.render.applyQuality writes (its doc has the table). A settings page (og.ui.settingsTree) offers the three.

A string: one of the variants below. The zero value is `"medium"`.

| Variant | Doc |
|---|---|
| `"medium"` | The full frame: every quality field at its default. |
| `"low"` | The cheapest frame: no rect-light shadows, two shadowed lights, half-size shadow maps, plain fog, hard particles, no screen-space reflections, ambient occlusion or global illumination, FXAA for MSAA; and, for a game that turns one of those back on, ambient occlusion at half resolution and half the reflection and fog steps. |
| `"high"` | The full frame refined, at the GPU time of the effects the scene draws: FSR 3.1 as the temporal upscaler (sharper and steadier in motion than TAAU, about 1 ms to 2 ms more at 1080p and 1440p on an M4 Pro), the shadow maps at twice their side, ambient occlusion at full resolution whatever PostEffects.aoResolution asks, twice PostEffects.ssrSteps and twice the volumetric fog's slices (or raymarch steps). A scene without those effects differs from medium only by its shadow maps. |

```ogt
enum GraphicsQuality {
  doc "A graphics preset: a row of RenderSettings' quality fields that og.render.applyQuality writes (its doc has the table). A settings page (og.ui.settingsTree) offers the three."
  variant medium "The full frame: every quality field at its default."
  variant low "The cheapest frame: no rect-light shadows, two shadowed lights, half-size shadow maps, plain fog, hard particles, no screen-space reflections, ambient occlusion or global illumination, FXAA for MSAA; and, for a game that turns one of those back on, ambient occlusion at half resolution and half the reflection and fog steps."
  variant high "The full frame refined, at the GPU time of the effects the scene draws: FSR 3.1 as the temporal upscaler (sharper and steadier in motion than TAAU, about 1 ms to 2 ms more at 1080p and 1440p on an M4 Pro), the shadow maps at twice their side, ambient occlusion at full resolution whatever PostEffects.aoResolution asks, twice PostEffects.ssrSteps and twice the volumetric fog's slices (or raymarch steps). A scene without those effects differs from medium only by its shadow maps."
}
```

<sub>Module `render` line 537</sub>
