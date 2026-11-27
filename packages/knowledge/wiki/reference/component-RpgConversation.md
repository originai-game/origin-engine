<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgConversation"></a>
### `RpgConversation`

A conversation across the foot of the screen: the speaker's portrait, name and line, and the player's numbered replies. Reports reply (a reply's key) and next (the Continue button, or confirm, while there are no replies). With a rate the line shows a few characters at a time by the world's tick (always whole to a screen reader) and reports revealed once it is whole; the game decides what a press before that does.

The line types out under reduced motion too: the TSX UI cannot read the setting yet.

`import { RpgConversation } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/conversation.tsx`).

| Prop | Type | What |
|---|---|---|
| `speaker` | `string` | The speaker's name; empty is narration. |
| `portrait` | `Texture \| 0` | The speaker's portrait (og.load.texture); 0 hides it. |
| `line` | `string` | What is said now. |
| `rate?` | `number` | Characters of the line shown per second, by the world's tick, starting again with each new line; 0 (and when left out) shows the line at once, and setting 0 finishes a line early. A screen reader always reads the whole line. |
| `replies` | `ReadonlyArray<{ id: string; number: number; text: string; used: boolean }>` | The player's replies, keyed rows of { id, number, text, used }: number the figure before it; used greys a reply already chosen. Empty shows a Continue button. |
| `selected` | `string` | The reply the focus rests on; empty is none. |
| `showControls?` | `boolean` | Shows optional History, Skip and Auto buttons; hidden in existing compact layouts. Default false. |
| `canBacklog?` | `boolean` | The game permits opening conversation history. Default true. |
| `canSkip?` | `boolean` | The game permits requesting a skip; the component never advances dialogue itself. Default false. |
| `canAuto?` | `boolean` | The game permits requesting an automatic-advance setting change. Default true. |
| `auto?` | `boolean` | The game's automatic-advance setting; toggleAuto requests a change without advancing dialogue. Default false. |
| `onBacklog?` | `(e: UiEvent, key?: string) => void` | The backlog event, with the row's key where it comes from a row. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onReply?` | `(e: UiEvent, key?: string) => void` | The reply event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |
| `onToggleAuto?` | `(e: UiEvent, key?: string) => void` | The toggleAuto event, with the row's key where it comes from a row. |
| `onRevealed?` | `(e: UiEvent) => void` | The revealed event: the line is whole, once per line that was revealed at a rate. |

Pack slots: `Button.plate`, `Chip.frame`, `Plate.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `line`, `controls`, `typing`, `auto`, `disabled`.

Actions: `confirm` (Continue).
