<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgConversationBacklog"></a>
### `RpgConversationBacklog`

The conversation's history: the title and Close, then the lines as cut cards — the speaker in gold small capitals, narration quiet and in italics, the line replaying washed gold with a gold bar — with Load earlier lines over them and Replay or Stop replay under a replayable one. Reports close, older, replay and stop (an entry's key). Answers the back action.

`import { RpgConversationBacklog } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/conversation-backlog.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The history title; empty uses Conversation history. |
| `lines?` | `ReadonlyArray<{ id: string; speaker: string; text: string; canReplay: boolean }>` | Chronological keyed { id, speaker, text, canReplay } entries. Empty speaker is narration. The game supplies history and audio availability; lines keep their original wording. Default []. |
| `playing?` | `string` | The entry currently replaying; empty means no replay. The game owns playback and reports completion by clearing this id. |
| `hasOlder?` | `boolean` | Earlier history is available; older asks the game to prepend it, preserving row ids. Default false. |
| `loading?` | `boolean` | History is loading; earlier-line and replay actions are disabled, close remains available. Default false. |
| `onClose?` | `(e: UiEvent, key?: string) => void` | The close event, with the row's key where it comes from a row. |
| `onOlder?` | `(e: UiEvent, key?: string) => void` | The older event, with the row's key where it comes from a row. |
| `onReplay?` | `(e: UiEvent, key?: string) => void` | The replay event, with the row's key where it comes from a row. |
| `onStop?` | `(e: UiEvent, key?: string) => void` | The stop event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `empty`, `loading`, `playing`.

Actions: `back` (Close history).
