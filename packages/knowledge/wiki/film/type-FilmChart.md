<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-FilmChart"></a>

### `FilmChart` (struct)

A dialogue's film nodes laid out as a flowchart, columns by depth from the start: what a flowchart screen draws (the built-in NodeGraph), the game marking what the player has seen.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nodes` | `readonly FilmChartNode[]` | `[]` | — | — | — | Its film nodes, in file order. |
| `links` | `readonly FilmChartLink[]` | `[]` | — | — | — | Where each leads. |
| `width` | `number` | `0` | — | — | — | Its extent across, graph units (NodeGraph's view is `[0, 0, width, height]`). |
| `height` | `number` | `0` | — | — | — | Its extent down. |

```ogt
struct FilmChart {
  doc "A dialogue's film nodes laid out as a flowchart, columns by depth from the start: what a flowchart screen draws (the built-in NodeGraph), the game marking what the player has seen."
  field nodes List.FilmChartNode "Its film nodes, in file order."
  field links List.FilmChartLink "Where each leads."
  field width f32 "Its extent across, graph units (NodeGraph's view is `[0, 0, width, height]`)."
  field height f32 "Its extent down."
}
```

<sub>Module `film` line 163</sub>
