<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ToggleResult"></a>

### `ToggleResult` (struct)

A toggle's response and its state after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it. |
| `on` | `boolean` | `false` | — | — | — | The state, to keep in game state and pass back next tick. |

```ogt
struct ToggleResult {
  doc "A toggle's response and its state after this tick's input."
  field response Response "What happened to it."
  field on bool "The state, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 15981</sub>
