<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyMinimap"></a>
### `StrategyMinimap`

The minimap at the bottom left: the whole map square on a frosted plate, your units and buildings, your allies' and your foes' as dots in their colours (a base larger, a resource a gold diamond), the camera's view as a white frame and each alert as an animated ping ring. Supports click/drag camera navigation (`onNavigate`) and pings (`onPing`).

`import { StrategyMinimap } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/minimap.tsx`).

| Prop | Type | What |
|---|---|---|
| `map` | `Texture \| 0` | The whole map's picture (og.load.texture), drawn square; 0 draws the dark well alone. |
| `dots?` | `ReadonlyArray<{ id: string; kind: string; x: number; y: number; base: boolean; label: string }>` | What the minimap shows, keyed rows of { id, kind, x, y, base, label }: kind own, ally, foe or neutral (a resource, a relic; on data-kind, each its colour, a neutral a diamond); x and y on the map, 0 to 1; base marks a building or a town, drawn larger (data-base); label names it for a reader. |
| `viewX` | `number` | The camera's view on the map: its left edge, 0 to 1. |
| `viewY` | `number` | The camera's view: its top edge, 0 to 1. |
| `viewWidth` | `number` | The camera's view: its width, 0 to 1; 0 hides the frame. |
| `viewHeight` | `number` | The camera's view: its height, 0 to 1. |
| `pings?` | `ReadonlyArray<{ id: string; x: number; y: number; label: string }>` | Alerts on the map, keyed rows of { id, x, y, label }: a red ring round the place (x and y, 0 to 1); label names it for a reader. |
| `onNavigate?` | `(e: UiEvent, at: { x: number; y: number }) => void` | Called when the player clicks or drags on the minimap with normalized map coordinates (0 to 1). |
| `onPing?` | `(e: UiEvent, at: { x: number; y: number }) => void` | Called when the player right-clicks or alt-clicks the minimap to place a ping. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `allies`, `empty`, `overflow`.
