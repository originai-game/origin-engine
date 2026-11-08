<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TweenKeyframe"></a>

### `TweenKeyframe` (desc struct)

One step of a tween's keyframes: from where the step before ended to `to`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `to` | `readonly number[]` | `[]` | — | — | — | The value at the step's end. |
| `duration` | `number \| null` | `null` | — | `s` | — | Seconds; null takes the tween's. |
| `ease` | `Ease \| null` | `null` | — | — | — | The curve; null takes the tween's. |
| `delay` | `number` | `0` | ≥ 0 | `s` | — | Seconds after the step before. |

```ogt
struct TweenKeyframe {
  doc "One step of a tween's keyframes: from where the step before ended to `to`."
  desc true
  field to List.f32 "The value at the step's end."
  field duration Option.f32 "Seconds; null takes the tween's." {
    unit s
  }
  field ease Option.Ease "The curve; null takes the tween's."
  field delay f32 "Seconds after the step before." {
    min 0
    unit s
  }
}
```

<sub>Module `tween` line 186</sub>
