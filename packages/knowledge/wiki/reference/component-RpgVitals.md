<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgVitals"></a>
### `RpgVitals`

A character's vitals: a round portrait with a level badge and health and mana bars beside it. large is the player's plate, with the figures on the bars; compact is a party row. Read only.

`import { RpgVitals } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/vitals.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The character's name; read out, and shown on the large plate. |
| `portrait` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 is none. |
| `level` | `number` | The level on the badge; 0 hides the badge. |
| `health` | `number` | Health now. |
| `healthMax` | `number` | Most health; the bar shows health over it. |
| `mana` | `number` | Mana now. |
| `manaMax` | `number` | Most mana; 0 hides the mana bar. |
| `plate` | `string` | large (the player's plate, figures on the bars) or compact (a party row). |
| `down` | `boolean` | Out of the fight: the plate greys. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `compact`.
