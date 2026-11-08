<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TableDesc"></a>

### `TableDesc` (desc struct)

A sortable, filterable table of rows by columns; only the rows in view are built.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rows` | `unknown` | `null` | — | — | — | The rows: an array of objects, the columns' keys their fields. Pass the same array while it is unchanged: the sort and the filter are computed once per change of the rows, the sort or the filter. |
| `columns` | `readonly TableColumn[]` | `[]` | — | — | — | The columns. |
| `sortBy` | `number` | `-1` | — | — | — | The column sorted by; -1 is the rows' own order (keep TableResult.sortBy). |
| `descending` | `boolean` | `false` | — | — | — | Sorted high to low (keep TableResult.descending). |
| `filter` | `string` | `""` | — | — | — | The search text (keep TableResult.filter). |
| `search` | `boolean` | `true` | — | — | — | A search field over it. |
| `searchPlaceholder` | `string` | `""` | — | — | — | The search field's placeholder; empty is Search. |
| `selected` | `number \| null` | `null` | — | — | — | The selected row, by its index in rows. |
| `rowHeight` | `number \| null` | `null` | — | `cssPx` | — | Every row's height; null is 40 (48 on a phone). |
| `striped` | `boolean` | `true` | — | — | — | Every other row shaded. |
| `footer` | `boolean` | `true` | — | — | — | A count line under it (`120 of 10,000`). |
| `countLabel` | `string` | `""` | — | — | — | The count line, with {shown} and {total}; empty is `{shown} of {total}`. |
| `noMatchLabel` | `string` | `""` | — | — | — | What it says when the filter keeps no row; empty is No matches. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (the rows scroll inside); null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |
| `loading` | `boolean` | `false` | — | — | — | Loading: skeleton rows. |
| `error` | `string` | `""` | — | — | — | An error to show with Retry; empty is none. |
| `emptyLabel` | `string` | `""` | — | — | — | What it says when there are no rows. |
| `emptyHint` | `string` | `""` | — | — | — | A line under it. |
| `retry` | `boolean` | `true` | — | — | — | The error has a Retry button. |
| `retryLabel` | `string` | `""` | — | — | — | Retry's label. |
| `loadingLabel` | `string` | `""` | — | — | — | What a screen reader hears while loading. |

```ogt
struct TableDesc {
  doc "A sortable, filterable table of rows by columns; only the rows in view are built."
  desc true
  field rows Json "The rows: an array of objects, the columns' keys their fields. Pass the same array while it is unchanged: the sort and the filter are computed once per change of the rows, the sort or the filter."
  field columns List.TableColumn "The columns."
  field sortBy i32 "The column sorted by; -1 is the rows' own order (keep TableResult.sortBy)." {
    default -1
  }
  field descending bool "Sorted high to low (keep TableResult.descending)."
  field filter String "The search text (keep TableResult.filter)."
  field search bool "A search field over it." {
    default true
  }
  field searchPlaceholder String "The search field's placeholder; empty is Search."
  field selected Option.u32 "The selected row, by its index in rows."
  field rowHeight Option.f32 "Every row's height; null is 40 (48 on a phone)." {
    unit cssPx
  }
  field striped bool "Every other row shaded." {
    default true
  }
  field footer bool "A count line under it (`120 of 10,000`)." {
    default true
  }
  field countLabel String "The count line, with {shown} and {total}; empty is `{shown} of {total}`."
  field noMatchLabel String "What it says when the filter keeps no row; empty is No matches."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height (the rows scroll inside); null fits."
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

<sub>Module `ui` line 2044</sub>
