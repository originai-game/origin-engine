<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Pips"></a>
### `Pips`

A counted resource on one line: its name, then one icon per unit. An empty unit is its socket; a full one is cut to its fraction and wears the tone. Unavailable, the units lose their colour and dim, full ones still brighter than their sockets.

`import { Pips } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/pips.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The resource's name; empty uses Resource. |
| `icon?` | `string` | The atlas icon for a unit: heart, bullet, bolt or a game's own icon. Default "heart". |
| `items?` | `ReadonlyArray<{ id: string; value: number; label?: string }>` | Keyed units { id, value, label }: value is the filled fraction, 0 to 1; label names the unit for accessibility. The game supplies capacity, ordering and fractional units, with no fixed maximum. |
| `loading?` | `boolean` | Waiting for resource data; units are replaced by Loading. Default false. |
| `disabled?` | `boolean` | The resource is unavailable; its values remain readable. Default false. |
| `tone?` | `string` | The full units' colour: empty is the style's pip colour (a life's red); success, warn, danger, info or primary names another. |
| `class?` | `string` |  |

Pack slots: `Pips.fill`, `Pips.fill.info`, `Pips.fill.primary`, `Pips.fill.success`, `Pips.fill.warn`, `Pips.label`, `Pips.pip`, `Pips.units`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `full`, `empty`, `depleted`, `loading`, `disabled`.
