<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `table` module

A game's data as rows keyed by id: items, abilities, recipes, loot, waves, levels. A table is an `.ogt` of `type table` that og cook checks (every id once, every `asset` path there, every row a `row` field names in its table); og.table.load reads it against one of the game's components, field by field, and answers each row by a handle that names the same row in every load, hot reload and save. The values are the game's: nothing here picks, sums or balances them.

See `data`.

- **TypeScript**: functions are the namespace `og.table`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module table {
  doc "A game's data as rows keyed by id: items, abilities, recipes, loot, waves, levels. A table is an `.ogt` of `type table` that og cook checks (every id once, every `asset` path there, every row a `row` field names in its table); og.table.load reads it against one of the game's components, field by field, and answers each row by a handle that names the same row in every load, hot reload and save. The values are the game's: nothing here picks, sums or balances them."
  # declarations follow, each on its own page
```

<sub>Module `table` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`TableRow`](table/type-TableRow.md) | handle |  | A row of a loaded table: a hash of its table's asset id and its own id (52 bits, so exact as a number), never 0. |
| [`TableRow()`](#column-TableRow) | column kind |  | A TableRow handle field: a row of a table, written in a table as `row "id"` or `row "table" "id"`. |
| [`TableInfo`](table/type-TableInfo.md) | struct |  | A loaded table. |
| [`TableRowName`](table/type-TableRowName.md) | struct |  | Which row a handle names. |
| [`MissingRow`](table/type-MissingRow.md) | struct |  | A field that holds a TableRow handle no loaded table has (og.table.missing): a row deleted since a save, or a table not loaded yet. |
| [`og.table.load`](table/fn-load.md) | function | `<C extends AnyComponent>(world: World, path: string, schema: C): Result<TableInfo>` | Read a cooked table as `schema`'s rows: each field of a row is checked against the component's field of that name (its kind, a str's capacity, an enum's variants, a list's length), a field left out t… |
| [`og.table.parse`](table/fn-parse.md) | function | `<C extends AnyComponent>(world: World, path: string, text: string, schema: C): Result<TableInfo>` | A table from text, now, under the asset id `path` (tests and generated content; files go through og cook, which also checks assets and the rows named): the same reader and checks as og.table.load. |
| [`og.table.row`](table/fn-row.md) | function | `(world: World, table: string, id: string): Result<TableRow>` | The handle of a row by its table and id. |
| [`og.table.rows`](table/fn-rows.md) | function | `(world: World, table: string): Result<readonly TableRow[]>` | Every row of a loaded table, in file order. |
| [`og.table.get`](table/fn-get.md) | function | `<C extends AnyComponent>(world: World, row: TableRow, schema: C): Result<Value<C>>` | A row's values, as the component its table was loaded as. |
| [`og.table.name`](table/fn-name.md) | function | `(world: World, row: TableRow): Result<TableRowName>` | Which table and id a row's handle names. |
| [`og.table.missing`](table/fn-missing.md) | function | `(world: World): readonly MissingRow[]` | Every component and resource field holding a TableRow that no loaded table has, by entity then record and field: what a save from before a row was deleted still refers to. |

## Column kinds

Each is a factory, `TableRow(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-TableRow"></a>`TableRow` | `TableRow` | `Uint32Array` | 2 | A TableRow handle field: a row of a table, written in a table as `row "id"` or `row "table" "id"`. |

```ogt
column TableRow {
  doc "A TableRow handle field: a row of a table, written in a table as `row \"id\"` or `row \"table\" \"id\"`."
  array Uint32Array
  stride 2
  value TableRow
}
```

<sub>Module `table` line 7</sub>
