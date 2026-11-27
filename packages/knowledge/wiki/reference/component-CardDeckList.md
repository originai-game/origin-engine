<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CardDeckList"></a>
### `CardDeckList`

The deck being built: its name, the size against the game's limit (a meter and the figures, data-over and data-under as well as the ink), one row per card with its cost, name and copies, a minus and a plus per row, and the main button. The game counts, sorts and rules; the list reports choose (a row's key), add and remove (a row's plus or minus, its key; X or pad Y anywhere in the focused row is its minus, so it is silent where the game refuses the removal) and confirm. The rows fill the height given and scroll inside it; the size and the button keep the foot.

`import { CardDeckList } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/deck-list.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The deck's name as the player named it. |
| `entries` | `ReadonlyArray<{ id: string; name: string; cost: string; type: string; rarity: string; copies: number; canAdd: boolean; canRemove: boolean }>` | The deck's cards in the game's order (by cost, then name), keyed rows of { id, name, cost, type, rarity, copies, canAdd, canRemove }: copies the game's count; canAdd and canRemove the game's answers (its copy limit, its deck limit, a card the deck must keep). |
| `total` | `number` | How many cards the deck holds (the game's count). |
| `limit` | `number` | The size the game requires or allows; 0 hides the size line. total above limit reaches the size as data-over, below as data-under. |
| `selected` | `string` | The entry shown in detail; empty is none. |
| `confirmLabel` | `string` | The main button's text (Save deck, Done); empty hides it. |
| `canConfirm?` | `boolean` | The main button is enabled (the game's verdict on the deck). |
| `onAdd?` | `(e: UiEvent, key?: string) => void` | The add event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |
| `onRemove?` | `(e: UiEvent, key?: string) => void` | The remove event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `full`.

Actions: `details` (Remove one).
