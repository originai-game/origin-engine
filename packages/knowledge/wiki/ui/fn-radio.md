<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-radio"></a>

### `og.ui.radio`

A radio group of `Option#i` rows: a ring, filled primary when chosen, and a label.

```ts
og.ui.radio(id: string, options: readonly string[], selected: number, desc?: RadioDesc): ChoiceResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `options` | `readonly string[]` | no | The option labels. |
| `selected` | `number` | no | The chosen index from game state. |
| `desc` | `RadioDesc` | yes | Row or column. |

**Returns** `ChoiceResult`.

Binding symbol `og_ui_radio` · publish builds: kept

```ogt
fn radio {
  doc "A radio group of `Option#i` rows: a ring, filled primary when chosen, and a label."
  param id String "Its id."
  param options List.String "The option labels."
  param selected u32 "The chosen index from game state."
  param desc RadioDesc "Row or column."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 17172</sub>
