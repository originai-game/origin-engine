<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Vector3"></a>

### `Vector3` (struct)

A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |
| `z` | `number` | `0` | — | — | — | The z component. |

```ogt
struct Vector3 {
  doc "A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js."
  field x f32 "The x component."
  field y f32 "The y component."
  field z f32 "The z component."
}
```

<sub>Module `math` line 11</sub>
