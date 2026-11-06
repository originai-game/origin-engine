<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-table"></a>

### `og.ui.table`

A sortable, filterable table; only the rows in view are built, so 10,000 rows and more stay within budget.

```ts
og.ui.table(id: string, desc?: TableDesc): TableResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `TableDesc` | yes | Rows, columns, the sort and the filter. |

**Returns** `TableResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn table {
  doc "A sortable, filterable table; only the rows in view are built, so 10,000 rows and more stay within budget."
  host false
  param id String "Its id."
  param desc TableDesc "Rows, columns, the sort and the filter."
  returns TableResult
}
```

<sub>`api.ogt` line 18828</sub>
