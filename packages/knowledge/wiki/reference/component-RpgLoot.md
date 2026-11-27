<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgLoot"></a>
### `RpgLoot`

What a chest, a body or a reward holds: a band with the source's name and how many things it holds, one row per item (its icon in a socket and its rarity on the row's edge, the name in that colour over what it is and, when the game refuses it, why in words; the count on the name's line and Take), and Close beside Take all. The game decides what can be taken; the panel reports take (an item's key), takeAll and close (Close, or Escape and pad B anywhere in it). The rows scroll inside the height given.

Dropped against the .svelte: the socket's rarity rim (the rarity tints the socket's ground instead; a box shadow cannot follow the cut shape), the art's drop-shadow filter, and the focused row's wash gradient and inset rim (a flat wash and the leading edge stand in; a cut element takes one bg colour and no box shadow).

`import { RpgLoot } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/loot.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The source's name; empty uses Loot. |
| `items?` | `ReadonlyArray<{ id: string; name: string; icon: string; count: number; description: string; available: boolean; reason: string; rarity: string }>` | Keyed { id, name, icon, count, description, available, reason, rarity }. rarity is common, uncommon, rare, epic, legendary or mythic (empty is common): the row's edge, the icon's socket and the name take its colour. The game supplies legality/capacity and removes collected rows after take or takeAll. Default []. |
| `canTakeAll?` | `boolean` | The game permits taking the displayed items together. Empty/loading/busy never sends takeAll. Default false. |
| `loading?` | `boolean` | Rewards are loading; existing items remain visible but cannot be collected. Default false. |
| `busy?` | `boolean` | A collection request is pending; disables take and takeAll to prevent duplicate requests. Default false. |
| `error?` | `string` | The game's collection failure, such as a full inventory; empty hides it. |
| `onClose?` | `(e: UiEvent, key?: string) => void` | The close event, with the row's key where it comes from a row. |
| `onTake?` | `(e: UiEvent, key?: string) => void` | The take event, with the row's key where it comes from a row. |
| `onTakeAll?` | `(e: UiEvent, key?: string) => void` | The takeAll event, with the row's key where it comes from a row. |

Pack slots: `List.cascade`, `List.item`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `empty`, `loading`, `busy`, `error`.

Actions: `back` (Close).
