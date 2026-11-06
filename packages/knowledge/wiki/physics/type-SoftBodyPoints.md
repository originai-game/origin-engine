<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-SoftBodyPoints"></a>

### `SoftBodyPoints` (struct)

An og.SoftBody's surface after the last step.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | — | — | Its points, in world space. |
| `indices` | `readonly number[]` | `[]` | — | — | — | Its triangles, three indices into `points` each, wound outwards (2D: a fan over the ring). |

```ogt
struct SoftBodyPoints {
  doc "An og.SoftBody's surface after the last step."
  field points List.Vector3 "Its points, in world space."
  field indices List.u32 "Its triangles, three indices into `points` each, wound outwards (2D: a fan over the ring)."
}
```

<sub>`api.ogt` line 10487</sub>
