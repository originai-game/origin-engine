<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Frustum"></a>

### `Frustum` (struct)

A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). frustum.fromCamera and frustum.fromProjectionMatrix make one.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `planes` | `readonly Plane[]` | `[]` | — | — | — | The six planes, normals inwards. |

```ogt
struct Frustum {
  doc "A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). frustum.fromCamera and frustum.fromProjectionMatrix make one."
  field planes List.Plane "The six planes, normals inwards."
}
```

<sub>Module `math` line 70</sub>
