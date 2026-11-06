<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-checkboxList"></a>

### `og.ui.checkboxList`

Checkboxes in a column (`Option#i`).

```ts
og.ui.checkboxList(id: string, labels: readonly string[], values: readonly boolean[]): CheckListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `labels` | `readonly string[]` | no | The labels. |
| `values` | `readonly boolean[]` | no | The states from game state. |

**Returns** `CheckListResult`.

Binding symbol `og_ui_checkbox_list` · publish builds: kept

```ogt
fn checkboxList {
  doc "Checkboxes in a column (`Option#i`)."
  param id String "Its id."
  param labels List.String "The labels."
  param values List.bool "The states from game state."
  returns CheckListResult
}
```

<sub>`api.ogt` line 17175</sub>
