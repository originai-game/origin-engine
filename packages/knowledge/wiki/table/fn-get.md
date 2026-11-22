<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-get"></a>

### `og.table.get`

A row's values, as the component its table was loaded as.

```ts
og.table.get<C extends AnyComponent>(world: World, row: TableRow, schema: C): Result<Value<C>>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `row` | `TableRow` | no | The row. |
| `schema` | `C` | no | The component its table was loaded as. |

**Returns** `Result<Value<C>>`: Refused for a row no loaded table has, or a table loaded as another component.

Binding symbol `og_table_get` · publish builds: kept

```ogt
fn get {
  doc "A row's values, as the component its table was loaded as."
  generic C AnyComponent
  param world World "The world."
  param row TableRow "The row."
  param schema C "The component its table was loaded as."
  returns Result.Value.C "Refused for a row no loaded table has, or a table loaded as another component."
}
```

<sub>Module `table` line 72</sub>
