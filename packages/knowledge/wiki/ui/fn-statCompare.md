<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-statCompare"></a>

### `og.ui.statCompare`

Stat-compare rows: current, new and the signed change with its mark.

```ts
og.ui.statCompare(id: string, desc?: StatCompareDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `StatCompareDesc` | yes | The stats. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn statCompare {
  doc "Stat-compare rows: current, new and the signed change with its mark."
  host false
  param id String "Its id."
  param desc StatCompareDesc "The stats."
  returns Void
}
```

<sub>`api.ogt` line 18794</sub>
