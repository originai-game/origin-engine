<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-setDisplay"></a>

### `og.render.setDisplay`

Set the player's graphics settings (DisplaySettings: preset, frame cap, battery saver, render scale, automatic quality, fullscreen, power preference). A call merges over the current settings: only the fields it names change, so a settings screen sets the one the player moved without restating the rest (og.render.resetDisplay goes back to the defaults). Applied from the next frame (the power preference at the next load); presentation only, so it is allowed inside any system and never changes the simulation.

```ts
og.render.setDisplay(world: World, settings?: DisplaySettings): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose display it is. |
| `settings` | `DisplaySettings` | yes | The settings to change: fields left out keep their values (the defaults before any call), so a game that sets its frame cap keeps the player's render scale; quality null is the game's own. |

**Returns** `Result<void>`: Refused for a world that is not alive.

Binding symbol `og_render_set_display` · publish builds: kept

```ogt
fn setDisplay {
  doc "Set the player's graphics settings (DisplaySettings: preset, frame cap, battery saver, render scale, automatic quality, fullscreen, power preference). A call merges over the current settings: only the fields it names change, so a settings screen sets the one the player moved without restating the rest (og.render.resetDisplay goes back to the defaults). Applied from the next frame (the power preference at the next load); presentation only, so it is allowed inside any system and never changes the simulation."
  param world World "The world whose display it is."
  param settings DisplaySettings "The settings to change: fields left out keep their values (the defaults before any call), so a game that sets its frame cap keeps the player's render scale; quality null is the game's own."
  returns Result.Void "Refused for a world that is not alive."
}
```

<sub>Module `render` line 2836</sub>
