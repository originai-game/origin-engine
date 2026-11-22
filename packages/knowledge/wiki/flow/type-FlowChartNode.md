<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowChartNode"></a>

### `FlowChartNode` (struct)

A flow node on og.flow.chart's layout.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `node` | `string` | `""` | — | — | — | Its title. |
| `segment` | `string` | `""` | — | — | — | Its segment's asset id. |
| `chapter` | `string` | `""` | — | — | — | Its `og-flow-chapter`, if any. |
| `ending` | `string` | `""` | — | — | — | Its `og-flow-ending`, if any. |
| `depth` | `number` | `0` | — | — | — | Its column: the fewest flow nodes from the start before it (unreached nodes after the deepest). |
| `x` | `number` | `0` | — | — | — | Its centre across, graph units (NodeGraph's). |
| `y` | `number` | `0` | — | — | — | Its centre down. |

```ogt
struct FlowChartNode {
  doc "A flow node on og.flow.chart's layout."
  field node String "Its title."
  field segment String "Its segment's asset id."
  field chapter String "Its `og-flow-chapter`, if any."
  field ending String "Its `og-flow-ending`, if any."
  field depth u32 "Its column: the fewest flow nodes from the start before it (unreached nodes after the deepest)."
  field x f32 "Its centre across, graph units (NodeGraph's)."
  field y f32 "Its centre down."
}
```

<sub>Module `flow` line 149</sub>
