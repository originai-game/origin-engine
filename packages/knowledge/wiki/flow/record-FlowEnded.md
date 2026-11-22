<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowEnded"></a>

### `og.FlowEnded` (event)

A flow's last segment reached its cut with the conversation ended.

Token `og.FlowEnded`, registered under the schema name `og.FlowEnded`; its value type is `FlowEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The last node (at most 63 bytes). |
| `ending` | `string` | `""` | — | — | — | Its `og-flow-ending`, empty without one (at most 63 bytes). |

```ogt
event FlowEnded {
  doc "A flow's last segment reached its cut with the conversation ended."
  field flow Entity "The flow."
  field node String "The last node (at most 63 bytes)."
  field ending String "Its `og-flow-ending`, empty without one (at most 63 bytes)."
}
```

<sub>Module `flow` line 198</sub>
