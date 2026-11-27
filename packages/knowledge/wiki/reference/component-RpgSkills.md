<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgSkills"></a>
### `RpgSkills`

The ability bar at the foot of the screen, a row of HudSlot blocks: the consumables in smaller slots, set apart before the skills, then each skill, and the ultimate, larger, at the end. The slot draws itself (its cooldown, cost, charges and key cap are HudSlot's); the bar only sizes and spaces them. A game that wants another bar lays HudSlot out itself. The game owns every cooldown, cost and charge; read only.

The bar's per-kind sizes (a smaller item slot, a larger ultimate, the compact shrink) are the block's own look now: HudSlot draws one size.

`import { RpgSkills } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/skills.tsx`).

| Prop | Type | What |
|---|---|---|
| `slots?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; key: string; kind: string; cooldown: number; time: string; count: number; ready: boolean; unaffordable: boolean; reason: string; charges: number; chargesMax: number; charge: number }>` | The bar's slots in the game's order, keyed rows of { id, name, icon, glyph, key, kind, cooldown, time, count, ready, unaffordable, reason, charges, chargesMax, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (1, Q), on a key cap under the slot; kind skill, item (a consumable: a smaller slot with its count on a tag, set apart before the skills; a phone leaves items out) or ultimate (a larger hexagon inside a gold ring that fills with charge, 0 to 1, its share in figures until full; on data-kind); cooldown the share still to run, 0 to 1 (a dark radial sweep over the art with a lit leading edge), with time its figure as the game words it (8); count the items held (×5; 0 hides it); ready lit (a lit edge and glow); unaffordable it cannot be paid for (the art greys under a veil and reason, as the game words it (No mana), stands on it); chargesMax 0 hides the charge pips, else charges of them are lit over the slot. |

Fixtures: `default`, `cooling`, `unaffordable`, `overflow`.
