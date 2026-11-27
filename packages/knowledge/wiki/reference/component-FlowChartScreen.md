<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-FlowChartScreen"></a>
### `FlowChartScreen`

The story so far over the dimmed flow: a band with Back and the title, the chart of flow nodes (seen ones done, the one playing current, the rest locked), and the picked node on a plate lit in the accent, its words beside Play from here, which drops under them where the plate is narrow.

`import { FlowChartScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-chart.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `nodes` | `ReadonlyArray<{ id: string; x: number; y: number; label: string; icon: string; state: string; shape: string; team: string }>` | og.flow.chart's nodes as NodeGraph's: keyed rows of { id (the node), x, y, label, icon, state, shape, team }. state: current (where the flow is), done (a segment seen), locked (not seen); an ending's shape diamond, a chapter's square, the rest circle. |
| `links` | `ReadonlyArray<{ id: string; points: readonly number[]; active: boolean }>` | og.flow.chart's links as NodeGraph's: keyed rows of { id, points, active }, points the link's FlowChartLink.points, active a choice the player has taken. |
| `view` | `readonly [number, number, number, number]` | The chart's extent, [0, 0, chart.width, chart.height] from og.flow.chart. |
| `zoom` | `number` | NodeGraph's zoom, kept by the game from its zoom and pinch actions. |
| `panX` | `number` | NodeGraph's pan across. |
| `panY` | `number` | NodeGraph's pan down. |
| `selected` | `string` | The node picked (its id); empty is none. |
| `detail` | `string` | What the picked node is, as the game words it (its chapter, or Not seen yet); empty hides the panel. |
| `canPlay` | `boolean` | The picked node can be played from (a chapter the player has saved at). |
| `onPick?` | `(e: UiEvent, id: string) => void` | A node was picked: its id. |
| `onZoom?` | `(e: UiEvent, delta: number) => void` | The wheel over the chart: `delta` its notches, up negative (the game keeps zoom). |
| `onPan?` | `(e: UiEvent, dx: number, dy: number) => void` | The chart was dragged by `dx`, `dy` px (the game keeps the pan). |
| `onPlay?` | `(e: UiEvent) => void` | Play from the picked node. |
| `onBack?` | `(e: UiEvent) => void` |  |

Pack slots: `Screen.frame`.

Fixtures: `default`, `unseen`, `none`.

Actions: `pick` (Pick a node), `play` (Play from here), `back` (Back).
