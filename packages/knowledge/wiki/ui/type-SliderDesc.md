<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SliderDesc"></a>

### `SliderDesc` (desc struct)

A slider's range and width.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | — | — | The smallest value; min equal to max (the zero value) means 0 to 1. |
| `max` | `number` | `0` | — | — | — | The largest value. |
| `step` | `number` | `0` | ≥ 0 | — | — | Snap to multiples of step from min; 0 is continuous. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |

```ogt
struct SliderDesc {
  doc "A slider's range and width."
  desc true
  field min f32 "The smallest value; min equal to max (the zero value) means 0 to 1."
  field max f32 "The largest value."
  field step f32 "Snap to multiples of step from min; 0 is continuous." {
    min 0
  }
  field width Option.Size "Its width; null grows."
}
```

<sub>`api.ogt` line 16000</sub>
