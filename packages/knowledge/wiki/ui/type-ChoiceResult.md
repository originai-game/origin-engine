<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChoiceResult"></a>

### `ChoiceResult` (struct)

A choice control's response and the chosen index after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it; changed when the choice moved. |
| `selected` | `number` | `0` | — | — | — | The chosen index, to keep in game state and pass back next tick. |

```ogt
struct ChoiceResult {
  doc "A choice control's response and the chosen index after this tick's input."
  field response Response "What happened to it; changed when the choice moved."
  field selected u32 "The chosen index, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 16675</sub>
