<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="record-ArtShape"></a>

### `og.ArtShape` (resource)

The style's shape language for 3D models: triangle budgets, how faces are shaded, bevelled edges, a voxel grid and the smallest feature a silhouette keeps. Every field is no constraint at its zero value (normalAngle matters only with normals angle).

Token `og.ArtShape`, registered under the schema name `og.ArtShape`; its value type is `ArtShape`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `propFaces` | `number` | `0` | ≥ 0 | — | — | The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget. |
| `characterFaces` | `number` | `0` | ≥ 0 | — | — | The most triangles one character may have; 0 is no budget. |
| `normals` | `ArtNormals` | `"any"` | — | — | — | How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle). |
| `normalAngle` | `number` | `30` | 0 … 180 | `deg` | — | With normals angle: the angle between two faces past which their edge is shaded hard. |
| `bevel` | `number` | `0` | ≥ 0 | `m` | — | Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none. |
| `voxel` | `number` | `0` | ≥ 0 | `m` | — | Models built of cubes of this edge (voxel worlds); 0 is no voxel grid. |
| `silhouette` | `number` | `0` | ≥ 0 | `m` | — | The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint. |

```ogt
resource ArtShape {
  doc "The style's shape language for 3D models: triangle budgets, how faces are shaded, bevelled edges, a voxel grid and the smallest feature a silhouette keeps. Every field is no constraint at its zero value (normalAngle matters only with normals angle)."
  field propFaces u32 "The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget." {
    min 0
  }
  field characterFaces u32 "The most triangles one character may have; 0 is no budget." {
    min 0
  }
  field normals ArtNormals "How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle)."
  field normalAngle f32 "With normals angle: the angle between two faces past which their edge is shaded hard." {
    default 30
    min 0
    max 180
    unit deg
  }
  field bevel f32 "Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none." {
    min 0
    unit m
  }
  field voxel f32 "Models built of cubes of this edge (voxel worlds); 0 is no voxel grid." {
    min 0
    unit m
  }
  field silhouette f32 "The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint." {
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 26629</sub>
