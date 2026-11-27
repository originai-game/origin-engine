<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Counter"></a>
### `Counter`

A number that rolls to its value as it changes (`Count.roll`). When `from` is given, it draws `from` on its first tick and rolls to `value` on the next, deterministically on the world clock.

`import { Counter } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/counter.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The number it shows. |
| `format?` | `"grouped" \| "compact" \| "percent" \| undefined` | How it writes the number: grouped (1,234, the default), compact (1.2K) or percent (0 to 1). |
| `decimals?` | `number \| undefined` | Digits after the point. |
| `from?` | `number \| string \| undefined` | Where its first count starts (a score counting up as its screen opens); rolls on the next tick. |
| `duration?` | `number \| string \| undefined` | How long a count takes, in ms or CSS time; overrides Count.roll's duration. |
| `class?` | `string \| undefined` |  |

Pack slots: `Count.roll`, `Counter.value`.
