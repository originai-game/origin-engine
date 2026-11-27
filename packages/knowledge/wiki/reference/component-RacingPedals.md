<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingPedals"></a>
### `RacingPedals`

The pedals at the bottom left: throttle and brake as two leaning upright wells filled from the foot, green and red, each named under it. Read only.

`import { RacingPedals } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/pedals.tsx`).

| Prop | Type | What |
|---|---|---|
| `throttle` | `number` | The throttle's travel now, 0 to 1. |
| `brake` | `number` | The brake's travel now, 0 to 1. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `braking`, `empty`, `overflow`.
