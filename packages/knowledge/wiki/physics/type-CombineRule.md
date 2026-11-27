<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-CombineRule"></a>

### `CombineRule` (enum)

How two colliders' friction or restitution coefficients combine.

A string: one of the variants below. The zero value is `"average"`.

| Variant | Doc |
|---|---|
| `"average"` | Arithmetic mean of both colliders. |
| `"min"` | Minimum of both colliders. |
| `"multiply"` | Product of both colliders. |
| `"max"` | Maximum of both colliders. |

```ogt
enum CombineRule {
  doc "How two colliders' friction or restitution coefficients combine."
  variant average "Arithmetic mean of both colliders."
  variant min "Minimum of both colliders."
  variant multiply "Product of both colliders."
  variant max "Maximum of both colliders."
}
```

<sub>Module `physics` line 120</sub>
