<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hudLayout"></a>

### `og.ui.hudLayout`

A player's HUD layout as JSON, for the game's settings: each moved, scaled or faded element's key with its x, y, scale and opacity. Save it when HudEdit.changed says so; give it back with og.ui.setHudLayout on the next run.

```ts
og.ui.hudLayout(player: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `player` | `number` | no | The player, 0 to 3. |

**Returns** `string`.

Binding symbol `og_ui_hud_layout` · publish builds: kept

```ogt
fn hudLayout {
  doc "A player's HUD layout as JSON, for the game's settings: each moved, scaled or faded element's key with its x, y, scale and opacity. Save it when HudEdit.changed says so; give it back with og.ui.setHudLayout on the next run."
  param player u32 "The player, 0 to 3."
  returns String
}
```

<sub>Module `ui` line 1589</sub>
