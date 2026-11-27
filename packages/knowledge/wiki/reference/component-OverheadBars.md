<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-OverheadBars"></a>
### `OverheadBars`

A health bar over every unit in the side's colour with a draining trail ghost and an optional thin resource line grouped under one world anchor. Read only.

`import { OverheadBars } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/overhead-bars.tsx`).

| Prop | Type | What |
|---|---|---|
| `bars?` | `ReadonlyArray<OverheadBar>` | The units' bars, keyed rows of { id, entity, at, value, max, side, resource, resourceMax }: entity the unit by its bits (0: at, a world point x y z); value its health out of max; side whose it is, own (green), ally (blue), enemy (red) or neutral (yellow); resource out of resourceMax a thin line under the health (mana, energy), resourceMax 0 draws none. |
| `height?` | `number` | World units over each unit's point (its head). |
| `scaleAt?` | `number` | Distance at which bars are drawn at scale 1. |
| `scaleMin?` | `number` | Minimum world-distance scale. |
| `scaleMax?` | `number` | Maximum world-distance scale. |
| `maxDistance?` | `number` | Maximum world distance before hiding. |

Each `OverheadBar`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `entity` | `number` | The unit, by its bits (0: at, a world point x y z). |
| `at` | `string` | A world point x y z without an entity. |
| `value` | `number` | Its health out of max. |
| `max` | `number` |  |
| `side?` | `string` | Whose it is: own (green), ally (blue), enemy (red) or neutral (yellow). |
| `resource?` | `number` | A thin resource line under the health (mana, energy). |
| `resourceMax?` | `number` |  |

Pack slots: `Bar.fill`, `Bar.ghost`, `Bar.trail`.

Fixtures: `default`, `sides`, `low`.
