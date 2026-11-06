<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-QuestView"></a>

### `QuestView` (struct)

A quest as a quest log shows it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `title` | `string` | `""` | — | — | — | Its title, resolved in the active locale. |
| `summary` | `string` | `""` | — | — | — | The current stage's summary, else the quest's, resolved. |
| `category` | `string` | `""` | — | — | — | main, side, daily... (the log's tabs). |
| `stage` | `string` | `""` | — | — | — | The current stage's id. |
| `status` | `string` | `""` | — | — | — | active, completed or failed. |
| `tracked` | `boolean` | `false` | — | — | — | Shown in the HUD tracker (og.story.track; the first quest started is tracked). |
| `isNew` | `boolean` | `false` | — | — | — | Not looked at since it started (og.story.markSeen clears it). |
| `updated` | `boolean` | `false` | — | — | — | Changed since it was last looked at (og.story.markSeen clears it). |
| `updatedTick` | `number` | `0` | — | — | — | The tick it last changed (sort the log by recent). |
| `giver` | `string` | `""` | — | — | — | Who gave it, resolved (empty: nobody). |
| `region` | `string` | `""` | — | — | — | Where it is, resolved (empty: nowhere). |
| `marker` | `MarkerView \| null` | `null` | — | — | — | Where it points now. |
| `rewards` | `readonly RewardView[]` | `[]` | — | — | — | Its rewards. |
| `objectives` | `readonly ObjectiveView[]` | `[]` | — | — | — | The current stage's objectives. |

```ogt
struct QuestView {
  doc "A quest as a quest log shows it."
  field id String "Its id."
  field title String "Its title, resolved in the active locale."
  field summary String "The current stage's summary, else the quest's, resolved."
  field category String "main, side, daily... (the log's tabs)."
  field stage String "The current stage's id."
  field status String "active, completed or failed."
  field tracked bool "Shown in the HUD tracker (og.story.track; the first quest started is tracked)."
  field isNew bool "Not looked at since it started (og.story.markSeen clears it)."
  field updated bool "Changed since it was last looked at (og.story.markSeen clears it)."
  field updatedTick u32 "The tick it last changed (sort the log by recent)."
  field giver String "Who gave it, resolved (empty: nobody)."
  field region String "Where it is, resolved (empty: nowhere)."
  field marker Option.MarkerView "Where it points now."
  field rewards List.RewardView "Its rewards."
  field objectives List.ObjectiveView "The current stage's objectives."
}
```

<sub>`api.ogt` line 14696</sub>
