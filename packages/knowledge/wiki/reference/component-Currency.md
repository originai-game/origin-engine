<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Currency"></a>
### `Currency`

A readout, not a button. A game can put it beside a separately named purchase control; Currency never invents an economy or sends an intent. A chip: the resource's icon in a socket, its name over the amount, and beside the amount its capacity, its rate and Full. It is as wide as what it holds.

`import { Currency } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/currency.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | Resource name; empty uses Currency. |
| `icon` | `string` | Atlas icon; empty omits it. |
| `amount` | `number` | Amount held; zero remains visible. Ignored when valueText is supplied. |
| `valueText` | `string` | Game-formatted amount; empty uses the compact amount. |
| `limit` | `string` | Game-formatted capacity; empty omits it. The game decides whether the store is full. |
| `rate` | `string` | Optional income or drain, including its sign and unit. |
| `full?` | `boolean` | At capacity; reads Full in words. |
| `loading?` | `boolean` | Awaiting the amount; hides stale figures and reads Loading. |
| `unavailable?` | `boolean` | The balance could not be read; hides stale figures and reads Unavailable. |

Pack slots: `Count.roll`, `Plate.frame`.

Fixtures: `default`, `empty`, `full`, `loading`, `unavailable`, `overflow`.
