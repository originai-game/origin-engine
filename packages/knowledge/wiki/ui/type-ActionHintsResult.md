<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ActionHintsResult"></a>

### `ActionHintsResult` (struct)

What the action bar answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pressed` | `string \| null` | `null` | — | — | — | The action of the hint pressed (a touch, a click): act as if the control fired. |
| `height` | `number` | `0` | — | `cssPx` | — | The room it takes from its edge of the screen, the safe area under it included (as laid out last tick): a scrolling screen pads its content's end by it, so nothing hides under the bar. Under a finger it is a bar of buttons across the bottom. |

```ogt
struct ActionHintsResult {
  doc "What the action bar answered this tick."
  field pressed Option.String "The action of the hint pressed (a touch, a click): act as if the control fired."
  field height f32 "The room it takes from its edge of the screen, the safe area under it included (as laid out last tick): a scrolling screen pads its content's end by it, so nothing hides under the bar. Under a finger it is a bar of buttons across the bottom." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 2374</sub>
