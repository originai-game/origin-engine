<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="fn-missing"></a>

### `og.table.missing`

Every component and resource field holding a TableRow that no loaded table has, by entity then record and field: what a save from before a row was deleted still refers to. Ask after og.WorldRestored.

```ts
og.table.missing(world: World): readonly MissingRow[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly MissingRow[]`: Empty when every row held is loaded.

Binding symbol `og_table_missing` · publish builds: kept

```ogt
fn missing {
  doc "Every component and resource field holding a TableRow that no loaded table has, by entity then record and field: what a save from before a row was deleted still refers to. Ask after og.WorldRestored."
  param world World "The world."
  returns List.MissingRow "Empty when every row held is loaded."
}
```

<sub>Module `table` line 88</sub>
