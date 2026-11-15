<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HudEdit"></a>

### `HudEdit` (struct)

The HUD edit mode as the last frame left it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `on` | `boolean` | `false` | — | — | — | The edit mode is on. |
| `player` | `number` | `0` | — | — | — | The player whose layout the screen shows and edits. |
| `selected` | `string` | `""` | — | — | — | The key of the element being edited; empty for none. |
| `placement` | `HudPlacement` | `{ x: 0, y: 0, scale: 0, opacity: 0 }` | — | — | — | Its placement. |
| `keys` | `readonly string[]` | `[]` | — | — | — | The customisable elements on screen, in drawing order. |
| `changed` | `boolean` | `false` | — | — | — | The player changed the layout this tick (a move ended, a step, a reset): the time to save og.ui.hudLayout in the game's settings. |

```ogt
struct HudEdit {
  doc "The HUD edit mode as the last frame left it."
  field on bool "The edit mode is on."
  field player u32 "The player whose layout the screen shows and edits."
  field selected String "The key of the element being edited; empty for none."
  field placement HudPlacement "Its placement."
  field keys List.String "The customisable elements on screen, in drawing order."
  field changed bool "The player changed the layout this tick (a move ended, a step, a reset): the time to save og.ui.hudLayout in the game's settings."
}
```

<sub>Module `ui` line 636</sub>
