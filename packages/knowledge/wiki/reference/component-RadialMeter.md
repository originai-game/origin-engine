<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RadialMeter"></a>
### `RadialMeter`

A stamina wheel: up to three rings that drain and fill, red once exhausted until full again, faded out when the game hides it. Read only.

`import { RadialMeter } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/radial-meter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value?` | `number` | The value in wheels: 1 is one full wheel, up to 3. |
| `wheels?` | `number` | Wheels (an extended stamina's second and third ring), 1 to 3. Default 1. |
| `exhausted?` | `boolean` | Ran out: red until full again. |
| `hidden?` | `boolean` | Faded out (the game hides it once it has been full a while). |
| `name?` | `string` | What a screen reader calls it; empty is Stamina. |

Pack slots: `Radial.box`, `Radial.fill`, `Radial.fill.exhausted`, `Radial.sheen`, `Radial.track`.

Fixtures: `default`.
