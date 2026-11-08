<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-nodeGraph"></a>

### `og.ui.nodeGraph`

A pan-and-zoom graph of nodes and links: skill trees, run maps, flowcharts.

Each node is a circle, rounded square, hexagon, diamond or triangle; a node with a team (a sector map's allied and hostile systems) is filled in its side's colour, a sided node with no icon showing no letters (its shape and colour mark it). Its state shows by its ring and a mark.

Parts: Graph.frame (its ground's texture on <id>/Window/View/Canvas), Graph.node[<state>][<kind>][<shape>][team=<n>][selected], Graph.zoom, Graph.zoomButton. Accessibility: group named label, value the zoom; each node a button named by its label, value its state, selected when selected. Probe ids: <id>/Window/View/Canvas/Node#i, <id>/Window/View/Canvas/Caption#i, <id>/Bar/Zoom/In, <id>/Bar/Zoom/Out.

```ts
og.ui.nodeGraph(id: string, desc?: NodeGraphDesc): NodeGraphResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `NodeGraphDesc` | yes | Nodes, links and the zoom. |

**Returns** `NodeGraphResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn nodeGraph {
  doc "A pan-and-zoom graph of nodes and links: skill trees, run maps, flowcharts."
  doc "Each node is a circle, rounded square, hexagon, diamond or triangle; a node with a team (a sector map's allied and hostile systems) is filled in its side's colour, a sided node with no icon showing no letters (its shape and colour mark it). Its state shows by its ring and a mark."
  doc "Parts: Graph.frame (its ground's texture on <id>/Window/View/Canvas), Graph.node[<state>][<kind>][<shape>][team=<n>][selected], Graph.zoom, Graph.zoomButton. Accessibility: group named label, value the zoom; each node a button named by its label, value its state, selected when selected. Probe ids: <id>/Window/View/Canvas/Node#i, <id>/Window/View/Canvas/Caption#i, <id>/Bar/Zoom/In, <id>/Bar/Zoom/Out."
  host false
  param id String "Its id."
  param desc NodeGraphDesc "Nodes, links and the zoom."
  returns NodeGraphResult
}
```

<sub>Module `ui` line 3002</sub>
