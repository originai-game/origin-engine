<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-TerrainChannelDesc"></a>

### `TerrainChannelDesc` (desc struct)

An authored valley or river centre line in terrain-local xyz metres. The existing river carver's parabolic section lowers toward the line's y minus depth, never raises ground. Repeated river lines form a network by sharing endpoints; the game specifies every branch. Applied after automatic processing, before preservation and biome painting.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | — | — | At least two finite xyz points with distinct consecutive xz positions. |
| `width` | `number` | `0` | 0 … 1000000 | — | — | Required full width in metres, 0.001..1000000; zero is refused. |
| `depth` | `number` | `0` | 0 … 1000000 | — | — | Distance below the line's y to carve the bed; zero places a valley floor at the line. |
| `river` | `boolean` | `false` | — | — | — | Mark as a river: y must not rise downstream; its declared segment lengths contribute to riverLength. |

```ogt
struct TerrainChannelDesc {
  doc "An authored valley or river centre line in terrain-local xyz metres. The existing river carver's parabolic section lowers toward the line's y minus depth, never raises ground. Repeated river lines form a network by sharing endpoints; the game specifies every branch. Applied after automatic processing, before preservation and biome painting."
  desc true
  field points List.Vector3 "At least two finite xyz points with distinct consecutive xz positions."
  field width f32 "Required full width in metres, 0.001..1000000; zero is refused." {
    min 0
    max 1000000
  }
  field depth f32 "Distance below the line's y to carve the bed; zero places a valley floor at the line." {
    min 0
    max 1000000
  }
  field river bool "Mark as a river: y must not rise downstream; its declared segment lengths contribute to riverLength."
}
```

<sub>Module `procgen` line 484</sub>
