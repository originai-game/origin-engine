<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-field"></a>

### `og.load.field`

Read a cooked file.field lattice by its canonical module/content ID without allocating world or GPU handles.

```ts
og.load.field(world: World, path: string): Result<FieldData>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose bundle holds the field. |
| `path` | `string` | no | The field's canonical module/content ID, such as base/fields/wind. |

**Returns** `Result<FieldData>`: Refused if missing, not resident, not a field, stale or invalid.

Binding symbol `og_load_field` · publish builds: kept

```ogt
fn field {
  doc "Read a cooked file.field lattice by its canonical module/content ID without allocating world or GPU handles."
  param world World "The world whose bundle holds the field."
  param path String "The field's canonical module/content ID, such as base/fields/wind."
  returns Result.FieldData "Refused if missing, not resident, not a field, stale or invalid."
}
```

<sub>Module `load` line 17</sub>
