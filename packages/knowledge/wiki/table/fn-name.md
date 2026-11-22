<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-name"></a>

### `og.table.name`

Which table and id a row's handle names.

```ts
og.table.name(world: World, row: TableRow): Result<TableRowName>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `row` | `TableRow` | no | The row. |

**Returns** `Result<TableRowName>`: Refused for a row no loaded table has.

Binding symbol `og_table_name` · publish builds: kept

```ogt
fn name {
  doc "Which table and id a row's handle names."
  param world World "The world."
  param row TableRow "The row."
  returns Result.TableRowName "Refused for a row no loaded table has."
}
```

<sub>Module `table` line 81</sub>
