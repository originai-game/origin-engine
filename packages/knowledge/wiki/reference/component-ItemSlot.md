<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ItemSlot"></a>
### `ItemSlot`

Reports choose. In a keyed collection the intent keeps its enclosing row key; otherwise the game's stable component instance identifies it.

`import { ItemSlot } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/item-slot.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The item name, also its accessible name and tooltip. Empty slots use the localized empty-slot name. |
| `icon?` | `string` | An atlas icon. Empty is no item art. |
| `count?` | `number` | The quantity; a badge appears above 1. Default 1. |
| `rarity?` | `string` | The item's rarity: common, uncommon, rare, epic, legendary or mythic; a pack selects data-rarity. Default "common". |
| `cooldown?` | `number` | Seconds remaining, supplied by the game. A cooling item can still be selected for inspection. Default 0. |
| `cooldownMax?` | `number` | The cooldown's full duration; the overlay shows the remaining fraction. Default 1. |
| `keyLabel?` | `string` | A shortcut label; empty hides it. |
| `selected?` | `boolean` | The game-selected slot, exposed as aria-pressed. Default false. |
| `disabled?` | `boolean` | Unavailable; selection is disabled. Default false. |
| `locked?` | `boolean` | Locked; shown in words and selection is disabled. Default false. |
| `loading?` | `boolean` | Waiting for item data; selection is disabled. Default false. |
| `empty?` | `boolean` | An empty slot. It can be selected so the game can offer an equip action. Default false. |
| `showName?` | `boolean` | Show the wrapping item name below the slot. Default false. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Slot.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `selected`, `locked`, `disabled`, `loading`, `empty`.
