<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgCrafting"></a>
### `RpgCrafting`

A crafting station: a band with the station's name, the recipes as rows (the result's icon in a socket, its rarity on the row's edge and in its name, why it cannot be made in words), and beside them (under them when narrow) the selected recipe: its art large over a glow of its rarity, what it does and how many the player holds, the ingredients as have / need with a short one marked in words, the batch stepper with the cost, and Craft. The game decides every requirement; the panel reports choose, increase, decrease and craft, each with the recipe's key.

The hero wears a linear wash of the rarity's colour.

`import { RpgCrafting } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/crafting.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The station name; empty uses Crafting. |
| `recipes?` | `ReadonlyArray<{ id: string; name: string; icon: string; description: string; canCraft: boolean; reason: string; rarity: string; held: string }>` | Keyed { id, name, icon, description, canCraft, reason, rarity, held } recipes. Uncraftable recipes remain selectable for inspection; reason explains the restriction. rarity (common … mythic, empty common) colours the row's edge, the socket and the name; held, how many of the result the player has as the game words it, is shown under the result (empty hides it). Default []. |
| `selected?` | `string` | The selected recipe id, or empty for selection guidance. Clear the selection when removing its recipe. |
| `ingredients?` | `ReadonlyArray<{ id: string; name: string; quantity: string; sufficient: boolean }>` | Requirements for the selected recipe and quantity: keyed { id, name, quantity, sufficient }. The game formats quantity (2 / 3) and computes sufficiency. Default []. |
| `quantity?` | `number` | The game-owned batch size. Increase/decrease intents carry the selected recipe id; the game validates and updates it. Default 1. |
| `canIncrease?` | `boolean` | The game permits a larger batch. Default false. |
| `canDecrease?` | `boolean` | The game permits a smaller batch. Default false. |
| `cost?` | `string` | The complete batch cost as the game words it; empty hides it. |
| `loading?` | `boolean` | Recipe data is loading; existing recipes remain visible but inactive. Default false. |
| `busy?` | `boolean` | A craft is in progress; selection, quantities and crafting are inactive. Default false. |
| `error?` | `string` | The game's failure message; empty hides it without discarding the selection. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onCraft?` | `(e: UiEvent, key?: string) => void` | The craft event, with the row's key where it comes from a row. |
| `onDecrease?` | `(e: UiEvent, key?: string) => void` | The decrease event, with the row's key where it comes from a row. |
| `onIncrease?` | `(e: UiEvent, key?: string) => void` | The increase event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Slot.frame`.

Fixtures: `default`, `empty`, `loading`, `busy`, `missing`.
