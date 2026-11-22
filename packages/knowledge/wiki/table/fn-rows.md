<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-rows"></a>

### `og.table.rows`

Every row of a loaded table, in file order.

```ts
og.table.rows(world: World, table: string): Result<readonly TableRow[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `table` | `string` | no | The table's path. |

**Returns** `Result<readonly TableRow[]>`: Refused for a table not loaded.

Binding symbol `og_table_rows` · publish builds: kept

```ogt
fn rows {
  doc "Every row of a loaded table, in file order."
  param world World "The world."
  param table String "The table's path."
  returns Result.List.TableRow "Refused for a table not loaded."
}
```

<sub>Module `table` line 65</sub>
