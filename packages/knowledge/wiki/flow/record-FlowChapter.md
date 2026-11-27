<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowChapter"></a>

### `og.FlowChapter` (event)

A flow entered a chapter's node (`og-flow-chapter`): a place to save.

Token `og.FlowChapter`, registered under the schema name `og.FlowChapter`; its value type is `FlowChapter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `chapter` | `string` | `""` | — | — | — | The chapter's i18n key (at most 63 bytes). |

```ogt
event FlowChapter {
  doc "A flow entered a chapter's node (`og-flow-chapter`): a place to save."
  field flow Entity "The flow."
  field node String "The node (at most 63 bytes)."
  field chapter String "The chapter's i18n key (at most 63 bytes)."
}
```

<sub>Module `flow` line 283</sub>
