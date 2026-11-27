<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterDamage"></a>
### `ShooterDamage`

Damage the player deals, beside the reticle: the newest hit's figure largest with its word (Headshot) over it, the older ones stepping down and away from the aim. Read only. The rows' pop-in and fade-out transitions have no TSX surface and are dropped.

`import { ShooterDamage } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/damage.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<DamageHit>` | The player's latest hits, newest first. The newest stands largest beside the reticle and the older ones step down and away (a phone shows the newest alone); the game trims the list (three or so) and a new row pops in. Read only. |

Each `DamageHit`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `amount` | `number` | The damage dealt. |
| `kind?` | `string` | normal, headshot, shield (it struck a shield) or kill. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `shield`, `kill`, `overflow`.
