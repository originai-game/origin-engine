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
| `label` | `string` | `""` | — | — | — | The choice's i18n text key, or empty. |
| `outcome` | `string` | `""` | — | — | — | The `#outcome` name if this link is an outcome, else empty. |
| `conditional` | `boolean` | `false` | — | — | — | True when reaching `to` passes through a Yarn condition or multiple targets. |
| `points` | `readonly number[]` | `[]` | — | — | — | A curve from one to the other as x, y pairs in graph units: a cubic leaving and arriving level, sampled in 16 segments (NodeGraph's link `points`). |

```ogt
struct FlowChartLink {
  doc "Where a flow node may lead, through logic nodes."
  field from String "The node."
  field to String "The flow node it may lead to."
  field choice i32 "The ChoiceView.index that leads there; -1 for a node without choices."
  field label String "The choice's i18n text key, or empty."
  field outcome String "The `#outcome` name if this link is an outcome, else empty."
  field conditional bool "True when reaching `to` passes through a Yarn condition or multiple targets."
  field points List.f32 "A curve from one to the other as x, y pairs in graph units: a cubic leaving and arriving level, sampled in 16 segments (NodeGraph's link `points`)."
}
```

<sub>Module `flow` line 250</sub>
