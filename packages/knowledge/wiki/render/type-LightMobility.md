<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-LightMobility"></a>

### `LightMobility` (enum)

How a light may change while the game runs, as Unreal's light mobility: which of its light og bake lightmaps bakes into its scene's lightmaps and light volumes, and which the renderer draws every frame.

See `render §4`.

A string: one of the variants below. The zero value is `"movable"`.

| Variant | Doc |
|---|---|
| `"movable"` | Moves or changes in any way: drawn whole every frame, and none of its light is baked. |
| `"stationary"` | Stays where it is; its colour and intensity may change. Drawn every frame, the light it bounces baked; past RenderSettings.staticLightReach its diffuse light may come from probes (render §3). The default. |
| `"static"` | Never moves or changes: its direct light is baked too, and while its scene's bake is drawn the renderer leaves it out (render §3). Unbaked, it draws as stationary. Under a running og.DayNight a DirectionalLight is movable. |

```ogt
enum LightMobility {
  doc "How a light may change while the game runs, as Unreal's light mobility: which of its light og bake lightmaps bakes into its scene's lightmaps and light volumes, and which the renderer draws every frame."
  guide "render §4"
  variant movable "Moves or changes in any way: drawn whole every frame, and none of its light is baked."
  variant stationary "Stays where it is; its colour and intensity may change. Drawn every frame, the light it bounces baked; past RenderSettings.staticLightReach its diffuse light may come from probes (render §3). The default."
  variant static "Never moves or changes: its direct light is baked too, and while its scene's bake is drawn the renderer leaves it out (render §3). Unbaked, it draws as stationary. Under a running og.DayNight a DirectionalLight is movable."
}
```

<sub>Module `render` line 562</sub>
