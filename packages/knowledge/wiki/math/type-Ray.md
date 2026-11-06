<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Ray"></a>

### `Ray` (struct)

A ray, as three.js Ray.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the ray starts. |
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction the ray points in. |

```ogt
struct Ray {
  doc "A ray, as three.js Ray."
  field origin Vector3 "Where the ray starts."
  field direction Vector3 "The unit direction the ray points in."
}
```

<sub>Module `math` line 42</sub>
