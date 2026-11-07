<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-QuestUpdated"></a>

### `og.QuestUpdated` (event)

A quest started, counted, changed stage, completed or failed. Sent the tick after.

Token `og.QuestUpdated`, registered under the schema name `og.QuestUpdated`; its value type is `QuestUpdated`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `quest` | `string` | `""` | — | — | — | The quest's id. |
| `status` | `string` | `""` | — | — | — | active, completed or failed. |

```ogt
event QuestUpdated {
  doc "A quest started, counted, changed stage, completed or failed. Sent the tick after."
  field quest String "The quest's id."
  field status String "active, completed or failed."
}
```

<sub>Module `story` line 56</sub>
