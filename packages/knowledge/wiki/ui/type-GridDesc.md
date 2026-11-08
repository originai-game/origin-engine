<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GridDesc"></a>

### `GridDesc` (desc struct)

A virtualised grid of equal cells (inventories, level select), scrolling vertically.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many cells there are. |
| `cellWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's width; 0 is 64 (never under the 44 px touch target). With columns set and a width other than fit, the cells share the grid's width instead, as wide as it allows (never under the touch target), and keep their height. |
| `cellHeight` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's height; 0 is its width. |
| `columns` | `number` | `0` | — | — | — | Cells across, exactly, whatever the width: a fit grid is that many cells of cellWidth, any other the cells share; 0 is as many cells of cellWidth as fit. |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | Space between cells; 0 is space.sm. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits (exactly the columns when columns is set). |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null grows. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `selected` | `number \| null` | `null` | — | — | — | The selected cell, for og.ui.grid. |

```ogt
struct GridDesc {
  doc "A virtualised grid of equal cells (inventories, level select), scrolling vertically."
  desc true
  field count u32 "How many cells there are."
  field cellWidth f32 "A cell's width; 0 is 64 (never under the 44 px touch target). With columns set and a width other than fit, the cells share the grid's width instead, as wide as it allows (never under the touch target), and keep their height." {
    unit cssPx
    min 0
  }
  field cellHeight f32 "A cell's height; 0 is its width." {
    unit cssPx
    min 0
  }
  field columns u32 "Cells across, exactly, whatever the width: a fit grid is that many cells of cellWidth, any other the cells share; 0 is as many cells of cellWidth as fit."
  field gap f32 "Space between cells; 0 is space.sm." {
    unit cssPx
    min 0
  }
  field width Option.Size "Its width; null fits (exactly the columns when columns is set)."
  field height Option.Size "Its height; null grows."
  field padding Insets "Space inside."
  field selected Option.u32 "The selected cell, for og.ui.grid."
}
```

<sub>Module `ui` line 915</sub>
