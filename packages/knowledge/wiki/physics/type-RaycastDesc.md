<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RaycastDesc"></a>

### `RaycastDesc` (desc struct)

A ray for og.physics.castRay.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the ray starts. |
| `direction` | `Vector3` | `{ x: 0, y: -1, z: 0 }` | — | — | — | Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far it reaches; 0 is unbounded. |
| `solid` | `boolean` | `true` | — | — | — | A ray starting inside a solid collider hits immediately at distance 0; false reports where it exits the surface. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct RaycastDesc {
  doc "A ray for og.physics.castRay."
  desc true
  field origin Vector3 "Where the ray starts." {
    unit m
  }
  field direction Vector3 "Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy)." {
    default 0 -1 0
  }
  field maxDistance f32 "How far it reaches; 0 is unbounded." {
    min 0
    unit m
  }
  field solid bool "A ray starting inside a solid collider hits immediately at distance 0; false reports where it exits the surface." {
    default true
  }
  field filter QueryFilter "Which colliders count."
}
```

<sub>Module `physics` line 2173</sub>
