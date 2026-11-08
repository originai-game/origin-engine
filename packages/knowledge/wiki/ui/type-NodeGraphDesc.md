<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NodeGraphDesc"></a>

### `NodeGraphDesc` (desc struct)

A pan-and-zoom graph of nodes and links: skill and tech trees, run maps, flowcharts.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nodes` | `readonly GraphNode[]` | `[]` | — | — | — | The nodes. |
| `links` | `readonly GraphLink[]` | `[]` | — | — | — | The links, under the nodes. |
| `zoom` | `number` | `1` | — | — | — | Its zoom (keep NodeGraphResult.zoom). |
| `minZoom` | `number` | `0.5` | — | — | — | The least zoom. |
| `maxZoom` | `number` | `2` | — | — | — | The greatest zoom. |
| `nodeSize` | `number \| null` | `null` | — | `cssPx` | — | A node's size at zoom 1; null is 56 (52 on a phone). |
| `curved` | `boolean` | `true` | — | — | — | Links bend as a run map's paths. |
| `labels` | `boolean` | `true` | — | — | — | Names under the nodes; without them no room is kept under a node for one, so the fit fills the window. |
| `selected` | `string` | `""` | — | — | — | The selected node's id: ringed in the focus ink. |
| `zoomInLabel` | `string` | `""` | — | — | — | Zoom in's accessible name. |
| `zoomOutLabel` | `string` | `""` | — | — | — | Zoom out's accessible name. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null is 420 (360 on a phone). |
| `ground` | `Texture \| null` | `null` | — | — | — | A texture tiled over the field's ground (star dust, a grid: light marks on transparent), 256 px a tile, tinted in the ink at 6 % so it reads as a field and never competes with the nodes; under the links, panning with them. Null is the plain ground. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |

```ogt
struct NodeGraphDesc {
  doc "A pan-and-zoom graph of nodes and links: skill and tech trees, run maps, flowcharts."
  desc true
  field nodes List.GraphNode "The nodes."
  field links List.GraphLink "The links, under the nodes."
  field zoom f32 "Its zoom (keep NodeGraphResult.zoom)." {
    default 1
  }
  field minZoom f32 "The least zoom." {
    default 0.5
  }
  field maxZoom f32 "The greatest zoom." {
    default 2
  }
  field nodeSize Option.f32 "A node's size at zoom 1; null is 56 (52 on a phone)." {
    unit cssPx
  }
  field curved bool "Links bend as a run map's paths." {
    default true
  }
  field labels bool "Names under the nodes; without them no room is kept under a node for one, so the fit fills the window." {
    default true
  }
  field selected String "The selected node's id: ringed in the focus ink."
  field zoomInLabel String "Zoom in's accessible name."
  field zoomOutLabel String "Zoom out's accessible name."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height; null is 420 (360 on a phone)."
  field ground Option.Texture "A texture tiled over the field's ground (star dust, a grid: light marks on transparent), 256 px a tile, tinted in the ink at 6 % so it reads as a field and never competes with the nodes; under the links, panning with them. Null is the plain ground."
  field label String "What a screen reader calls it."
}
```

<sub>Module `ui` line 1527</sub>
