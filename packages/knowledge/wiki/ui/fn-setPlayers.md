<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setPlayers"></a>

### `og.ui.setPlayers`

Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it.

```ts
og.ui.setPlayers(count: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `count` | `number` | no | Players, 1 to 4 (clamped). |

**Returns** `void`.

Binding symbol `og_ui_set_players` · publish builds: kept

```ogt
fn setPlayers {
  doc "Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it."
  param count u32 "Players, 1 to 4 (clamped)."
  returns Void
}
```

<sub>Module `ui` line 1499</sub>
