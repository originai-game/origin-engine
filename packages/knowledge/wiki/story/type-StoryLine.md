<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-StoryLine"></a>

### `StoryLine` (struct)

A line of a conversation's backlog.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seq` | `number` | `0` | — | — | — | Its DialogueView.seq. |
| `node` | `string` | `""` | — | — | — | Its node. |
| `speaker` | `SpeakerView` | `{ id: "", name: "", entity: 0, portrait: "", expression: "", side: "" }` | — | — | — | Who said it. |
| `key` | `string` | `""` | — | — | — | Its i18n key. |
| `args` | `unknown` | `null` | — | — | — | Its arguments. |
| `text` | `string` | `""` | — | — | — | The line, resolved in the active locale. |
| `voice` | `string` | `""` | — | — | — | Its voice clip (empty: none). |
| `seen` | `boolean` | `false` | — | — | — | It had been read before it was shown. |
| `chosen` | `ChosenView \| null` | `null` | — | — | — | The choice made on it, if any. |

```ogt
struct StoryLine {
  doc "A line of a conversation's backlog."
  field seq u32 "Its DialogueView.seq."
  field node String "Its node."
  field speaker SpeakerView "Who said it."
  field key String "Its i18n key."
  field args Json "Its arguments."
  field text String "The line, resolved in the active locale."
  field voice String "Its voice clip (empty: none)."
  field seen bool "It had been read before it was shown."
  field chosen Option.ChosenView "The choice made on it, if any."
}
```

<sub>Module `story` line 154</sub>
