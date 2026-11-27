<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-NodeGraph"></a>
### `NodeGraph`

A graph on the HUD's smoked glass: links as lines (lit where taken) under nodes on plates, each captioned beneath on a chip of the well. The view is fitted into the well whole and centred; only where nodes stand and the lines between them scale, a node keeps its size. Zoom and pan are the game's, from the wheel and a drag. Reports pick (a node's id).

`import { NodeGraph } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/node-graph.tsx`).

| Prop | Type | What |
|---|---|---|
| `nodes` | `ReadonlyArray<{ id: string; x: number; y: number; label: string; icon: string; state: string; shape: string; team: string }>` | The nodes: each { id, x, y, label, icon, state, shape, team }, x and y their centres in graph units; state available, locked, active, done or current (a rim and a dim as well as a colour); shape circle, square, hex or diamond; team ally, enemy or empty (pair a side with a shape, so sides are not told by colour alone). |
| `links` | `ReadonlyArray<{ id: string; points: readonly number[]; active: boolean }>` | The links under the nodes: each { id, points, active }, points x, y pairs in graph units (og.flow.chart's link points, or the game's own curve sampled), active a path taken. |
| `view` | `readonly [number, number, number, number]` | The graph's extent in graph units, [x, y, width, height]: it is fitted into the well whole, scaled uniformly and centred. |
| `zoom?` | `number` | Its zoom about the middle; the game keeps it from onZoom. Default 1. |
| `panX?` | `number` | How far it is panned across and down, px; the game keeps them from onPan. |
| `panY?` | `number` |  |
| `selected?` | `string` | The selected node's id: ringed in the focus colour. |
| `name?` | `string` | What a screen reader calls it; empty is Graph. |
| `onPick?` | `(e: UiEvent, id: string) => void` | A node was pressed (a locked one too: the game says why it is closed). |
| `onZoom?` | `(e: UiEvent, delta: number) => void` | The wheel over the well: `delta` its notches, up negative. |
| `onPan?` | `(e: UiEvent, dx: number, dy: number) => void` | The well was dragged by `dx`, `dy` px since the last move. |

Fixtures: `default`, `zoomed`.

Actions: `pick` (Pick a node), `zoom` (Zoom (wheel)), `pan` (Pan).
