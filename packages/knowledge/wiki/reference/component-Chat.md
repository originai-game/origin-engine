<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Chat"></a>
### `Chat`

A chat panel: channel tabs with unread badges, the message log (system lines, the player's own messages and others' with sender and time, long text wrapping and long words breaking) and an input row. The game owns the channels, the messages, moderation and the clock; the panel reports choosetab (a channel's key), send (the Send button) and onDraft (the field's text, trimmed to maxLength). The log fills the height it is given and scrolls inside it, newest at the foot.

`import { Chat } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/chat.tsx`).

| Prop | Type | What |
|---|---|---|
| `channels` | `ReadonlyArray<{ id: string; label: string; unread: number }>` | The channels, keyed rows of { id, label, unread }: unread the count the game keeps, 0 hides its badge. Empty shows no tab row. |
| `channel` | `string` | The channel shown. |
| `messages` | `ReadonlyArray<{ id: string; kind: string; sender: string; time: string; text: string }>` | The shown channel's messages, oldest first: keyed rows of { id, kind, sender, time, text }: kind player, own (the player's) or system; time as the game formats it; text wraps, and a long word breaks. |
| `draft` | `string` | The message being written, two-way: value={draft} shows it; the field reports every change through onDraft with the text trimmed to maxLength, and the game clears it after a send. |
| `maxLength` | `number` | The longest message the game takes, in characters. |
| `canSend?` | `boolean` | The game takes a message now; false disables the field and Send (muted, slow mode). |
| `notice` | `string` | A line over the field as the game words it (Slow mode: 3 s, You are muted); empty hides it. |
| `onDraft?` | `(e: UiEvent, text: string) => void` | The field's text changed (onDraft), trimmed to maxLength. |
| `onChooseTab?` | `(e: UiEvent, key?: string) => void` | The choosetab event, with the row's key where it comes from a row. |
| `onSend?` | `(e: UiEvent, key?: string) => void` | The send event, with the row's key where it comes from a row. |

Pack slots: `Field.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `disabled`, `overflow`.
