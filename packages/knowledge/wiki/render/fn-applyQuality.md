<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-applyQuality"></a>

### `og.render.applyQuality`

Apply a graphics preset: write its row of RenderSettings' quality fields (the rest of RenderSettings keeps its values) and record it in RenderSettings.quality. The renderer reads those fields and nothing else, so what the probe and dump_world show of RenderSettings is what the next frame draws. A game calls it when the player picks a preset on its settings page (og.ui.settingsTree), and applies a saved preset itself at start. Inside a system, the system must declare RenderSettings among its writes. The rows:

| Field | low | medium | high |
|---|---|---|---|
| rectShadows | false | true | true |
| shadowedLights | 2 | 8 | 16 |
| shadowCascades | 2 | 3 | 4 |
| shadowMapScale | 0.5 | 1 | 2 |
| volumetricFog | false | true | true |
| softParticles | false | true | true |
| screenSpaceReflections | false | true | true |
| ambientOcclusion | false | true | true |
| aoResolution | half | postEffects | full |
| ssrStepScale | 0.5 | 1 | 2 |
| volumetricStepScale | 0.5 | 1 | 2 |
| msaaSamples | 1 | 4 | 4 |
| upscaler | taau | taau | fsr3 |

Medium is every field's default, the frame drawn before any preset. The device caps two of the rows, as RenderCapabilities says: shadowedLights by maxShadowedLights and msaaSamples by maxMsaaSamples; the fields keep the row's value.

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
  doc "Apply a graphics preset: write its row of RenderSettings' quality fields (the rest of RenderSettings keeps its values) and record it in RenderSettings.quality. The renderer reads those fields and nothing else, so what the probe and dump_world show of RenderSettings is what the next frame draws. A game calls it when the player picks a preset on its settings page (og.ui.settingsTree), and applies a saved preset itself at start. Inside a system, the system must declare RenderSettings among its writes. The rows:"
  doc "| Field | low | medium | high |"
  doc "|---|---|---|---|"
  doc "| rectShadows | false | true | true |"
  doc "| shadowedLights | 2 | 8 | 16 |"
  doc "| shadowCascades | 2 | 3 | 4 |"
  doc "| shadowMapScale | 0.5 | 1 | 2 |"
  doc "| volumetricFog | false | true | true |"
  doc "| softParticles | false | true | true |"
  doc "| screenSpaceReflections | false | true | true |"
  doc "| ambientOcclusion | false | true | true |"
  doc "| aoResolution | half | postEffects | full |"
  doc "| ssrStepScale | 0.5 | 1 | 2 |"
  doc "| volumetricStepScale | 0.5 | 1 | 2 |"
  doc "| msaaSamples | 1 | 4 | 4 |"
  doc "| upscaler | taau | taau | fsr3 |"
  doc "Medium is every field's default, the frame drawn before any preset. The device caps two of the rows, as RenderCapabilities says: shadowedLights by maxShadowedLights and msaaSamples by maxMsaaSamples; the fields keep the row's value."
  param world World "The world whose RenderSettings change."
  param preset GraphicsQuality "The preset."
  returns Result.Void "Refused for a world that is not alive, or inside a system that did not declare RenderSettings among its writes."
}
```

<sub>`api.ogt` line 6502</sub>
