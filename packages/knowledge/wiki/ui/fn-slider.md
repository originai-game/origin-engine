<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-slider"></a>

### `og.ui.slider`

A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick.

```ts
og.ui.slider(id: string, value: number, desc?: SliderDesc): SliderResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value from game state. |
| `desc` | `SliderDesc` | yes | Range, step and width. |

**Returns** `SliderResult`.

Binding symbol `og_ui_slider` · publish builds: kept

```ogt
fn slider {
  doc "A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick."
  param id String "Its id."
  param value f32 "The value from game state."
  param desc SliderDesc "Range, step and width."
  returns SliderResult
}
```

<sub>`api.ogt` line 16392</sub>
