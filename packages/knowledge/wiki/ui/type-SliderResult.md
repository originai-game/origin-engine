<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SliderResult"></a>

### `SliderResult` (struct)

A slider's response and its value after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it. |
| `value` | `number` | `0` | — | — | — | The value, to keep in game state and pass back next tick. |

```ogt
struct SliderResult {
  doc "A slider's response and its value after this tick's input."
  field response Response "What happened to it."
  field value f32 "The value, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 16065</sub>
