<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ControlSizes"></a>

### `ControlSizes` (struct)

The control, row and header heights and the gutter of the density in effect.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `controlSm` | `number` | `0` | — | `cssPx` | — | A small control's height. |
| `controlMd` | `number` | `0` | — | `cssPx` | — | A control's height. |
| `controlLg` | `number` | `0` | — | `cssPx` | — | A large control's height. |
| `gutter` | `number` | `0` | — | `cssPx` | — | The gutter. |
| `header` | `number` | `0` | — | `cssPx` | — | The header height. |
| `row` | `number` | `0` | — | `cssPx` | — | The row height. |
| `section` | `number` | `0` | — | `cssPx` | — | The section spacing. |

```ogt
struct ControlSizes {
  doc "The control, row and header heights and the gutter of the density in effect."
  field controlSm f32 "A small control's height." {
    unit cssPx
  }
  field controlMd f32 "A control's height." {
    unit cssPx
  }
  field controlLg f32 "A large control's height." {
    unit cssPx
  }
  field gutter f32 "The gutter." {
    unit cssPx
  }
  field header f32 "The header height." {
    unit cssPx
  }
  field row f32 "The row height." {
    unit cssPx
  }
  field section f32 "The section spacing." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 16189</sub>
