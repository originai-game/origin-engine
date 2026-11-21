<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-FilmChartNode"></a>

### `FilmChartNode` (struct)

A film node on og.film.chart's layout.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `node` | `string` | `""` | — | — | — | Its title. |
| `segment` | `string` | `""` | — | — | — | Its segment's asset id. |
| `chapter` | `string` | `""` | — | — | — | Its `og-film-chapter`, if any. |
| `ending` | `string` | `""` | — | — | — | Its `og-film-ending`, if any. |
| `depth` | `number` | `0` | — | — | — | Its column: the fewest film nodes from the start before it (unreached nodes after the deepest). |
| `x` | `number` | `0` | — | — | — | Its centre across, graph units (NodeGraph's). |
| `y` | `number` | `0` | — | — | — | Its centre down. |

```ogt
struct FilmChartNode {
  doc "A film node on og.film.chart's layout."
  field node String "Its title."
  field segment String "Its segment's asset id."
  field chapter String "Its `og-film-chapter`, if any."
  field ending String "Its `og-film-ending`, if any."
  field depth u32 "Its column: the fewest film nodes from the start before it (unreached nodes after the deepest)."
  field x f32 "Its centre across, graph units (NodeGraph's)."
  field y f32 "Its centre down."
}
```

<sub>Module `film` line 144</sub>
