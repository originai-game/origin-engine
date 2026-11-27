<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CardPiles"></a>
### `CardPiles`

The draw, discard and exhaust piles as buttons: a stack of card backs (one when it is empty) with the pile's icon on top, the game's count on a tag at its corner and a label under it. A pile at 0 carries data-empty; one the game will not open is disabled. Reports open (a pile's key). The piles sit in a row spread across the width they are given; a page that wants the draw pile in one corner and the discard in the other mounts two.

`import { CardPiles } from "@origingame/engine/ui/components"` (kit `card`, `packages/engine/src/ui/components/card/piles.tsx`).

| Prop | Type | What |
|---|---|---|
| `piles?` | `ReadonlyArray<{ id: string; kind: string; label: string; icon: string; count: number; enabled: boolean }>` | The piles in the game's order, keyed rows of { id, kind, label, icon, count, enabled }: kind is draw, discard, exhaust or the game's own word (it reaches the pile as data-kind); count the game's number; enabled false when the game will not open it. |
| `label` | `string` | What a screen reader calls the group; empty is Piles. |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `empty`, `overflow`.
