<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowSegment"></a>

### `og.FlowSegment` (event)

A flow began a node's segment.

Token `og.FlowSegment`, registered under the schema name `og.FlowSegment`; its value type is `FlowSegment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `segment` | `string` | `""` | — | — | — | Its segment's asset id (at most 63 bytes). |

```ogt
event FlowSegment {
  doc "A flow began a node's segment."
  field flow Entity "The flow."
  field node String "The node (at most 63 bytes)."
  field segment String "Its segment's asset id (at most 63 bytes)."
}
```

<sub>Module `flow` line 269</sub>
