<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DamageNumbers"></a>
### `DamageNumbers`

(No doc comment.)

`import { DamageNumbers } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/damage-numbers.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<DamageHit>` | The hits: each { id, entity, at, amount, color } kept for the rise's length, id unique per hit, entity the unit by its bits (0: at, a world point x y z), color #rrggbb (empty is white). Each new row spawns one number on the GPU that rises and fades. |

Each `DamageHit`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `entity` | `number` | The unit hit, by its bits (0: at, a world point x y z). |
| `at` | `string` | A world point x y z without an entity. |
| `amount` | `number` | The damage, shown as the number. |
| `color?` | `string` | The number's ink as #rrggbb (empty: its kind's). |
| `kind?` | `string` | normal (empty), crit (larger and gold), heal, shield, weak or miss: the theme's look for each (Damage.number.*). |

Pack slots: `Damage.number`, `Damage.number.…`.

Fixtures: `default`.
