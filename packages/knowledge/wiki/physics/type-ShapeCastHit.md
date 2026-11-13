<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-ShapeCastHit"></a>

### `ShapeCastHit` (struct)

The first collider a swept shape meets.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The collider's entity. |
| `distance` | `number` | `0` | — | `m` | — | How far the shape travelled before touching. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The contact point on the hit collider, in world space. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The hit collider's surface normal there, in world space. |

```ogt
struct ShapeCastHit {
  doc "The first collider a swept shape meets."
  field entity Entity "The collider's entity."
  field distance f32 "How far the shape travelled before touching." {
    unit m
  }
  field point Vector3 "The contact point on the hit collider, in world space."
  field normal Vector3 "The hit collider's surface normal there, in world space."
}
```

<sub>Module `physics` line 1875</sub>
