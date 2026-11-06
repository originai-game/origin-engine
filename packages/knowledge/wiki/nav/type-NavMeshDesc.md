<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavMeshDesc"></a>

### `NavMeshDesc` (desc struct)

A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The floor's lowest corner (its y is the floor's height). |
| `max` | `Vector3` | `{ x: 16, y: 1, z: 16 }` | — | `m` | — | The floor's highest corner. |
| `blocked` | `readonly NavBox[]` | `[]` | — | — | — | Boxes standing on the floor (walls, crates). |
| `cellSize` | `number` | `0.25` | 0.02 … 4 | `m` | — | The voxel width. |
| `agentRadius` | `number` | `0.4` | 0 … 10 | `m` | — | Walls are kept this far away. |
| `agentHeight` | `number` | `1.8` | 0.1 … 20 | `m` | — | The agent's height. |
| `tileSize` | `number` | `32` | 8 … 256 | — | — | Cells per tile side: an obstacle rebuilds the tiles it touches. |

```ogt
struct NavMeshDesc {
  doc "A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for."
  desc true
  field min Vector3 "The floor's lowest corner (its y is the floor's height)." {
    unit m
  }
  field max Vector3 "The floor's highest corner." {
    default 16 1 16
    unit m
  }
  field blocked List.NavBox "Boxes standing on the floor (walls, crates)."
  field cellSize f32 "The voxel width." {
    default 0.25
    min 0.02
    max 4
    unit m
  }
  field agentRadius f32 "Walls are kept this far away." {
    default 0.4
    min 0
    max 10
    unit m
  }
  field agentHeight f32 "The agent's height." {
    default 1.8
    min 0.1
    max 20
    unit m
  }
  field tileSize u32 "Cells per tile side: an obstacle rebuilds the tiles it touches." {
    default 32
    min 8
    max 256
  }
}
```

<sub>`api.ogt` line 13978</sub>
