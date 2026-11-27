<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterAbilities"></a>
### `ShooterAbilities`

The ability bar at the foot of the screen: each ability in an octagon with a lit edge, its cooldown a dark sweep over the art with the seconds left, its charges on a tag and its key cap under it; the ultimate in a larger hexagon inside a ring that fills with its charge. The game owns every cooldown and charge; read only. Every part is an Ability pack slot (listed below), so a theme sets its size, shape, ring, glow, sweep and type.

`import { ShooterAbilities } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/abilities.tsx`).

| Prop | Type | What |
|---|---|---|
| `slots?` | `ReadonlyArray<AbilitySlotData>` | The abilities and items on the bar in the game's order. The game owns every cooldown and charge; read only. |

Each `AbilitySlotData`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls it (Phase dash). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `key?` | `string` | The input as the game names it (Q), on a key cap under the slot; empty is none. |
| `cooldown?` | `number` | The share of its cooldown still to run, 0 to 1: a dark sweep over the art. |
| `time?` | `string` | The seconds left as the game words it (6); shown over the sweep. |
| `count?` | `number` | The charges or items held, on a tag (×2; 0 hides it). |
| `ready?` | `boolean` | Lit (a lit edge and glow). |
| `disabled?` | `boolean` | Greyed. |
| `ultimate?` | `boolean` | The ultimate: larger, in a ring that fills with its charge. |
| `charge?` | `number` | The ultimate's charge, 0 to 1; its share in figures until it is ready. |

Pack slots: `Ability.box`, `Ability.box.ultimate`, `Ability.chip.charges`, `Ability.chip.key`, `Ability.chip.ready`, `Ability.chip.ult`, `Ability.count`, `Ability.face`, `Ability.face.ultimate`, `Ability.frame`, `Ability.frame.disabled`, `Ability.frame.ready`, `Ability.frame.ultimate`, `Ability.ring`, `Ability.ring.fill`, `Ability.ring.ready`, `Ability.sweep`, `Ability.sweep.ultimate`.

Fixtures: `default`, `ready`, `cooling`, `overflow`.
