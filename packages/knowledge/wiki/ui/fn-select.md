<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-select"></a>

### `og.ui.select`

A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it.

```ts
og.ui.select(id: string, options: readonly string[], selected: number, desc?: SelectDesc): ChoiceResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `options` | `readonly string[]` | no | The option labels. |
| `selected` | `number` | no | The chosen index from game state. |
| `desc` | `SelectDesc` | yes | Placeholder, width, disabled. |

**Returns** `ChoiceResult`.

Binding symbol `og_ui_select` · publish builds: kept

```ogt
fn select {
  doc "A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it."
  param id String "Its id."
  param options List.String "The option labels."
  param selected u32 "The chosen index from game state."
  param desc SelectDesc "Placeholder, width, disabled."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 17155</sub>
