<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ValueSliderResult"></a>

### `ValueSliderResult` (struct)

A slider's value after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it. |
| `value` | `number` | `0` | — | — | — | The value, to keep and pass back. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |
| `committed` | `boolean` | `false` | — | — | — | A drag ended, or a key or pad step landed: when to save or apply a costly setting. |

```ogt
struct ValueSliderResult {
  doc "A slider's value after this tick."
  field response Response "What happened to it."
  field value f32 "The value, to keep and pass back."
  field changed bool "It changed this tick."
  field committed bool "A drag ended, or a key or pad step landed: when to save or apply a costly setting."
}
```

<sub>`api.ogt` line 19431</sub>
