<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterVitals"></a>
### `ShooterVitals`

The player's vitals at the screen's bottom corner: the portrait in an octagon with a lit edge, the name and level, health as a figure over its most, a shield bar cut into cells over a fat health bar that leaves a warm ghost when it drops and turns to danger when low. The game owns every value; read only. Approximations: the portrait's octagon cut is rounded and the cells' mask is hairline dividers.

`import { ShooterVitals } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/vitals.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The player's name as the game words it; empty hides it. |
| `level?` | `number` | The player's level beside the name; 0 hides it. |
| `portrait?` | `Texture \| 0` | The player's portrait texture (og.load.texture); 0 shows the person glyph in its place. |
| `health` | `number` | Health now. |
| `healthMax?` | `number` | Most health. Default 100. |
| `ghost?` | `number` | Health before the latest loss: the bar keeps a warm ghost to it that drains; at or under health shows none. |
| `shield?` | `number` | Shield now, a cut bar over the health bar. |
| `shieldMax?` | `number` | Most shield; 0 hides the shield bar. |
| `segment?` | `number` | Shield per cell of the shield bar (25: four cells for 100). Default 25. |
| `low?` | `number` | The fraction of most health at or under which the health bar turns to danger (0.3). Default 0.3. |

Pack slots: `Bar.fill`, `Bar.trail`, `Plate.frame`.

Fixtures: `default`, `full`, `low`, `critical`, `noshield`, `overflow`.
