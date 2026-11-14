<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RaycastHit"></a>

### `RaycastHit` (struct)

The nearest collider a ray meets.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The collider's entity. |
| `body` | `Entity` | `0` | — | — | — | The body the collider belongs to: its own entity, or for a compound child collider (an og.Collider on a child of an og.RigidBody) the ancestor body it is part of. |
| `distance` | `number` | `0` | — | `m` | — | From the origin along the normalised direction. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the ray meets the collider. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The surface normal there (z is 0 in 2D). |

```ogt
struct RaycastHit {
  doc "The nearest collider a ray meets."
  field entity Entity "The collider's entity."
  field body Entity "The body the collider belongs to: its own entity, or for a compound child collider (an og.Collider on a child of an og.RigidBody) the ancestor body it is part of."
  field distance f32 "From the origin along the normalised direction." {
    unit m
  }
  field point Vector3 "Where the ray meets the collider."
  field normal Vector3 "The surface normal there (z is 0 in 2D)."
}
```

<sub>Module `physics` line 1842</sub>
