<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-FractureDesc"></a>

### `FractureDesc` (desc struct)

Deterministic 3D Voronoi fracture of a closed triangle mesh into solid pieces and Blast-style contact bonds.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `positions` | `Float32Array` | — | — | — | — | Closed mesh vertex positions, x y z per vertex. |
| `indices` | `Uint32Array` | — | — | — | — | Counter-clockwise triangle indices (multiple of 3). |
| `sites` | `readonly Vector3[]` | `[]` | — | — | — | Optional 2..128 distinct finite Voronoi cell centres in mesh metres; replaces chunks, seed and impact sampling. Site order determines cell ids, including empty cells outside the mesh. |
| `chunks` | `number` | `8` | 2 … 128 | — | — | Target number of Voronoi seed cells. |
| `seed` | `number` | `1` | — | — | — | Deterministic RNG seed for cell centres. |
| `metric` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | Anisotropic Voronoi distance metric (smaller component stretches shards along that axis, e.g. wood splinters). |
| `impact` | `Vector3 \| null` | `null` | — | `m` | — | Optional impact point in mesh space to cluster smaller shards nearby. |
| `impactRadius` | `number` | `0` | ≥ 0 | `m` | — | Cluster radius around `impact` (0 uses half the mesh size). |
| `uvScale` | `number` | `1` | ≥ 0.001 | — | — | Box-projection UV repeats per metre on interior cut faces. |

```ogt
struct FractureDesc {
  doc "Deterministic 3D Voronoi fracture of a closed triangle mesh into solid pieces and Blast-style contact bonds."
  desc true
  field positions Float32Array "Closed mesh vertex positions, x y z per vertex."
  field indices Uint32Array "Counter-clockwise triangle indices (multiple of 3)."
  field sites List.Vector3 "Optional 2..128 distinct finite Voronoi cell centres in mesh metres; replaces chunks, seed and impact sampling. Site order determines cell ids, including empty cells outside the mesh."
  field chunks u32 "Target number of Voronoi seed cells." {
    default 8
    min 2
    max 128
  }
  field seed u32 "Deterministic RNG seed for cell centres." {
    default 1
  }
  field metric Vector3 "Anisotropic Voronoi distance metric (smaller component stretches shards along that axis, e.g. wood splinters)." {
    default 1 1 1
  }
  field impact Option.Vector3 "Optional impact point in mesh space to cluster smaller shards nearby." {
    unit m
  }
  field impactRadius f32 "Cluster radius around `impact` (0 uses half the mesh size)." {
    min 0
    unit m
  }
  field uvScale f32 "Box-projection UV repeats per metre on interior cut faces." {
    default 1
    min 0.001
  }
}
```

<sub>Module `procgen` line 870</sub>
