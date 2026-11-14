<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-resetDisplay"></a>

### `og.render.resetDisplay`

Put every graphics setting back to its default (DisplaySettings' zero value: the game's own quality, frame cap auto, render scale 1, automatic quality on, no battery saver, windowed, high performance): a settings page's Reset to defaults. Applied from the next frame like og.render.setDisplay; a power preference that changes back applies at the next load.

```ts
og.render.resetDisplay(world: World): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose display it is. |

**Returns** `Result<void>`: Refused for a world that is not alive.

Binding symbol `og_render_reset_display` · publish builds: kept

```ogt
fn resetDisplay {
  doc "Put every graphics setting back to its default (DisplaySettings' zero value: the game's own quality, frame cap auto, render scale 1, automatic quality on, no battery saver, windowed, high performance): a settings page's Reset to defaults. Applied from the next frame like og.render.setDisplay; a power preference that changes back applies at the next load."
  param world World "The world whose display it is."
  returns Result.Void "Refused for a world that is not alive."
}
```

<sub>Module `render` line 3105</sub>
