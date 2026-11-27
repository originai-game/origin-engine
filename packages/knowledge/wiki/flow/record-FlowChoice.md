<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowChoice"></a>

### `og.FlowChoice` (event)

A flow took a choice at its cut (before og.story ran it).

Token `og.FlowChoice`, registered under the schema name `og.FlowChoice`; its value type is `FlowChoice`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `index` | `number` | `0` | — | — | — | The ChoiceView.index taken. |
| `timedOut` | `boolean` | `false` | — | — | — | Nothing was chosen in the window: the default was taken. |

```ogt
event FlowChoice {
  doc "A flow took a choice at its cut (before og.story ran it)."
  field flow Entity "The flow."
  field node String "The node (at most 63 bytes)."
  field index u32 "The ChoiceView.index taken."
  field timedOut bool "Nothing was chosen in the window: the default was taken."
}
```

<sub>Module `flow` line 275</sub>
