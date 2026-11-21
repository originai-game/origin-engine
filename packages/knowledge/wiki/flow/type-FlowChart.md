<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowChart"></a>

### `FlowChart` (struct)

A dialogue's flow nodes laid out as a flowchart, columns by depth from the start: what a flowchart screen draws (the built-in NodeGraph), the game marking what the player has seen.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nodes` | `readonly FlowChartNode[]` | `[]` | — | — | — | Its flow nodes, in file order. |
| `links` | `readonly FlowChartLink[]` | `[]` | — | — | — | Where each leads. |
| `width` | `number` | `0` | — | — | — | Its extent across, graph units (NodeGraph's view is `[0, 0, width, height]`). |
| `height` | `number` | `0` | — | — | — | Its extent down. |

```ogt
struct FlowChart {
  doc "A dialogue's flow nodes laid out as a flowchart, columns by depth from the start: what a flowchart screen draws (the built-in NodeGraph), the game marking what the player has seen."
  field nodes List.FlowChartNode "Its flow nodes, in file order."
  field links List.FlowChartLink "Where each leads."
  field width f32 "Its extent across, graph units (NodeGraph's view is `[0, 0, width, height]`)."
  field height f32 "Its extent down."
}
```

<sub>Module `flow` line 163</sub>
