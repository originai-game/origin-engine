<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Minimap"></a>
### `Minimap`

A square minimap: a region of the map texture (or a grid without one), markers over it and a zone name under it on a plate. Read only.

`import { Minimap } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/minimap.tsx`).

| Prop | Type | What |
|---|---|---|
| `texture?` | `Texture \| 0` | The map texture (og.load.texture); 0 draws a grid over the stage colour. |
| `u0?` | `number` | The region of the map shown: its left edge, 0 to 1. |
| `v0?` | `number` | Its top edge, 0 to 1 (v down). |
| `u1?` | `number` | Its right edge (1: the whole width). Default 1. |
| `v1?` | `number` | Its bottom edge (1: the whole height). Default 1. |
| `size?` | `number` | Its side in px (160). |
| `markers?` | `ReadonlyArray<MinimapMarker>` | Markers over the map. |
| `label?` | `string` | A zone name under it; empty is none. |

Each `MinimapMarker`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `x` | `number` | Across the shown region, 0 to 1. |
| `y` | `number` | Down the shown region, 0 to 1. |
| `tone?` | `string` | neutral, info, success, warn, danger or accent. |
| `player?` | `boolean` | The player's: the primary, larger and ringed. |

Pack slots: `Plate.frame`.

Fixtures: `default`.
