<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-StoryAsset"></a>

### `StoryAsset` (struct)

A loaded dialogue or quest.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | dialogue or quest. |
| `name` | `string` | `""` | — | — | — | A dialogue's name or a quest's id: what og.story.start and og.story.startQuest take. |
| `nodes` | `readonly string[]` | `[]` | — | — | — | A dialogue's nodes, in order (a quest: its stages). |
| `speakers` | `readonly string[]` | `[]` | — | — | — | A dialogue's speakers (a quest: none). |
| `variables` | `readonly string[]` | `[]` | — | — | — | The variables it declares (var: the conversation's; global: the story's). |

```ogt
struct StoryAsset {
  doc "A loaded dialogue or quest."
  field kind String "dialogue or quest."
  field name String "A dialogue's name or a quest's id: what og.story.start and og.story.startQuest take."
  field nodes List.String "A dialogue's nodes, in order (a quest: its stages)."
  field speakers List.String "A dialogue's speakers (a quest: none)."
  field variables List.String "The variables it declares (var: the conversation's; global: the story's)."
}
```

<sub>`api.ogt` line 14503</sub>
