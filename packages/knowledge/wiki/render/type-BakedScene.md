<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-BakedScene"></a>

### `BakedScene` (struct)

One instantiated scene's baked lighting, as og.render.bakedLighting reports it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `string` | `""` | — | — | — | The scene's asset id, such as `scenes/cornell.ogt`. |
| `baked` | `boolean` | `false` | — | — | — | Its lightmap and probes are loaded and drawn. |
| `reason` | `string` | `""` | — | — | — | Why not, when not: the scene has no bake (`og bake lightmaps` writes `<scene>.lightmap.ogt`), its bake is stale (a static model or the scene changed since; the reason names which), or RenderSettings.bakedLighting is off. Empty when baked. |
| `models` | `number` | `0` | — | — | — | Static model instances drawn with the lightmap. |
| `pages` | `number` | `0` | — | — | — | Lightmap pages (RGBA16F texture array layers). |
| `texels` | `number` | `0` | — | — | — | The pages' side in texels. |
| `volumes` | `number` | `0` | — | — | — | LightVolume grids its dynamic objects sample. |
| `probes` | `number` | `0` | — | — | — | Probes in those volumes. |

```ogt
struct BakedScene {
  doc "One instantiated scene's baked lighting, as og.render.bakedLighting reports it."
  field scene String "The scene's asset id, such as `scenes/cornell.ogt`."
  field baked bool "Its lightmap and probes are loaded and drawn."
  field reason String "Why not, when not: the scene has no bake (`og bake lightmaps` writes `<scene>.lightmap.ogt`), its bake is stale (a static model or the scene changed since; the reason names which), or RenderSettings.bakedLighting is off. Empty when baked."
  field models u32 "Static model instances drawn with the lightmap."
  field pages u32 "Lightmap pages (RGBA16F texture array layers)."
  field texels u32 "The pages' side in texels."
  field volumes u32 "LightVolume grids its dynamic objects sample."
  field probes u32 "Probes in those volumes."
}
```

<sub>Module `render` line 3925</sub>
