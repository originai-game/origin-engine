<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowInspected"></a>

### `og.FlowInspected` (event)

An `inspect` hotspot was triggered without cutting (og.flow.inspect).

Token `og.FlowInspected`, registered under the schema name `og.FlowInspected`; its value type is `FlowInspected`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `hotspot` | `string` | `""` | — | — | — | The hotspot's id (at most 63 bytes). |

```ogt
event FlowInspected {
  doc "An `inspect` hotspot was triggered without cutting (og.flow.inspect)."
  field flow Entity "The flow."
  field node String "The node (at most 63 bytes)."
  field hotspot String "The hotspot's id (at most 63 bytes)."
}
```

<sub>Module `flow` line 298</sub>
