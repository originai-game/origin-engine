<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgWorldMap"></a>
### `RpgWorldMap`

A map with its markers, a one-line legend and an optional caption. The map's art is the screen: no panel lies over it, only each marker's name on a small cut plate, the legend and the caption. A marker's kind is its shape as well as its colour (a quest a gold diamond, a waypoint a cyan ring, danger a red triangle, the player a cyan disc with an arrow); its name is white at any kind. The chosen marker is ringed and larger. The map is drawn to cover the box it is given, times zoom and moved by panX and panY; markers sit at fractions of it, keep their size at any zoom and report choose (a marker's key). The player's marker is not a button target. The game owns the zoom and the pan: the map reports zoom (the wheel, with deltaY), pinch (two touches, with scale), and grab, pan and release (a press, its moves and its end, each with x and y in the map's box), and draws what the game sets.

`import { RpgWorldMap } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/world-map.tsx`).

| Prop | Type | What |
|---|---|---|
| `map` | `Texture \| 0` | The map's texture (og.load.texture); markers sit on it in fractions of its size. |
| `markers?` | `ReadonlyArray<{ id: string; kind: string; x: number; y: number; label: string }>` | Keyed rows of { id, kind, x, y, label }: kind is quest, waypoint, danger or player; x and y 0 to 1 across and down the map. |
| `selected` | `string` | The marker selected; empty is none. |
| `zoom?` | `number` | How far the map is zoomed in: 1 (and 0, and when left out) fits it to the box, 2 shows it twice the size. The game keeps it, from the zoom and pinch events, within its own limits. |
| `panX?` | `number` | How far the zoomed map is moved across, CSS px (0, and when left out, pins its start corner). The game keeps it, from grab, pan and release. |
| `panY?` | `number` | How far the zoomed map is moved down, CSS px; 0 when left out. |
| `legend?` | `ReadonlyArray<{ id: string; kind: string; label: string }>` | The legend, keyed rows of { id, kind, label }, one line along the map's foot; empty hides it. |
| `here` | `string` | A caption pinned to the map's top corner (Current location); empty hides it. |
| `onZoom?` | `(e: UiEvent) => void` | The wheel zooms: the game reads the wheel's delta and keeps the zoom within its limits. |
| `onGrab?` | `(e: UiEvent) => void` | A press on the map begins a grab. |
| `onPan?` | `(e: UiEvent) => void` | A grabbed map's moves pan it, each with x and y in the map's box. |
| `onRelease?` | `(e: UiEvent) => void` | The grab's end releases it. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a marker; the marker's key. The player's marker is not a button target. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `places`.
