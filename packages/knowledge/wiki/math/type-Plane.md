<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Plane"></a>

### `Plane` (struct)

A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit normal. |
| `constant` | `number` | `0` | — | `m` | — | The signed distance from the origin to the plane, negated along the normal (three.js Plane.constant). |

```ogt
struct Plane {
  doc "A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points."
  field normal Vector3 "The unit normal."
  field constant f32 "The signed distance from the origin to the plane, negated along the normal (three.js Plane.constant)." {
    unit m
  }
}
```

<sub>`api.ogt` line 103</sub>
