<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="type-TableRowName"></a>

### `TableRowName` (struct)

Which row a handle names.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `table` | `string` | `""` | — | — | — | The table's asset id. |
| `id` | `string` | `""` | — | — | — | The row's id. |
| `schema` | `string` | `""` | — | — | — | The component its table was loaded as, which og.table.get takes. |

```ogt
struct TableRowName {
  doc "Which row a handle names."
  field table String "The table's asset id."
  field id String "The row's id."
  field schema String "The component its table was loaded as, which og.table.get takes."
}
```

<sub>Module `table` line 22</sub>
