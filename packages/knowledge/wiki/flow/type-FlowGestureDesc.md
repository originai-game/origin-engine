<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowGestureDesc"></a>

### `FlowGestureDesc` (desc struct)

A deterministic touch or pointer gesture fed into active swipe/drag QTEs (og.flow.gesture).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | `swipe` or `drag`. |
| `direction` | `string` | `""` | — | — | — | `up`, `down`, `left`, or `right`. |
| `progress` | `number` | `0` | — | `ratio` | — | For `drag`: 0 to 1 completion progress (1 completes); ignored for `swipe`. |

```ogt
struct FlowGestureDesc {
  doc "A deterministic touch or pointer gesture fed into active swipe/drag QTEs (og.flow.gesture)."
  desc true
  field kind String "`swipe` or `drag`."
  field direction String "`up`, `down`, `left`, or `right`."
  field progress f32 "For `drag`: 0 to 1 completion progress (1 completes); ignored for `swipe`." {
    unit ratio
  }
}
```

<sub>Module `flow` line 191</sub>
