<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.table.load`

Read a cooked table as `schema`'s rows: each field of a row is checked against the component's field of that name (its kind, a str's capacity, an enum's variants, a list's length), a field left out takes its default, and a row field holds a TableRow. All or nothing: a table with one bad field is refused, and one loaded before stays as it was. Loading a path again replaces its rows; handles stay valid for the ids it still has. Its `text` fields' base text joins og.i18n. Load tables in setup: a hot reload runs setup again.

Also known as: data table, item database, spreadsheet, ScriptableObject, DataTable, CSV rows, game data, load items, json data

```ts
og.table.load<C extends AnyComponent>(world: World, path: string, schema: C): Result<TableInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The table's path. |
| `schema` | `C` | no | The game component each row is read as. |

**Returns** `Result<TableInfo>`: Refused for a path that is not a cooked table, a different or undeclared component, colliding row handles, or incompatible values.

Binding symbol `og_table_load` · publish builds: kept

```ogt
fn load {
  doc "Read a cooked table as `schema`'s rows: each field of a row is checked against the component's field of that name (its kind, a str's capacity, an enum's variants, a list's length), a field left out takes its default, and a row field holds a TableRow. All or nothing: a table with one bad field is refused, and one loaded before stays as it was. Loading a path again replaces its rows; handles stay valid for the ids it still has. Its `text` fields' base text joins og.i18n. Load tables in setup: a hot reload runs setup again."
  aliases "data table, item database, spreadsheet, ScriptableObject, DataTable, CSV rows, game data, load items, json data"
  generic C AnyComponent
  param world World "The world."
  param path String "The table's path."
  param schema C "The game component each row is read as."
  returns Result.TableInfo "Refused for a path that is not a cooked table, a different or undeclared component, colliding row handles, or incompatible values."
}
```

<sub>Module `table` line 36</sub>
