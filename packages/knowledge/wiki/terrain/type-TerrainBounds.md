<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainBounds"></a>

### `TerrainBounds` (struct)

A terrain's box in world space.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The lowest corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The highest corner. |

```ogt
struct TerrainBounds {
  doc "A terrain's box in world space."
  field min Vector3 "The lowest corner." {
    unit m
  }
  field max Vector3 "The highest corner." {
    unit m
  }
}
```

<sub>`api.ogt` line 25251</sub>
