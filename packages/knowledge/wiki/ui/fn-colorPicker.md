<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-colorPicker"></a>

### `og.ui.colorPicker`

A colour picker: a saturation and brightness square with a ring at the colour, a hue strip, an opacity strip, swatches and a hex field, with the colour before and after. A press or a drag on the square or a strip sets it (a touch too); Left and Right step the hue strip, the opacity strip and the square's saturation while they have the focus; swatches are buttons; a hex field takes `#rgb`, `#rrggbb` and `#rrggbbaa`. The hue is kept while the colour is grey (UI state). Parts: `ColorPicker.square[focused]`, `ColorPicker.hue` and `ColorPicker.alpha` (the strips' reach, 44 px tall under a finger) with `ColorPicker.hueBar` and `ColorPicker.alphaBar` (the strips drawn in them), `ColorPicker.thumb[focused]`, `ColorPicker.swatch[selected][hover|focused]`, `ColorPicker.preview`. Roles: two sliders and a slider-like square (its hex as its value), swatches as radios. Probe: `<id>/Square`, `<id>/Hue`, `<id>/Alpha`, `<id>/Swatches/Swatch#i`, `<id>/Bottom/Hex/Input/Frame/Field`.

```ts
og.ui.colorPicker(id: string, color?: PickedColor, desc?: ColorPickerDesc): ColorPickerResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `color` | `PickedColor` | yes | The colour from game state. |
| `desc` | `ColorPickerDesc` | yes | Its swatches and parts. |

**Returns** `ColorPickerResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn colorPicker {
  doc "A colour picker: a saturation and brightness square with a ring at the colour, a hue strip, an opacity strip, swatches and a hex field, with the colour before and after. A press or a drag on the square or a strip sets it (a touch too); Left and Right step the hue strip, the opacity strip and the square's saturation while they have the focus; swatches are buttons; a hex field takes `#rgb`, `#rrggbb` and `#rrggbbaa`. The hue is kept while the colour is grey (UI state). Parts: `ColorPicker.square[focused]`, `ColorPicker.hue` and `ColorPicker.alpha` (the strips' reach, 44 px tall under a finger) with `ColorPicker.hueBar` and `ColorPicker.alphaBar` (the strips drawn in them), `ColorPicker.thumb[focused]`, `ColorPicker.swatch[selected][hover|focused]`, `ColorPicker.preview`. Roles: two sliders and a slider-like square (its hex as its value), swatches as radios. Probe: `<id>/Square`, `<id>/Hue`, `<id>/Alpha`, `<id>/Swatches/Swatch#i`, `<id>/Bottom/Hex/Input/Frame/Field`."
  host false
  param id String "Its id."
  param color PickedColor "The colour from game state."
  param desc ColorPickerDesc "Its swatches and parts."
  returns ColorPickerResult
}
```

<sub>`api.ogt` line 19839</sub>
