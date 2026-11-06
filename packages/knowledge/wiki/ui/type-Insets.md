<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Insets"></a>

### `Insets` (struct)

Padding or margins on four sides.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `left` | `number` | `0` | — | `cssPx` | — | The left side. |
| `top` | `number` | `0` | — | `cssPx` | — | The top side. |
| `right` | `number` | `0` | — | `cssPx` | — | The right side. |
| `bottom` | `number` | `0` | — | `cssPx` | — | The bottom side. |

```ogt
struct Insets {
  doc "Padding or margins on four sides."
  field left f32 "The left side." {
    unit cssPx
  }
  field top f32 "The top side." {
    unit cssPx
  }
  field right f32 "The right side." {
    unit cssPx
  }
  field bottom f32 "The bottom side." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 15803</sub>
