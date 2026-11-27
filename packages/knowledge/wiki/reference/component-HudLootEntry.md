<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudLootEntry"></a>
### `HudLootEntry`

One row of a HUD's loot feed, a block a feed is built from: an icon, the item's name, its rarity on a tag and the count gained, on a dark glass that fades toward the middle with its edge at the screen's side tinted by rarity; a fading row dims. Read only. The art's cut corner (corner-shape) has no TSX surface: a rounded one stands in.

`import { HudLootEntry } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/loot-entry.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The item's name (Ember Blade). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `rarity?` | `string` | common, uncommon, rare, epic or legendary: the edge's colour (on data-rarity); an epic or legendary item's name is inked too. |
| `label?` | `string` | The rarity as the game words it (Epic), on a tag; empty is none. |
| `quantity?` | `number` | The count gained, written ×3 (or +1,240 when currency is true); 0 hides it. |
| `currency?` | `boolean` | The quantity is money (gold): written as a gain. |
| `fading?` | `boolean` | An old row on its way out: dimmed. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `gold`, `fading`, `empty`, `overflow`.
