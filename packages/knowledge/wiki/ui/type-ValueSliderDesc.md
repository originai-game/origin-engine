<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ValueSliderDesc"></a>

### `ValueSliderDesc` (desc struct)

A slider: range, step, label, readout, ticks and marks.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | — | — | The range's low end. |
| `max` | `number` | `1` | — | — | — | Its high end. |
| `step` | `number` | `0` | — | — | — | Snap to multiples of this from min (0: continuous; keys step a twentieth). |
| `label` | `string` | `""` | — | — | — | Its name, shown above it; empty for none. |
| `showValue` | `boolean` | `true` | — | — | — | Its value at the label row's end. |
| `format` | `ValueFormat` | `{}` | — | — | — | How its value reads. |
| `ticks` | `boolean` | `false` | — | — | — | A tick at every step (for a stepped slider of at most 20 steps). |
| `marks` | `readonly SliderMark[]` | `[]` | — | — | — | Labelled points under the track. |
| `origin` | `number \| null` | `null` | — | — | — | Fill from this value rather than min (a balance, a bias: a bipolar slider). |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `bubble` | `boolean` | `true` | — | — | — | A bubble over the knob with the value while it is dragged or focused. |

```ogt
struct ValueSliderDesc {
  doc "A slider: range, step, label, readout, ticks and marks."
  desc true
  field min f32 "The range's low end."
  field max f32 "Its high end." {
    default 1
  }
  field step f32 "Snap to multiples of this from min (0: continuous; keys step a twentieth)."
  field label String "Its name, shown above it; empty for none."
  field showValue bool "Its value at the label row's end." {
    default true
  }
  field format ValueFormat "How its value reads."
  field ticks bool "A tick at every step (for a stepped slider of at most 20 steps)."
  field marks List.SliderMark "Labelled points under the track."
  field origin Option.f32 "Fill from this value rather than min (a balance, a bias: a bipolar slider)."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null grows."
  field bubble bool "A bubble over the knob with the value while it is dragged or focused." {
    default true
  }
}
```

<sub>`api.ogt` line 19498</sub>
