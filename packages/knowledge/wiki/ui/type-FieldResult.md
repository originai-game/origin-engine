<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FieldResult"></a>

### `FieldResult` (struct)

A text field's response and its text after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it. |
| `value` | `string` | `""` | — | — | — | The text, to keep in game state and pass back next tick. |

```ogt
struct FieldResult {
  doc "A text field's response and its text after this tick's input."
  field response Response "What happened to it."
  field value String "The text, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 16077</sub>
