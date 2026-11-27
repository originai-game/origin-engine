<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgEffects"></a>
### `RpgEffects`

The buffs and debuffs on the leader, under the vitals: a row of HudEffect tiles. The row only spaces them; a game that wants another arrangement lays the tile out itself. The game owns every timer; read only.

`import { RpgEffects } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/effects.tsx`).

| Prop | Type | What |
|---|---|---|
| `effects?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; remaining: number; label: string; harmful: boolean }>` | The buffs and debuffs on a character in the game's order, keyed rows of { id, name, icon, glyph, remaining, label, harmful }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); remaining the share of its time still to run, 0 to 1 (a dark radial sweep over the art; negative: no timer); label its time or stacks as the game words it, on a tag (0:12, ×3), empty for none; harmful a debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `untimed`, `expiring`, `overflow`.
