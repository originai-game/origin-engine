<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-valueSlider"></a>

### `og.ui.valueSlider`

A slider: a track, a fill from min (or from an origin, both ways) and a knob. A press anywhere on the track jumps the knob there and a drag follows (a touch too; the knob grows under a finger), Left and Right step it while it has the focus (a held key or d-pad repeats), with ticks at its steps, labelled marks, a readout and a bubble over the knob while it moves. Plays sliderTick at each step passed (rising in pitch with the value). Parts: `Slider.track[hover|pressed|focused|disabled]`, `Slider.rail`, `Slider.fill`, `Slider.knob[pressed|focused]`, `Slider.core` (the knob's dot while held or focused), `Slider.tick[passed]`, `Slider.bubble`. Role slider with its range. Probe: `<id>`, `<id>/Track`, `<id>/Track/Knob`, `<id>/Head/Value`.

```ts
og.ui.valueSlider(id: string, value: number, desc?: ValueSliderDesc): ValueSliderResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value from game state. |
| `desc` | `ValueSliderDesc` | yes | Its range, step and look. |

**Returns** `ValueSliderResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn valueSlider {
  doc "A slider: a track, a fill from min (or from an origin, both ways) and a knob. A press anywhere on the track jumps the knob there and a drag follows (a touch too; the knob grows under a finger), Left and Right step it while it has the focus (a held key or d-pad repeats), with ticks at its steps, labelled marks, a readout and a bubble over the knob while it moves. Plays sliderTick at each step passed (rising in pitch with the value). Parts: `Slider.track[hover|pressed|focused|disabled]`, `Slider.rail`, `Slider.fill`, `Slider.knob[pressed|focused]`, `Slider.core` (the knob's dot while held or focused), `Slider.tick[passed]`, `Slider.bubble`. Role slider with its range. Probe: `<id>`, `<id>/Track`, `<id>/Track/Knob`, `<id>/Head/Value`."
  host false
  param id String "Its id."
  param value f32 "The value from game state."
  param desc ValueSliderDesc "Its range, step and look."
  returns ValueSliderResult
}
```

<sub>`api.ogt` line 19537</sub>
