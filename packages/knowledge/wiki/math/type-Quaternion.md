<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Quaternion"></a>

### `Quaternion` (struct)

A unit quaternion rotation, as three.js Quaternion. The zero value is the identity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |
| `z` | `number` | `0` | — | — | — | The z component. |
| `w` | `number` | `1` | — | — | — | The w (scalar) component. |

```ogt
struct Quaternion {
  doc "A unit quaternion rotation, as three.js Quaternion. The zero value is the identity."
  field x f32 "The x component."
  field y f32 "The y component."
  field z f32 "The z component."
  field w f32 "The w (scalar) component." {
    default 1
  }
}
```

<sub>`api.ogt` line 59</sub>
