<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-FractureBond"></a>

### `FractureBond` (struct)

A shared contact face between two touching fracture pieces.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `number` | `0` | — | — | — | First piece index (`a < b`). |
| `b` | `number` | `0` | — | — | — | Second piece index. |
| `centroid` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Area-weighted contact centroid. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Unit contact normal from piece `a` towards piece `b`. |
| `area` | `number` | `0` | — | — | — | Contact surface area. |

```ogt
struct FractureBond {
  doc "A shared contact face between two touching fracture pieces."
  field a u32 "First piece index (`a < b`)."
  field b u32 "Second piece index."
  field centroid Vector3 "Area-weighted contact centroid." {
    unit m
  }
  field normal Vector3 "Unit contact normal from piece `a` towards piece `b`."
  field area f32 "Contact surface area."
}
```

<sub>Module `procgen` line 913</sub>
