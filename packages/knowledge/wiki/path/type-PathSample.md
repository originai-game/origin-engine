<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="type-PathSample"></a>

### `PathSample` (struct)

A point of a path.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `tangent` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction of travel. |
| `up` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | A unit up perpendicular to the tangent: world +Y made perpendicular (world +Z where the path runs straight up or down). |

```ogt
struct PathSample {
  doc "A point of a path."
  field position Vector3 "Where it is." {
    unit m
  }
  field tangent Vector3 "The unit direction of travel."
  field up Vector3 "A unit up perpendicular to the tangent: world +Y made perpendicular (world +Z where the path runs straight up or down)."
}
```

<sub>`api.ogt` line 26013</sub>
