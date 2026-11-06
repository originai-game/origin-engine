<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Sphere"></a>

### `Sphere` (struct)

A sphere, as three.js Sphere. A negative radius is empty.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The centre. |
| `radius` | `number` | `0` | — | `m` | — | The radius. |

```ogt
struct Sphere {
  doc "A sphere, as three.js Sphere. A negative radius is empty."
  field center Vector3 "The centre."
  field radius f32 "The radius." {
    unit m
  }
}
```

<sub>`api.ogt` line 95</sub>
