<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowChartLink"></a>

### `FlowChartLink` (struct)

Where a flow node may lead, through logic nodes.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `string` | `""` | — | — | — | The node. |
| `to` | `string` | `""` | — | — | — | The flow node it may lead to. |
| `choice` | `number` | `0` | — | — | — | The ChoiceView.index that leads there; -1 for a node without choices. |
| `points` | `readonly number[]` | `[]` | — | — | — | A curve from one to the other as x, y pairs in graph units: a cubic leaving and arriving level, sampled in 16 segments (NodeGraph's link `points`). |

```ogt
struct FlowChartLink {
  doc "Where a flow node may lead, through logic nodes."
  field from String "The node."
  field to String "The flow node it may lead to."
  field choice i32 "The ChoiceView.index that leads there; -1 for a node without choices."
  field points List.f32 "A curve from one to the other as x, y pairs in graph units: a cubic leaving and arriving level, sampled in 16 segments (NodeGraph's link `points`)."
}
```

<sub>Module `flow` line 160</sub>
