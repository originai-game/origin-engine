<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-FilmChartLink"></a>

### `FilmChartLink` (struct)

Where a film node may lead, through logic nodes.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `string` | `""` | — | — | — | The node. |
| `to` | `string` | `""` | — | — | — | The film node it may lead to. |
| `choice` | `number` | `0` | — | — | — | The ChoiceView.index that leads there; -1 for a node without choices. |
| `d` | `string` | `""` | — | — | — | A curve from one to the other in SVG path data, graph units (NodeGraph's link `d`). |

```ogt
struct FilmChartLink {
  doc "Where a film node may lead, through logic nodes."
  field from String "The node."
  field to String "The film node it may lead to."
  field choice i32 "The ChoiceView.index that leads there; -1 for a node without choices."
  field d String "A curve from one to the other in SVG path data, graph units (NodeGraph's link `d`)."
}
```

<sub>Module `film` line 154</sub>
