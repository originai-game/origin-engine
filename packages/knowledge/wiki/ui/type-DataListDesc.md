<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DataListDesc"></a>

### `DataListDesc` (desc struct)

A virtualised list with its empty, loading and error states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many rows. |
| `rowHeight` | `number \| null` | `null` | — | `cssPx` | — | Every row's height; null is the density's row height, at least 48. |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | Space between rows. |
| `selected` | `number \| null` | `null` | — | — | — | The selected row. |
| `striped` | `boolean` | `false` | — | — | — | Every other row is marked (List.row[odd]). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (it scrolls inside); null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |
| `loading` | `boolean` | `false` | — | — | — | Loading: skeleton rows. |
| `error` | `string` | `""` | — | — | — | An error to show with Retry; empty is none. |
| `emptyLabel` | `string` | `""` | — | — | — | What it says when there are no rows. |
| `emptyHint` | `string` | `""` | — | — | — | A line under it. |
| `retry` | `boolean` | `true` | — | — | — | The error has a Retry button. |
| `retryLabel` | `string` | `""` | — | — | — | Retry's label. |
| `loadingLabel` | `string` | `""` | — | — | — | What a screen reader hears while loading. |

```ogt
struct DataListDesc {
  doc "A virtualised list with its empty, loading and error states."
  desc true
  field count u32 "How many rows."
  field rowHeight Option.f32 "Every row's height; null is the density's row height, at least 48." {
    unit cssPx
  }
  field gap f32 "Space between rows." {
    unit cssPx
    min 0
  }
  field selected Option.u32 "The selected row."
  field striped bool "Every other row is marked (List.row[odd])."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height (it scrolls inside); null fits."
  field label String "What a screen reader calls it."
  field loading bool "Loading: skeleton rows."
  field error String "An error to show with Retry; empty is none."
  field emptyLabel String "What it says when there are no rows."
  field emptyHint String "A line under it."
  field retry bool "The error has a Retry button." {
    default true
  }
  field retryLabel String "Retry's label."
  field loadingLabel String "What a screen reader hears while loading."
}
```

<sub>Module `ui` line 2475</sub>
