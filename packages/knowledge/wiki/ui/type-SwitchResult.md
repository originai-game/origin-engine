<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SwitchResult"></a>

### `SwitchResult` (struct)

A switch's or a check's state after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it. |
| `on` | `boolean` | `false` | — | — | — | Its state, to keep and pass back. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |

```ogt
struct SwitchResult {
  doc "A switch's or a check's state after this tick."
  field response Response "What happened to it."
  field on bool "Its state, to keep and pass back."
  field changed bool "It changed this tick."
}
```

<sub>Module `ui` line 3641</sub>
