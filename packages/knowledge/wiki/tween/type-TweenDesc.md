<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TweenDesc"></a>

### `TweenDesc` (desc struct)

One tween of one field.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose component is tweened. |
| `component` | `AnyComponent \| null` | `null` | — | — | — | The component token (og.Transform, og.Sprite, a game's own); null is og.Transform. |
| `field` | `string` | `""` | — | — | — | The field, or one lane of it: `position`, `position.y`, `scale`, `color.r`, `opacity`. f32, f64, i32 and u32 fields (rounded), vectors, colours and quaternions (as a rotation) tween; bool, enum and handle fields do not. |
| `to` | `readonly number[]` | `[]` | — | — | — | The end value: one number for a scalar or for every lane, or one per lane. |
| `from` | `readonly number[] \| null` | `null` | — | — | — | The start value, written from the tick the tween is started and held through its delay; null takes the field's value when the tween starts (after its delay). |
| `relative` | `boolean` | `false` | — | — | — | `to` is added to the start value. |
| `duration` | `number` | `0.3` | ≥ 0 | `s` | — | Seconds from start to end (one cycle). |
| `delay` | `number` | `0` | ≥ 0 | `s` | — | Seconds before it starts. |
| `ease` | `Ease` | `"quadOut"` | — | — | — | The curve. |
| `loops` | `number` | `0` | ≥ -1 | — | — | Extra cycles: 0 plays once, n repeats n more times, -1 forever. |
| `yoyo` | `boolean` | `false` | — | — | — | Every other cycle plays backwards (pulses, bobbing). |
| `unscaled` | `boolean` | `false` | — | — | — | Advance by real ticks, ignoring Time.scale and hit stop (UI motion). |
| `tag` | `number` | `0` | — | — | — | A number of yours, carried by og.TweenCompleted. |

```ogt
struct TweenDesc {
  doc "One tween of one field."
  desc true
  field entity Entity "The entity whose component is tweened."
  field component Option.AnyComponent "The component token (og.Transform, og.Sprite, a game's own); null is og.Transform."
  field field String "The field, or one lane of it: `position`, `position.y`, `scale`, `color.r`, `opacity`. f32, f64, i32 and u32 fields (rounded), vectors, colours and quaternions (as a rotation) tween; bool, enum and handle fields do not."
  field to List.f32 "The end value: one number for a scalar or for every lane, or one per lane."
  field from Option.List.f32 "The start value, written from the tick the tween is started and held through its delay; null takes the field's value when the tween starts (after its delay)."
  field relative bool "`to` is added to the start value."
  field duration f32 "Seconds from start to end (one cycle)." {
    default 0.3
    min 0
    unit s
  }
  field delay f32 "Seconds before it starts." {
    min 0
    unit s
  }
  field ease Ease "The curve."
  field loops i32 "Extra cycles: 0 plays once, n repeats n more times, -1 forever." {
    min -1
  }
  field yoyo bool "Every other cycle plays backwards (pulses, bobbing)."
  field unscaled bool "Advance by real ticks, ignoring Time.scale and hit stop (UI motion)."
  field tag u32 "A number of yours, carried by og.TweenCompleted."
}
```

<sub>`api.ogt` line 12408</sub>
