<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DataGridDesc"></a>

### `DataGridDesc` (desc struct)

A virtualised grid of equal cells with its empty, loading and error states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many cells. |
| `cellWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's width; 0 is 64 (see GridDesc.cellWidth). |
| `cellHeight` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's height; 0 is its width. |
| `columns` | `number` | `0` | — | — | — | Cells across; 0 is as many as fit. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Space between cells; null is space.sm. |
| `padding` | `Insets \| null` | `null` | — | — | — | Space inside; null is none. |
| `selected` | `number \| null` | `null` | — | — | — | The selected cell. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (it scrolls inside); null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |
| `loading` | `boolean` | `false` | — | — | — | Loading: skeleton rows. |
| `error` | `string` | `""` | — | — | — | An error to show with Retry; empty is none. |
| `emptyLabel` | `string` | `""` | — | — | — | What it says when there are no cells. |
| `emptyHint` | `string` | `""` | — | — | — | A line under it. |
| `retry` | `boolean` | `true` | — | — | — | The error has a Retry button. |
| `retryLabel` | `string` | `""` | — | — | — | Retry's label. |
| `loadingLabel` | `string` | `""` | — | — | — | What a screen reader hears while loading. |

```ogt
struct DataGridDesc {
  doc "A virtualised grid of equal cells with its empty, loading and error states."
  desc true
  field count u32 "How many cells."
  field cellWidth f32 "A cell's width; 0 is 64 (see GridDesc.cellWidth)." {
    unit cssPx
    min 0
  }
  field cellHeight f32 "A cell's height; 0 is its width." {
    unit cssPx
    min 0
  }
  field columns u32 "Cells across; 0 is as many as fit."
  field gap Option.f32 "Space between cells; null is space.sm." {
    unit cssPx
  }
  field padding Option.Insets "Space inside; null is none."
  field selected Option.u32 "The selected cell."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height (it scrolls inside); null fits."
  field label String "What a screen reader calls it."
  field loading bool "Loading: skeleton rows."
  field error String "An error to show with Retry; empty is none."
  field emptyLabel String "What it says when there are no cells."
  field emptyHint String "A line under it."
  field retry bool "The error has a Retry button." {
    default true
  }
  field retryLabel String "Retry's label."
  field loadingLabel String "What a screen reader hears while loading."
}
```

<sub>Module `ui` line 2512</sub>
