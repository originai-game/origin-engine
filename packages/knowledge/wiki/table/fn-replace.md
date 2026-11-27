<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-replace"></a>

### `og.table.replace`

Atomically replace a table with structured runtime rows. Values use the component's runtime shapes and numeric asset/row handles, not author ref tokens. Omitted fields take their component defaults. Refusal preserves the previous table. Author files go through file.table and cook instead.

```ts
og.table.replace<C extends AnyComponent>(world: World, path: string, data: unknown, schema: C): Result<TableInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The canonical identity to install, such as base/items. |
| `data` | `unknown` | no | {locale?: string, strings?: {[key: string]: string}, rows: [{id: string, values: {...}}]}; row IDs must be unique and nonempty; strings supplies localized base messages. |
| `schema` | `C` | no | The game component each row is read as. |

**Returns** `Result<TableInfo>`: Refused as og.table.load is.

Binding symbol `og_table_replace` · publish builds: kept

```ogt
fn replace {
  doc "Atomically replace a table with structured runtime rows. Values use the component's runtime shapes and numeric asset/row handles, not author ref tokens. Omitted fields take their component defaults. Refusal preserves the previous table. Author files go through file.table and cook instead."
  generic C AnyComponent
  param world World "The world."
  param path String "The canonical identity to install, such as base/items."
  param data Json "{locale?: string, strings?: {[key: string]: string}, rows: [{id: string, values: {...}}]}; row IDs must be unique and nonempty; strings supplies localized base messages."
  param schema C "The game component each row is read as."
  returns Result.TableInfo "Refused as og.table.load is."
}
```

<sub>Module `table` line 47</sub>
