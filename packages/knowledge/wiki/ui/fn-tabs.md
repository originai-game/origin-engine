<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-tabs"></a>

### `og.ui.tabs`

A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels.

```ts
og.ui.tabs(id: string, items: readonly string[], selected: number, desc?: TabsDesc): ChoiceResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `items` | `readonly string[]` | no | The tab labels. |
| `selected` | `number` | no | The chosen index from game state. |
| `desc` | `TabsDesc` | yes | Fill and width. |

**Returns** `ChoiceResult`.

Binding symbol `og_ui_tabs` · publish builds: kept

```ogt
fn tabs {
  doc "A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels."
  param id String "Its id."
  param items List.String "The tab labels."
  param selected u32 "The chosen index from game state."
  param desc TabsDesc "Fill and width."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 17131</sub>
