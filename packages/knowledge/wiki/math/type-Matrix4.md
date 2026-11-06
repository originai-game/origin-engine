<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Matrix4"></a>

### `Matrix4` (struct)

A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `elements` | `[number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number]` | `[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]` | — | — | — | The elements, column-major, as three.js Matrix4.elements. |

```ogt
struct Matrix4 {
  doc "A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity."
  field elements Array.f32.16 "The elements, column-major, as three.js Matrix4.elements." {
    default 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1
  }
}
```

<sub>`api.ogt` line 69</sub>
