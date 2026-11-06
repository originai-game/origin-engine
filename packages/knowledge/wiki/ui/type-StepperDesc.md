<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StepperDesc"></a>

### `StepperDesc` (desc struct)

A number stepper's range, step and digits.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | — | — | The smallest value; min equal to max (the zero value) is unbounded. |
| `max` | `number` | `0` | — | — | — | The largest value. |
| `step` | `number` | `0` | ≥ 0 | — | — | How much a press changes it; 0 is 1. |
| `decimals` | `number` | `0` | ≤ 6 | — | — | Digits after the point. |

```ogt
struct StepperDesc {
  doc "A number stepper's range, step and digits."
  desc true
  field min f32 "The smallest value; min equal to max (the zero value) is unbounded."
  field max f32 "The largest value."
  field step f32 "How much a press changes it; 0 is 1." {
    min 0
  }
  field decimals u32 "Digits after the point." {
    max 6
  }
}
```

<sub>`api.ogt` line 16690</sub>
