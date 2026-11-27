<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-row"></a>

### `og.table.row`

The handle of a row by its table and id.

```ts
og.table.row(world: World, table: string, id: string): Result<TableRow>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `table` | `string` | no | The table's path. |
| `id` | `string` | no | The row's id. |

**Returns** `Result<TableRow>`: Refused for a table not loaded or an id it lacks.

Binding symbol `og_table_row` · publish builds: kept

```ogt
fn row {
  doc "The handle of a row by its table and id."
  param world World "The world."
  param table String "The table's path."
  param id String "The row's id."
  returns Result.TableRow "Refused for a table not loaded or an id it lacks."
}
```

<sub>Module `table` line 56</sub>
