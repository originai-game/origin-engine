<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LabelDesc"></a>

### `LabelDesc` (desc struct)

How a label draws its text.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
| `onScene` | `boolean` | `false` | — | — | — | Text over the scene: uses sceneInk when color is null and draws the scheme's sceneShadow separation. A HUD plate uses background: { token: 'scenePlate' } on its container. |
| `uppercase` | `boolean` | `false` | — | — | — | Uppercase in the UI's locale before it is measured (Turkish i to İ, Greek without its accents), as a type role with `case upper` is; the probe and screen readers keep the source text. Never uppercase in game code: String.toUpperCase knows no locale. |
| `align` | `TextAlign` | `"start"` | — | — | — | How the lines align. |
| `nowrap` | `boolean` | `false` | — | — | — | Keep to one line (breaking only at `\n`). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font. |
| `heading` | `boolean` | `false` | — | — | — | A heading: a style with a flourish frames it. |
| `ellipsis` | `boolean` | `false` | — | — | — | Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text. |
| `maxLines` | `number` | `0` | — | — | — | The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out. |
| `fitMin` | `TypeStepName \| null` | `null` | — | — | — | Auto-fit: where the text does not fit its width (one line with nowrap, else its words and maxLines), it takes the ramp's smaller sizes, largest first, down to this step, keeping its own step's tracking and features; at the smallest, ellipsis cuts what still does not fit. Null keeps its step. A translation longer than the source (German on a button) shrinks instead of wrapping or overflowing. |

```ogt
struct LabelDesc {
  doc "How a label draws its text."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field color Option.Paint "The ink; null is ink."
  field onScene bool "Text over the scene: uses sceneInk when color is null and draws the scheme's sceneShadow separation. A HUD plate uses background: { token: 'scenePlate' } on its container."
  field uppercase bool "Uppercase in the UI's locale before it is measured (Turkish i to İ, Greek without its accents), as a type role with `case upper` is; the probe and screen readers keep the source text. Never uppercase in game code: String.toUpperCase knows no locale."
  field align TextAlign "How the lines align."
  field nowrap bool "Keep to one line (breaking only at `\\n`)."
  field width Option.Size "Its width; null fits the text."
  field font FontRole "Which font."
  field heading bool "A heading: a style with a flourish frames it."
  field ellipsis bool "Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text."
  field maxLines u32 "The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out."
  field fitMin Option.TypeStepName "Auto-fit: where the text does not fit its width (one line with nowrap, else its words and maxLines), it takes the ramp's smaller sizes, largest first, down to this step, keeping its own step's tracking and features; at the smallest, ellipsis cuts what still does not fit. Null keeps its step. A translation longer than the source (German on a button) shrinks instead of wrapping or overflowing."
}
```

<sub>`api.ogt` line 15805</sub>
