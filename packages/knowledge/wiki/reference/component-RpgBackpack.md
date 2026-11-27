<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgBackpack"></a>
### `RpgBackpack`

A bag of items: a band with a kicker, the title and the purse, category tabs between the action's key caps, how full the bag is, a grid of item tiles and a footer with the carried weight. The game owns what a category holds, the order and every number; the bag reports choose (an item's key), choosetab (a category's key), tabNext and tabPrev. A tile drags onto another (a locked one stays put) and the bag reports move with { dragged, target }, the two items' keys. A tile shows its rarity as its bar along the foot and its tags; its stack, Equipped, New, Locked and Broken sit in its corners, each named in words for a screen reader, Broken also on the tile. The selected tile wears corner brackets. The grid fills the height it is given and scrolls inside it.

`import { RpgBackpack } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/backpack.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The bag's heading (Backpack, Stash, a chest's name). |
| `kicker` | `string` | A small line over the title naming the screen (Inventory); empty hides it. |
| `tabs` | `ReadonlyArray<{ id: string; label: string; count: number }>` | The categories, a keyed collection of { id, label, count }: count, how many items the category holds, is shown beside its label above 0. |
| `tab` | `string` | The id of the category shown. |
| `items` | `ReadonlyArray<{ id: string; name: string; icon: string; count: number; rarity: string; equipped: boolean; fresh: boolean; locked: boolean; broken: boolean }>` | The items shown, in the game's order: each { id, name, icon, count, rarity, equipped, fresh, locked, broken }. rarity is common, uncommon, rare, epic, legendary or mythic; count above 1 is shown; fresh marks an unseen item, locked one the game will not let go, broken one it will not equip. |
| `selected` | `string` | The selected item's id; empty is none. |
| `capacity` | `string` | How full the bag is, as the game words it (30 / 36 slots); empty hides it. |
| `weight` | `string` | The load as the game words it (92.4 / 120); empty hides it. |
| `load` | `number` | The load from 0 to 1 for its bar; above 1 is overburdened. |
| `gold` | `string` | The purse as the game formats it; empty hides it. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onChoosetab?` | `(e: UiEvent, key?: string) => void` | The choosetab event, with the row's key where it comes from a row. |
| `onTabNext?` | `(e: UiEvent, key?: string) => void` | The tabNext event, with the row's key where it comes from a row. |
| `onTabPrev?` | `(e: UiEvent, key?: string) => void` | The tabPrev event, with the row's key where it comes from a row. |
| `onMove?` | `(e: UiEvent, move: { dragged: string; target: string }) => void` | A tile dropped on another: the dragged item's key and the target's. A locked tile is not picked up; the dragged tile goes home, and the game reorders, swaps or stacks as it sees fit. |

Pack slots: `Chip.frame`, `Panel.frame`, `Slot.frame`, `Tab.count`.

Fixtures: `default`, `empty`, `over`, `overflow`.

Actions: `tabPrev` (Previous category), `tabNext` (Next category).
