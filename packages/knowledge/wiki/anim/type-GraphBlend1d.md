<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphBlend1d"></a>

### `GraphBlend1d` (struct)

Threshold interpolation, clamped at the ends.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `param` | `string` | `""` | — | — | — | Declared parameter name. |
| `children` | `readonly GraphThreshold[]` | `[]` | — | — | — | At least one child; forward motion references are allowed. |

```ogt
struct GraphBlend1d {
  doc "Threshold interpolation, clamped at the ends."
  field param String "Declared parameter name."
  field children List.GraphThreshold "At least one child; forward motion references are allowed."
}
```

<sub>Module `anim` line 733</sub>
