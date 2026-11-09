<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-MarkRect"></a>

### `MarkRect` (struct)

A mark's box in the PNG's pixels.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `px` | — | Left edge. |
| `y` | `number` | `0` | — | `px` | — | Top edge. |
| `width` | `number` | `0` | — | `px` | — | Width. |
| `height` | `number` | `0` | — | `px` | — | Height. |

```ogt
struct MarkRect {
  doc "A mark's box in the PNG's pixels."
  field x u32 "Left edge." {
    unit px
  }
  field y u32 "Top edge." {
    unit px
  }
  field width u32 "Width." {
    unit px
  }
  field height u32 "Height." {
    unit px
  }
}
```

<sub>Module `probe` line 187</sub>
