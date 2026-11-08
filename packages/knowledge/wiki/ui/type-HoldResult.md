<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HoldResult"></a>

### `HoldResult` (struct)

A hold button's state after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, player: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | What happened to it. |
| `confirmed` | `boolean` | `false` | — | — | — | The hold completed this tick: do the thing. |
| `holding` | `boolean` | `false` | — | — | — | It is filling. |
| `progress` | `number` | `0` | — | `ratio` | — | How full it is, 0 to 1 (it drains back when let go). |

```ogt
struct HoldResult {
  doc "A hold button's state after this tick."
  field response Response "What happened to it."
  field confirmed bool "The hold completed this tick: do the thing."
  field holding bool "It is filling."
  field progress f32 "How full it is, 0 to 1 (it drains back when let go)." {
    unit ratio
  }
}
```

<sub>Module `ui` line 3372</sub>
