<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgLootFeed"></a>
### `RpgLootFeed`

The loot feed low on the screen's side: a column of HudLootEntry blocks, newest first, each an icon, the item's name, its rarity on a tag and the count gained on a dark glass with its edge tinted by rarity. Read only.

`import { RpgLootFeed } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/loot-feed.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; rarity: string; label: string; quantity: number; currency: boolean; fading: boolean }>` | What the player just looted, newest first, keyed rows of { id, name, icon, glyph, rarity, label, quantity, currency, fading }: icon a texture (0: the glyph, an icon name, in its place); rarity common, uncommon, rare, epic or legendary (the edge's colour, on data-rarity; an epic or legendary item's name is inked too); label the rarity as the game words it (Epic) on a tag, empty for none; quantity the count gained (0 hides it), written ×3, or +1,240 when currency is true (gold); fading an old row on its way out (dimmed). A new row slides in from the screen's edge; the game trims old rows. |

Fixtures: `default`, `rare`, `overflow`.
