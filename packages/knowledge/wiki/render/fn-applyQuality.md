<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-applyQuality"></a>

### `og.render.applyQuality`

Apply a graphics preset: write its row of RenderSettings' quality fields (the rest of RenderSettings keeps its values) and record it in RenderSettings.quality. The renderer reads those fields, so what the probe and dump_world show is what the next frame draws. Medium is every field's default. The device caps shadowedLights by RenderCapabilities.maxShadowedLights and msaaSamples by maxMsaaSamples; the fields keep the row's value. Inside a system, the system must declare RenderSettings among its writes.

See `looks §16`.

Also known as: graphics quality, low medium high graphics, graphics options menu, quality preset

```ts
og.render.applyQuality(world: World, preset: GraphicsQuality): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose RenderSettings change. |
| `preset` | `GraphicsQuality` | no | The preset. |

**Returns** `Result<void>`: Refused for a world that is not alive, or inside a system that did not declare RenderSettings among its writes.

Binding symbol `og_render_apply_quality` · publish builds: kept

```ogt
fn applyQuality {
  aliases "graphics quality, low medium high graphics, graphics options menu, quality preset"
  doc "Apply a graphics preset: write its row of RenderSettings' quality fields (the rest of RenderSettings keeps its values) and record it in RenderSettings.quality. The renderer reads those fields, so what the probe and dump_world show is what the next frame draws. Medium is every field's default. The device caps shadowedLights by RenderCapabilities.maxShadowedLights and msaaSamples by maxMsaaSamples; the fields keep the row's value. Inside a system, the system must declare RenderSettings among its writes."
  guide "looks §16"
  param world World "The world whose RenderSettings change."
  param preset GraphicsQuality "The preset."
  returns Result.Void "Refused for a world that is not alive, or inside a system that did not declare RenderSettings among its writes."
}
```

<sub>Module `render` line 2991</sub>
