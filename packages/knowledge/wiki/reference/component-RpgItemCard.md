<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgItemCard"></a>
### `RpgItemCard`

One item in detail: its art large over a glow of its rarity, the rarity and kind as a kicker in the rarity's ink over the name, the headline figure large with its change against what is equipped, the other figures as rows (this, change, now), the lore under a rule of the rarity, and up to two buttons, with the reason the main one is refused in words. The game words every figure and decides the trend; the card reports equip (the main button, X or pad Y) and discard (the second button). It fills the height it is given: the figures and the lore scroll when needed; the buttons stay at its foot; the art and the name scroll with the figures. Narrow, the art sits in a socket beside the name and the rows drop the equipped column.

Dropped against the .svelte: the art's drop-shadow filter and the hero glow's radial shape (a linear wash stands in; filters and radial gradients have no surface here).

`import { RpgItemCard } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/item-card.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The item's name. |
| `icon` | `string` | Its icon in the style's or the game's atlas, drawn large at the top. |
| `art` | `Texture \| 0` | A painted picture of the item (og.load.texture) shown large in place of its icon; 0 shows the icon. |
| `rarity` | `string` | common, uncommon, rare, epic, legendary or mythic; the name and the art take its ink. |
| `kind` | `string` | The rarity and kind line as the game words it (Legendary · Longsword), a kicker over the name in the rarity's ink. |
| `line?` | `string` | A line under the name as the game words it (One-handed · Item level 60); empty hides it. |
| `lead` | `ReadonlyArray<{ id: string; icon: string; label: string; value: string; delta: string; trend: string; current: string }>` | The headline figure, zero or one row of { id, icon, label, value, delta, trend, current }: trend is up, down or none; delta and current are text, empty hides them. |
| `stats` | `ReadonlyArray<{ id: string; icon: string; label: string; value: string; delta: string; trend: string; current: string }>` | The other figures, keyed rows of { id, icon, label, value, delta, trend, current }. |
| `lore` | `string` | The flavour text; empty hides it. |
| `primary` | `string` | The main button's text (Equip, Use); empty hides it. |
| `secondary` | `string` | The second button's text (Drop, Sell); empty hides it. |
| `canPrimary?` | `boolean` | The main button is enabled. |
| `canSecondary?` | `boolean` | The second button is enabled. |
| `reason?` | `string` | Why the main button is refused, as the game words it (Requires level 40); shown in red over the buttons while canPrimary is false. |
| `onDiscard?` | `(e: UiEvent, key?: string) => void` | The discard event, with the row's key where it comes from a row. |
| `onEquip?` | `(e: UiEvent, key?: string) => void` | The equip event, with the row's key where it comes from a row. |

Pack slots: `Card.rarity`, `Card.rarity.epic`, `Card.rarity.legendary`, `Card.rarity.mythic`, `Card.rarity.rare`, `Card.rarity.uncommon`, `Panel.frame`.

Fixtures: `default`, `plain`, `refused`.

Actions: `details` (Equip).
