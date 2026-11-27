<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Meter"></a>
### `Meter`

A track whose fill wears its value's region: good, middling or poor.

`import { Meter } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/meter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` |  |
| `min?` | `number \| undefined` | The range; default 0 to 1. |
| `max?` | `number \| undefined` |  |
| `region?` | `"optimum" \| "suboptimum" \| "poor" \| undefined` | Where the value lies against the meter's optimum, as the game judges it: the fill wears its colour. |
| `class?` | `string \| undefined` |  |
| `label?` | `string \| undefined` |  |

Pack slots: `Bar.fill`.

Fixtures: `optimum`, `suboptimum`, `poor`.
