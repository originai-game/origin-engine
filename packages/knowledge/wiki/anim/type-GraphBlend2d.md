<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphBlend2d"></a>

### `GraphBlend2d` (struct)

Freeform Cartesian gradient-band interpolation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `string` | `""` | — | — | — | Declared x parameter name. |
| `y` | `string` | `""` | — | — | — | Declared y parameter name. |
| `children` | `readonly GraphPoint[]` | `[]` | — | — | — | At least one child. |

```ogt
struct GraphBlend2d {
  doc "Freeform Cartesian gradient-band interpolation."
  field x String "Declared x parameter name."
  field y String "Declared y parameter name."
  field children List.GraphPoint "At least one child."
}
```

<sub>Module `anim` line 727</sub>
