<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-DamageDirection"></a>
### `DamageDirection`

Where recent hits came from: a mark at the screen's edge toward each hit. The game adds a hit and removes it when its mark should go. Read only.

`import { DamageDirection } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/damage-direction.tsx`).

| Prop | Type | What |
|---|---|---|
| `hits?` | `ReadonlyArray<DamageDirectionHit>` | Recent hits. The game removes a hit when its mark should go. |

Each `DamageDirectionHit`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `angle` | `number` | In radians from straight ahead, clockwise, relative to the view. |
| `strength?` | `number` | 0 to 1: the arc's opacity and length. |
| `kind?` | `string` | normal, shield (the shield took it) or critical. |

Pack slots: `Hit.arc`, `Hit.arc.…`, `Hit.ring`.

Fixtures: `default`, `crowded`.
