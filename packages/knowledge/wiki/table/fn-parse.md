<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-parse"></a>

### `og.table.parse`

A table from text, now, under the asset id `path` (tests and generated content; files go through og cook, which also checks assets and the rows named): the same reader and checks as og.table.load.

```ts
og.table.parse<C extends AnyComponent>(world: World, path: string, text: string, schema: C): Result<TableInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The asset id it is loaded under, which its own `row "id"` fields name. |
| `text` | `string` | no | A `type table` .ogt. |
| `schema` | `C` | no | The game component each row is read as. |

**Returns** `Result<TableInfo>`: Refused as og.table.load is.

Binding symbol `og_table_parse` · publish builds: kept

```ogt
fn parse {
  doc "A table from text, now, under the asset id `path` (tests and generated content; files go through og cook, which also checks assets and the rows named): the same reader and checks as og.table.load."
  generic C AnyComponent
  param world World "The world."
  param path String "The asset id it is loaded under, which its own `row \"id\"` fields name."
  param text String "A `type table` .ogt."
  param schema C "The game component each row is read as."
  returns Result.TableInfo "Refused as og.table.load is."
}
```

<sub>Module `table` line 46</sub>
