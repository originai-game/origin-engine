<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setHudLayout"></a>

### `og.ui.setHudLayout`

Gives a player's HUD layout back (og.ui.hudLayout's JSON; `{}` resets it). Answers false and leaves the layout as it was when it does not read (the reason is in the frame's warnings).

```ts
og.ui.setHudLayout(player: number, layout: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `player` | `number` | no | The player, 0 to 3. |
| `layout` | `string` | no | The layout's JSON. |

**Returns** `boolean`.

Binding symbol `og_ui_set_hud_layout` · publish builds: kept

```ogt
fn setHudLayout {
  doc "Gives a player's HUD layout back (og.ui.hudLayout's JSON; `{}` resets it). Answers false and leaves the layout as it was when it does not read (the reason is in the frame's warnings)."
  param player u32 "The player, 0 to 3."
  param layout String "The layout's JSON."
  returns bool
}
```

<sub>Module `ui` line 1670</sub>
