<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-LitFeature"></a>

### `LitFeature` (enum)

An optional feature of the lit shaders that RenderOptions.without can leave out. Every lit shader (meshes, material shaders, clustered meshes, terrain, grass, water, scattered copies, lit particles) carries each one's code behind a runtime test, and its pipelines compile it whether or not a scene uses it. RenderCapabilities.without lists those the renderer was made without.

A string: one of the variants below. The zero value is `"surfaceWeather"`.

| Variant | Doc |
|---|---|
| `"surfaceWeather"` | The weather on lit surfaces: og.WeatherState's wet ground, rain streaks, puddles, snow and dust, and the shore a sea's surf wets (og.Water). Left out, surfaces stay dry; og.Weather's particles, fog and light still draw. |
| `"gi"` | The probes of GlobalIllumination (method probes or both): left out, they are not traced and lit surfaces do not gather them, so those methods bounce no light from probes (both keeps its screen-space rays). The screen-space method and LightVolume are not this feature. |
| `"areaLights"` | Rect lights (og.RectLight): left out, a rect light is not drawn, lights nothing and casts no shadow. |
| `"decals"` | Decals (og.Decal): left out, none is drawn. |
| `"caustics"` | The caustics a water surface casts on what lies below it (og.Water.caustics): left out, none is drawn. |

```ogt
enum LitFeature {
  doc "An optional feature of the lit shaders that RenderOptions.without can leave out. Every lit shader (meshes, material shaders, clustered meshes, terrain, grass, water, scattered copies, lit particles) carries each one's code behind a runtime test, and its pipelines compile it whether or not a scene uses it. RenderCapabilities.without lists those the renderer was made without."
  variant surfaceWeather "The weather on lit surfaces: og.WeatherState's wet ground, rain streaks, puddles, snow and dust, and the shore a sea's surf wets (og.Water). Left out, surfaces stay dry; og.Weather's particles, fog and light still draw."
  variant gi "The probes of GlobalIllumination (method probes or both): left out, they are not traced and lit surfaces do not gather them, so those methods bounce no light from probes (both keeps its screen-space rays). The screen-space method and LightVolume are not this feature."
  variant areaLights "Rect lights (og.RectLight): left out, a rect light is not drawn, lights nothing and casts no shadow."
  variant decals "Decals (og.Decal): left out, none is drawn."
  variant caustics "The caustics a water surface casts on what lies below it (og.Water.caustics): left out, none is drawn."
}
```

<sub>Module `render` line 103</sub>
