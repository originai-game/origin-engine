<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CardCollection"></a>
### `CardCollection`

The card library of a deck builder: a heading, filter tabs and a grid of cards that fills the height it is given and scrolls inside it. The game filters and sorts what it passes; the grid reports add (a card's key: tap, Enter, pad A), inspect (X or pad Y on the focused card) and choosefilter (a filter's key), tabNext and tabPrev (E/Q, bumpers). A card the game will not add never reports add: it stays in reach, so it can be focused and read, reports inspect from a tap too, and says why in the game's words, with a lock when it is not owned and a cross when it is. The core DataGrid is a table of text rows, so the cards are a grid of their own.

`import { CardCollection } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/collection.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The collection's heading (Collection, Library); empty hides the band. |
| `filters` | `ReadonlyArray<{ id: string; label: string; count?: number }>` | The filters the game offers (classes, costs, types), keyed rows of { id, label, count } (count above 0 shows beside the label); empty hides the row. |
| `filter` | `string` | The id of the filter applied; the game filters the cards it passes. |
| `cards` | `ReadonlyArray<{ id: string; name: string; cost: string; icon: string; art: Texture \| 0; type: string; typeLabel: string; rarity: string; text: string; owned: number; inDeck: number; canAdd: boolean; locked: boolean; reason: string }>` | The cards shown, already filtered and sorted by the game: keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, owned, inDeck, canAdd, locked, reason }. owned and inDeck are the game's counts (owned above 1 is shown, inDeck above 0); canAdd is the game's answer (limit, rarity rule, not owned); locked marks a card not owned yet; reason is why the game will not add it, in the game's words (spoken and marked when canAdd is false; empty says Not owned for a locked card, Cannot be added otherwise). |
| `selected` | `string` | The card shown in detail; empty is none. |
| `empty` | `string` | What an empty result says; empty is No cards match. |
| `onAdd?` | `(e: UiEvent, key?: string) => void` | The add event, with the row's key where it comes from a row. |
| `onChoosefilter?` | `(e: UiEvent, key?: string) => void` | The choosefilter event, with the row's key where it comes from a row. |
| `onInspect?` | `(e: UiEvent, key?: string) => void` | The inspect event, with the row's key where it comes from a row. |
| `onTabNext?` | `(e: UiEvent, key?: string) => void` | The tabNext event, with the row's key where it comes from a row. |
| `onTabPrev?` | `(e: UiEvent, key?: string) => void` | The tabPrev event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Tab.indicator`.

Fixtures: `default`, `empty`.

Actions: `tabPrev` (Previous filter), `tabNext` (Next filter), `details` (Inspect).
