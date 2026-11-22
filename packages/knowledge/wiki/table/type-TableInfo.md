<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="type-TableInfo"></a>

### `TableInfo` (struct)

A loaded table.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Its asset id, which og.table.row and og.table.rows take. |
| `schema` | `string` | `""` | — | — | — | The component its rows are read as. |
| `locale` | `string` | `""` | — | — | — | The language of its `text` fields' base text. |
| `rows` | `readonly string[]` | `[]` | — | — | — | Its row ids, in file order. |

```ogt
struct TableInfo {
  doc "A loaded table."
  field path String "Its asset id, which og.table.row and og.table.rows take."
  field schema String "The component its rows are read as."
  field locale String "The language of its `text` fields' base text."
  field rows List.String "Its row ids, in file order."
}
```

<sub>Module `table` line 14</sub>
