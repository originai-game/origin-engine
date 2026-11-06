<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-keyValueRows"></a>

### `og.ui.keyValueRows`

Key-value rows: labels at the start, values at the end.

```ts
og.ui.keyValueRows(id: string, desc?: KeyValueRowsDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `KeyValueRowsDesc` | yes | The rows. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn keyValueRows {
  doc "Key-value rows: labels at the start, values at the end."
  host false
  param id String "Its id."
  param desc KeyValueRowsDesc "The rows."
  returns Void
}
```

<sub>`api.ogt` line 18876</sub>
