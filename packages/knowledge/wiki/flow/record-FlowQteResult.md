<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowQteResult"></a>

### `og.FlowQteResult` (event)

A quick-time event resolved with a grade (`perfect`, `good`, or `miss`).

Token `og.FlowQteResult`, registered under the schema name `og.FlowQteResult`; its value type is `FlowQteResult`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `choice` | `number` | `0` | — | — | — | The ChoiceView.index. |
| `grade` | `string` | `""` | — | — | — | `perfect`, `good`, or `miss` (at most 63 bytes). |

```ogt
event FlowQteResult {
  doc "A quick-time event resolved with a grade (`perfect`, `good`, or `miss`)."
  field flow Entity "The flow."
  field node String "The node (at most 63 bytes)."
  field choice u32 "The ChoiceView.index."
  field grade String "`perfect`, `good`, or `miss` (at most 63 bytes)."
}
```

<sub>Module `flow` line 305</sub>
