<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgCompanions"></a>
### `RpgCompanions`

The leader's companions as small portraits with health bars, a companion under the low share turning to danger. Read only.

`import { RpgCompanions } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/companions.tsx`).

| Prop | Type | What |
|---|---|---|
| `members?` | `ReadonlyArray<{ id: string; name: string; portrait: Texture \| 0; health: number; healthMax: number; ghost: number; mana: number; manaMax: number; key: string; fallen: boolean }>` | The leader's companions in the game's order, keyed rows of { id, name, portrait, health, healthMax, ghost, mana, manaMax, key, fallen }: portrait a texture (0: the person glyph); ghost the health before the latest loss; manaMax 0 hides the mana bar; key the input that switches to them as the game names it (2), on a key cap at the plate's end, empty for none; fallen out of the fight (on data-fallen: the plate dims, the portrait greys and a Fallen tag stands beside the name). |
| `low?` | `number` | The share of health under which a companion is low: the fill turns to danger and the rim glows (0.25). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `fallen`, `critical`, `overflow`.
