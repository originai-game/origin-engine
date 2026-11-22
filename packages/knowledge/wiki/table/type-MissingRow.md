<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="type-MissingRow"></a>

### `MissingRow` (struct)

A field that holds a TableRow handle no loaded table has (og.table.missing): a row deleted since a save, or a table not loaded yet.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose component holds it; null for a resource. |
| `record` | `string` | `""` | — | — | — | The component or resource. |
| `field` | `string` | `""` | — | — | — | The field. |
| `row` | `TableRow` | `0` | — | — | — | The handle it holds. |

```ogt
struct MissingRow {
  doc "A field that holds a TableRow handle no loaded table has (og.table.missing): a row deleted since a save, or a table not loaded yet."
  field entity Entity "The entity whose component holds it; null for a resource."
  field record String "The component or resource."
  field field String "The field."
  field row TableRow "The handle it holds."
}
```

<sub>Module `table` line 29</sub>
