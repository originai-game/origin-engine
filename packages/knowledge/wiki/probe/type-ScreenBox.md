<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ScreenBox"></a>

### `ScreenBox` (struct)

A box on the play surface.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Left edge. |
| `y` | `number` | `0` | — | `cssPx` | — | Top edge. |
| `width` | `number` | `0` | — | `cssPx` | — | Width. |
| `height` | `number` | `0` | — | `cssPx` | — | Height. |

```ogt
struct ScreenBox {
  doc "A box on the play surface."
  field x f32 "Left edge." {
    unit cssPx
  }
  field y f32 "Top edge." {
    unit cssPx
  }
  field width f32 "Width." {
    unit cssPx
  }
  field height f32 "Height." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 24417</sub>
