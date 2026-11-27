<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgLeader"></a>
### `RpgLeader`

The party leader's vitals at the screen's top corner, built from the RPG HUD's blocks: a portrait in a large octagon, the name and level, health as a figure over its most, and three bars: a fat health bar that leaves a warm ghost when it drops and turns to danger when low, a mana bar and a stamina bar cut into cells. Read only.

`import { RpgLeader } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/leader.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The leader's name; empty hides it. |
| `level` | `number` | The level beside the name; 0 hides it. |
| `portrait` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 draws the person glyph. |
| `health` | `number` | Health now, shown as a figure over its most and as the fat bar. |
| `healthMax` | `number` | Most health. |
| `ghost` | `number` | Health before the latest loss: a warm ghost behind the fill that drains after it (at or under health: none). |
| `mana` | `number` | Mana now. |
| `manaMax` | `number` | Most mana; 0 hides the mana bar. |
| `stamina` | `number` | Stamina (or energy) now. |
| `staminaMax` | `number` | Most stamina; 0 hides the stamina bar. |
| `segment?` | `number` | Stamina per cell of the stamina bar (one dodge's cost); 0 draws it whole. Default 0. |
| `low?` | `number` | The share of health under which it is low: the fill turns to danger and the bar's rim glows (0.25). Default 0.25. |

Fixtures: `default`, `full`, `low`, `critical`, `empty`, `overflow`.
