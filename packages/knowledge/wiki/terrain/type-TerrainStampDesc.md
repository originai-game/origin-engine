<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainStampDesc"></a>

### `TerrainStampDesc` (desc struct)

One stamp of a mesh into a terrain (og.terrain.stamp).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain. |
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the terrain stands: its og.Terrain entity's world position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How it is turned: its og.Terrain entity's world rotation. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How it is scaled: its og.Terrain entity's world scale. |
| `geometry` | `Geometry` | `0` | — | — | — | The mesh (its og.TerrainStamp's geometry). |
| `meshPosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the mesh stands: its og.TerrainStamp entity's world position. |
| `meshQuaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How the mesh is turned: its entity's world rotation. |
| `meshScale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How the mesh is scaled: its entity's world scale. |
| `mode` | `TerrainStampMode` | `"carve"` | — | — | — | What it does to the ground. |
| `depth` | `number` | `0.25` | ≥ 0 | `m` | — | carve: how far below the mesh's underside the ground goes, so it stays hidden under the mesh. |

```ogt
struct TerrainStampDesc {
  doc "One stamp of a mesh into a terrain (og.terrain.stamp)."
  desc true
  field data TerrainData "The terrain."
  field origin Vector3 "Where the terrain stands: its og.Terrain entity's world position." {
    unit m
  }
  field quaternion Quaternion "How it is turned: its og.Terrain entity's world rotation."
  field scale Vector3 "How it is scaled: its og.Terrain entity's world scale." {
    default 1 1 1
  }
  field geometry Geometry "The mesh (its og.TerrainStamp's geometry)."
  field meshPosition Vector3 "Where the mesh stands: its og.TerrainStamp entity's world position." {
    unit m
  }
  field meshQuaternion Quaternion "How the mesh is turned: its entity's world rotation."
  field meshScale Vector3 "How the mesh is scaled: its entity's world scale." {
    default 1 1 1
  }
  field mode TerrainStampMode "What it does to the ground."
  field depth f32 "carve: how far below the mesh's underside the ground goes, so it stays hidden under the mesh." {
    default 0.25
    min 0
    unit m
  }
}
```

<sub>Module `terrain` line 530</sub>
