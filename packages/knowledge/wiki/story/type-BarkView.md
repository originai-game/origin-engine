<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-BarkView"></a>

### `BarkView` (struct)

A bark being said.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who says it. |
| `dialogue` | `string` | `""` | — | — | — | Its dialogue. |
| `bark` | `string` | `""` | — | — | — | Its id. |
| `seq` | `number` | `0` | — | — | — | The line's number: a new number is a new bark. |
| `key` | `string` | `""` | — | — | — | The line's i18n key. |
| `args` | `unknown` | `null` | — | — | — | The arguments it was resolved with (evaluated when it was said, world reads included). |
| `text` | `string` | `""` | — | — | — | The line, resolved in the active locale. |
| `speaker` | `SpeakerView` | `{ id: "", name: "", entity: 0, portrait: "", expression: "", side: "" }` | — | — | — | The bark's speaker (entity: who says it). |
| `voice` | `string` | `""` | — | — | — | Its voice clip (empty: none), played as a conversation's. |
| `voiceLeft` | `number` | `0` | — | — | — | Ticks of its voice still to play. |
| `ticksLeft` | `number` | `0` | — | — | — | Ticks it still shows. |
| `priority` | `number` | `0` | — | — | — | Its priority: only a higher one interrupts it. |

```ogt
struct BarkView {
  doc "A bark being said."
  field entity Entity "Who says it."
  field dialogue String "Its dialogue."
  field bark String "Its id."
  field seq u32 "The line's number: a new number is a new bark."
  field key String "The line's i18n key."
  field args Json "The arguments it was resolved with (evaluated when it was said, world reads included)."
  field text String "The line, resolved in the active locale."
  field speaker SpeakerView "The bark's speaker (entity: who says it)."
  field voice String "Its voice clip (empty: none), played as a conversation's."
  field voiceLeft u32 "Ticks of its voice still to play."
  field ticksLeft u32 "Ticks it still shows."
  field priority i32 "Its priority: only a higher one interrupts it."
}
```

<sub>Module `story` line 187</sub>
