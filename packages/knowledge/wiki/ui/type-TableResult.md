<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TableResult"></a>

### `TableResult` (struct)

What a table answered this tick: keep the sort and the filter.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sortBy` | `number` | `0` | — | — | — | The column sorted by. |
| `descending` | `boolean` | `false` | — | — | — | High to low. |
| `filter` | `string` | `""` | — | — | — | The search text. |
| `clicked` | `number \| null` | `null` | — | — | — | The row pressed, by its index in rows. |
| `retry` | `boolean` | `false` | — | — | — | Retry was pressed. |
| `changed` | `boolean` | `false` | — | — | — | The sort or the filter changed this tick. |

```ogt
struct TableResult {
  doc "What a table answered this tick: keep the sort and the filter."
  field sortBy i32 "The column sorted by."
  field descending bool "High to low."
  field filter String "The search text."
  field clicked Option.u32 "The row pressed, by its index in rows."
  field retry bool "Retry was pressed."
  field changed bool "The sort or the filter changed this tick."
}
```

<sub>Module `ui` line 2507</sub>
