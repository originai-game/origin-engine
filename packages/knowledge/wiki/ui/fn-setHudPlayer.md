<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setHudPlayer"></a>

### `og.ui.setHudPlayer`

Whose layout this screen shows and edits, 0 to 3 (0 by default; kept until changed): each player of a shared screen keeps a layout of their own.

```ts
og.ui.setHudPlayer(player: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `player` | `number` | no | The player (clamped to 3). |

**Returns** `void`.

Binding symbol `og_ui_set_hud_player` · publish builds: kept

```ogt
fn setHudPlayer {
  doc "Whose layout this screen shows and edits, 0 to 3 (0 by default; kept until changed): each player of a shared screen keeps a layout of their own."
  param player u32 "The player (clamped to 3)."
  returns Void
}
```

<sub>`api.ogt` line 17426</sub>
