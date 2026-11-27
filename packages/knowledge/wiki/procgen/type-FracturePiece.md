<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-FracturePiece"></a>

### `FracturePiece` (struct)

One solid fractured shard mesh in the input mesh's coordinate space.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cell` | `number` | `0` | — | — | — | Voronoi cell index. |
| `positions` | `Float32Array` | — | — | — | — | Vertex positions, x y z per vertex. |
| `normals` | `Float32Array` | — | — | — | — | Vertex normals, x y z per vertex. |
| `uvs` | `Float32Array` | — | — | — | — | UV coordinates, u v per vertex. |
| `indices` | `Uint32Array` | — | — | — | — | Counter-clockwise triangle indices. |
| `volume` | `number` | `0` | — | — | — | Enclosed volume. |
| `centroid` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Centre of mass in input space. |

```ogt
struct FracturePiece {
  doc "One solid fractured shard mesh in the input mesh's coordinate space."
  field cell u32 "Voronoi cell index."
  field positions Float32Array "Vertex positions, x y z per vertex."
  field normals Float32Array "Vertex normals, x y z per vertex."
  field uvs Float32Array "UV coordinates, u v per vertex."
  field indices Uint32Array "Counter-clockwise triangle indices."
  field volume f32 "Enclosed volume."
  field centroid Vector3 "Centre of mass in input space." {
    unit m
  }
}
```

<sub>Module `procgen` line 899</sub>
