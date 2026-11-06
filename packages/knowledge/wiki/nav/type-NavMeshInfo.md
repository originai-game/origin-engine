<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavMeshInfo"></a>

### `NavMeshInfo` (struct)

A navmesh's shape.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mesh` | `Navigation` | `0` | — | — | — | What NavAgent.nav and the queries take. |
| `polygons` | `number` | `0` | — | — | — | Its convex polygons. |
| `tiles` | `number` | `0` | — | — | — | Its tiles. |
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The lowest corner of the box around its polygons. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The highest corner of the box around its polygons (its highest walkable floor's y). |
| `cellSize` | `number` | `0` | — | `m` | — | Its voxel width. |
| `profile` | `string` | `""` | — | — | — | The agent profile it was baked for (og.nav.load's profile); empty for a navmesh of one unnamed agent and for og.nav.build. |

```ogt
struct NavMeshInfo {
  doc "A navmesh's shape."
  field mesh Navigation "What NavAgent.nav and the queries take."
  field polygons u32 "Its convex polygons."
  field tiles u32 "Its tiles."
  field min Vector3 "The lowest corner of the box around its polygons." {
    unit m
  }
  field max Vector3 "The highest corner of the box around its polygons (its highest walkable floor's y)." {
    unit m
  }
  field cellSize f32 "Its voxel width." {
    unit m
  }
  field profile String "The agent profile it was baked for (og.nav.load's profile); empty for a navmesh of one unnamed agent and for og.nav.build."
}
```

<sub>`api.ogt` line 14013</sub>
