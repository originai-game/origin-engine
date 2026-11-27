<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Loadout"></a>
### `Loadout`

A loadout screen over the scene: the title in large condensed capitals, a category's name over a rule, each item a cut plate of smoked glass (the chosen one ringed in the accent and washed in it), Equipped on a green tag, the price in gold; names wrap on whole words. Approximations: the chosen wash is flat and keeps no rim, the Equipped tag's sheen is dropped, the scroll fade at the list's foot is dropped, and the safe-area padding and short-screen reflow are the caller's.

`import { Loadout } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/loadout.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The page's heading; empty takes the kit's (shooter.loadout.title, Loadout). |
| `categories?` | `ReadonlyArray<LoadoutCategory>` | The slot groups in order. |
| `slots?` | `ReadonlyArray<LoadoutSlot>` | What can be carried. |
| `selected?` | `string` | The slot id selected now (empty: none). |
| `onSelect?` | `(e: UiEvent, id: string) => void` | Choose a slot. |
| `onEquip?` | `(e: UiEvent, id: string) => void` | Equip. |
| `onBack?` | `(e: UiEvent) => void` | Back. |

Each `LoadoutCategory`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label?` | `string` | An empty label takes the kit's string for the id (shooter.loadout.category.<id>). |

Each `LoadoutSlot`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `category` | `string` | A category's id. |
| `name` | `string` | What it is. |
| `icon?` | `string` | An icon of the style's atlas. |
| `detail?` | `string` | A short line under the name (ammunition, damage). |
| `equipped?` | `boolean` | It is carried now. |
| `locked?` | `boolean` | It cannot be chosen yet (shown with its cost). |
| `cost?` | `string` | Its cost text; empty is none. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`.
