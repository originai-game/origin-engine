<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CheckListResult"></a>

### `CheckListResult` (struct)

A checkbox list's response and values after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it; changed when any box flipped. |
| `values` | `readonly boolean[]` | `[]` | — | — | — | The values, to keep in game state and pass back next tick. |

```ogt
struct CheckListResult {
  doc "A checkbox list's response and values after this tick's input."
  field response Response "What happened to it; changed when any box flipped."
  field values List.bool "The values, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 16645</sub>
