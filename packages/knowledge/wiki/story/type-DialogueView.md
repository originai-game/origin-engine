<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-DialogueView"></a>

### `DialogueView` (struct)

What a conversation shows now: the line in full (the dialogue box paces its own reveal), its speaker and the choices, if it waits on them.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |
| `node` | `string` | `""` | — | — | — | The node it is in. |
| `seq` | `number` | `0` | — | — | — | Counts lines shown across the story: a new number is a new line, even when a node repeats. |
| `seen` | `boolean` | `false` | — | — | — | The line had been read before it was shown (skip-read, a dimmed backlog); og.story.reset keeps what was read. |
| `key` | `string` | `""` | — | — | — | The line's i18n key. |
| `args` | `unknown` | `null` | — | — | — | The arguments it was resolved with (evaluated when it was shown). |
| `text` | `string` | `""` | — | — | — | The line, resolved in the active locale. |
| `voice` | `string` | `""` | — | — | — | Its voice clip's asset path (empty: none). og.story plays it on its own entity (an og.AudioSource on the sfx bus) from the tick after the line is shown, and stops it when the line moves on. |
| `voiceTicks` | `number` | `0` | — | — | — | The voice's length in ticks (0: no voice, or one that did not load). |
| `voiceLeft` | `number` | `0` | — | — | — | Ticks of the voice still to play: voiceTicks until it starts, 0 once it has ended (advance then, for auto-advance). |
| `speaker` | `SpeakerView` | `{ id: "", name: "", entity: 0, portrait: "", expression: "", side: "" }` | — | — | — | Who says it. |
| `shot` | `string` | `""` | — | — | — | The camera shot it is shown in: its `shot`, else its node's (a virtual camera's entity name or a cast role); empty: none. While it shows, og.story makes that og.VirtualCamera og.CameraBrain.solo: a name is the entity of that name, a role the cast entity's own og.VirtualCamera, else the first one whose lookAt is that entity. When no running conversation's line has a shot the game's own solo is handed back. |
| `choices` | `readonly ChoiceView[]` | `[]` | — | — | — | The choices, when it waits on them; empty while a line waits for og.story.advance. |
| `timer` | `TimerView \| null` | `null` | — | — | — | The choice timer, while it runs. |
| `lastCheck` | `RollView \| null` | `null` | — | — | — | The last active check this conversation rolled. |
| `ended` | `boolean` | `false` | — | — | — | The conversation has ended (the view keeps its last line). |

```ogt
struct DialogueView {
  doc "What a conversation shows now: the line in full (the dialogue box paces its own reveal), its speaker and the choices, if it waits on them."
  field conversation u32 "The conversation."
  field node String "The node it is in."
  field seq u32 "Counts lines shown across the story: a new number is a new line, even when a node repeats."
  field seen bool "The line had been read before it was shown (skip-read, a dimmed backlog); og.story.reset keeps what was read."
  field key String "The line's i18n key."
  field args Json "The arguments it was resolved with (evaluated when it was shown)."
  field text String "The line, resolved in the active locale."
  field voice String "Its voice clip's asset path (empty: none). og.story plays it on its own entity (an og.AudioSource on the sfx bus) from the tick after the line is shown, and stops it when the line moves on."
  field voiceTicks u32 "The voice's length in ticks (0: no voice, or one that did not load)."
  field voiceLeft u32 "Ticks of the voice still to play: voiceTicks until it starts, 0 once it has ended (advance then, for auto-advance)."
  field speaker SpeakerView "Who says it."
  field shot String "The camera shot it is shown in: its `shot`, else its node's (a virtual camera's entity name or a cast role); empty: none. While it shows, og.story makes that og.VirtualCamera og.CameraBrain.solo: a name is the entity of that name, a role the cast entity's own og.VirtualCamera, else the first one whose lookAt is that entity. When no running conversation's line has a shot the game's own solo is handed back."
  field choices List.ChoiceView "The choices, when it waits on them; empty while a line waits for og.story.advance."
  field timer Option.TimerView "The choice timer, while it runs."
  field lastCheck Option.RollView "The last active check this conversation rolled."
  field ended bool "The conversation has ended (the view keeps its last line)."
}
```

<sub>Module `story` line 128</sub>
