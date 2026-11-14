<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-ShapeDesc"></a>

### `ShapeDesc` (desc struct)

The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `ColliderShape` | `"box"` | — | — | — | The shape, as Collider.shape; a cast or an overlap is made for box, sphere, capsule, cylinder, convex and trimesh. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | `m` | — | Full extents of a box. |
| `radius` | `number` | `0.5` | — | `m` | — | Radius of a sphere, circle, capsule or cylinder. |
| `height` | `number` | `1` | — | `m` | — | Capsule or cylinder height. |
| `mesh` | `Geometry` | `0` | — | — | — | The geometry of a convex or triangle-mesh shape. |

```ogt
struct ShapeDesc {
  doc "The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box."
  desc true
  field shape ColliderShape "The shape, as Collider.shape; a cast or an overlap is made for box, sphere, capsule, cylinder, convex and trimesh."
  field size Vector3 "Full extents of a box." {
    default 1 1 1
    unit m
  }
  field radius f32 "Radius of a sphere, circle, capsule or cylinder." {
    default 0.5
    unit m
  }
  field height f32 "Capsule or cylinder height." {
    default 1
    unit m
  }
  field mesh Geometry "The geometry of a convex or triangle-mesh shape."
}
```

<sub>Module `physics` line 1903</sub>
