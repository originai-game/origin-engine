<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RangeSliderDesc"></a>

### `RangeSliderDesc` (desc struct)

A range slider: two knobs on one track.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | — | — | The range's low end. |
| `max` | `number` | `1` | — | — | — | Its high end. |
| `step` | `number` | `0` | — | — | — | Snap to multiples of this (0: continuous). |
| `minGap` | `number` | `0` | — | — | — | The least distance between the knobs. |
| `label` | `string` | `""` | — | — | — | Its name; empty for none. |
| `showValue` | `boolean` | `true` | — | — | — | Both values at the label row's end. |
| `format` | `ValueFormat` | `{}` | — | — | — | How its values read. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `lowLabel` | `string` | `""` | — | — | — | The low knob's accessible name; empty is `Minimum`. |
| `highLabel` | `string` | `""` | — | — | — | The high knob's; empty is `Maximum`. |

```ogt
struct RangeSliderDesc {
  doc "A range slider: two knobs on one track."
  desc true
  field min f32 "The range's low end."
  field max f32 "Its high end." {
    default 1
  }
  field step f32 "Snap to multiples of this (0: continuous)."
  field minGap f32 "The least distance between the knobs."
  field label String "Its name; empty for none."
  field showValue bool "Both values at the label row's end." {
    default true
  }
  field format ValueFormat "How its values read."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null grows."
  field lowLabel String "The low knob's accessible name; empty is `Minimum`."
  field highLabel String "The high knob's; empty is `Maximum`."
}
```

<sub>`api.ogt` line 19546</sub>
