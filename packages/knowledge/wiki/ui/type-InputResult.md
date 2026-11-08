<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-InputResult"></a>

### `InputResult` (struct)

A text field's state after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | The field's response (focused while it is edited, submitted on Enter). |
| `value` | `string` | `""` | — | — | — | The text, to keep and pass back. |
| `changed` | `boolean` | `false` | — | — | — | The text changed this tick. |
| `submitted` | `boolean` | `false` | — | — | — | Enter was pressed in it. |
| `cleared` | `boolean` | `false` | — | — | — | The clear button emptied it. |

```ogt
struct InputResult {
  doc "A text field's state after this tick."
  field response Response "The field's response (focused while it is edited, submitted on Enter)."
  field value String "The text, to keep and pass back."
  field changed bool "The text changed this tick."
  field submitted bool "Enter was pressed in it."
  field cleared bool "The clear button emptied it."
}
```

<sub>Module `ui` line 3873</sub>
