<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-rangeSlider"></a>

### `og.ui.rangeSlider`

A range slider: two knobs on one track and the fill between them. A press on the track takes the nearer knob there and a drag moves it (never past the other, kept minGap apart); each knob is its own focus stop, Left and Right step it. Parts: `RangeSlider.track`, `RangeSlider.rail`, `RangeSlider.fill`, `RangeSlider.knob[low|high][pressed|focused]` (under a finger drawn inside `RangeSlider.reach`, its 44 px target). Two sliders to a screen reader (lowLabel, highLabel). Probe: `<id>/Track/Low`, `<id>/Track/High`.

```ts
og.ui.rangeSlider(id: string, low: number, high: number, desc?: RangeSliderDesc): RangeResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `low` | `number` | no | The low end from game state. |
| `high` | `number` | no | The high end from game state. |
| `desc` | `RangeSliderDesc` | yes | Its range, step and look. |

**Returns** `RangeResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn rangeSlider {
  doc "A range slider: two knobs on one track and the fill between them. A press on the track takes the nearer knob there and a drag moves it (never past the other, kept minGap apart); each knob is its own focus stop, Left and Right step it. Parts: `RangeSlider.track`, `RangeSlider.rail`, `RangeSlider.fill`, `RangeSlider.knob[low|high][pressed|focused]` (under a finger drawn inside `RangeSlider.reach`, its 44 px target). Two sliders to a screen reader (lowLabel, highLabel). Probe: `<id>/Track/Low`, `<id>/Track/High`."
  host false
  param id String "Its id."
  param low f32 "The low end from game state."
  param high f32 "The high end from game state."
  param desc RangeSliderDesc "Its range, step and look."
  returns RangeResult
}
```

<sub>Module `ui` line 3927</sub>
