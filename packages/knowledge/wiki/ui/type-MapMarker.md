<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MapMarker"></a>

### `MapMarker` (desc struct)

A marker on a mini-map.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `ratio` | — | Across the map, 0 to 1. |
| `y` | `number` | `0` | — | `ratio` | — | Down the map, 0 to 1. |
| `tone` | `Tone` | `"neutral"` | — | — | — | Its colour; neutral is the stage ink. |
| `player` | `boolean` | `false` | — | — | — | The player: the primary, larger, ringed. |

```ogt
struct MapMarker {
  doc "A marker on a mini-map."
  desc true
  field x f32 "Across the map, 0 to 1." {
    unit ratio
  }
  field y f32 "Down the map, 0 to 1." {
    unit ratio
  }
  field tone Tone "Its colour; neutral is the stage ink."
  field player bool "The player: the primary, larger, ringed."
}
```

<sub>`api.ogt` line 16777</sub>
