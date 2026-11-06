<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-Story"></a>

### `og.Story` (resource)

The story's counters (written by og.story; the story itself is a state block). Registered at the first og.story.load.

Token `og.Story`, registered under the schema name `og.Story`; its value type is `Story`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lines` | `number` | `0` | — | — | read only | Lines shown so far (DialogueView.seq of the latest). |
| `conversations` | `number` | `0` | — | — | read only | Conversations not yet ended. |
| `activeQuests` | `number` | `0` | — | — | read only | Quests started and not yet completed or failed. |

```ogt
resource Story {
  doc "The story's counters (written by og.story; the story itself is a state block). Registered at the first og.story.load."
  field lines u32 "Lines shown so far (DialogueView.seq of the latest)." {
    readonly true
  }
  field conversations u32 "Conversations not yet ended." {
    readonly true
  }
  field activeQuests u32 "Quests started and not yet completed or failed." {
    readonly true
  }
}
```

<sub>`api.ogt` line 14463</sub>
