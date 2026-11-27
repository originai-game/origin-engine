<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Compass"></a>
### `Compass`

A compass strip: the cardinal and intercardinal points and ticks sliding under the heading, the heading in degrees on a tab, and the points of interest with their distances. Read only.

`import { Compass } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/compass.tsx`).

| Prop | Type | What |
|---|---|---|
| `heading?` | `number` | The player's heading in radians clockwise from north, 0 to 2π. |
| `pois?` | `ReadonlyArray<CompassPoi>` | Its points. |
| `units?` | `string` | The distance's unit (m). |

Each `CompassPoi`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `bearing` | `number` | In radians clockwise from north (0 to 2π). |
| `distance?` | `number` | In m (negative: none). |
| `kind?` | `string` | objective, ping, enemy, ally, loot or poi. |
| `icon?` | `string` | An icon of the atlas; empty draws a diamond. |
| `label?` | `string` | For a reader. |
| `elevation?` | `"above" \| "below" \| "level"` | Optional relative vertical elevation indicator: above, below, or level. |

Fixtures: `default`.
