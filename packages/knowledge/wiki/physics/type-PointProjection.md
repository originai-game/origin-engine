<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-PointProjection"></a>

### `PointProjection` (struct)

Where a point projects onto the nearest collider.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The collider's entity. |
| `body` | `Entity` | `0` | — | — | — | The body it belongs to (RaycastHit.body). |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The nearest point on the collider, in world space. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Unit direction out of the collider at the point, towards a point outside it (from a point inside, still outward); zero when the distance is 0. |
| `distance` | `number` | `0` | — | `m` | — | From the queried point to `point`. |
| `inside` | `boolean` | `false` | — | — | — | Whether the queried point is inside the collider. |

```ogt
struct PointProjection {
  doc "Where a point projects onto the nearest collider."
  field entity Entity "The collider's entity."
  field body Entity "The body it belongs to (RaycastHit.body)."
  field point Vector3 "The nearest point on the collider, in world space." {
    unit m
  }
  field normal Vector3 "Unit direction out of the collider at the point, towards a point outside it (from a point inside, still outward); zero when the distance is 0."
  field distance f32 "From the queried point to `point`." {
    unit m
  }
  field inside bool "Whether the queried point is inside the collider."
}
```

<sub>Module `physics` line 1787</sub>
