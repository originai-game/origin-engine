<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="record-FilmSegment"></a>

### `og.FilmSegment` (event)

A film began a node's segment.

Token `og.FilmSegment`, registered under the schema name `og.FilmSegment`; its value type is `FilmSegment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `film` | `Entity` | `0` | — | — | — | The film. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `segment` | `string` | `""` | — | — | — | Its segment's asset id (at most 63 bytes). |

```ogt
event FilmSegment {
  doc "A film began a node's segment."
  field film Entity "The film."
  field node String "The node (at most 63 bytes)."
  field segment String "Its segment's asset id (at most 63 bytes)."
}
```

<sub>Module `film` line 170</sub>
