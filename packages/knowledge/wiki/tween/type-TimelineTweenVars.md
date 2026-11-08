<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TimelineTweenVars"></a>

### `TimelineTweenVars` (desc struct)

One timeline tween of one field of every target (TweenTimeline.to, from, fromTo and set): GSAP's vars, with the field named.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `component` | `AnyComponent \| null` | `null` | — | — | — | The component; null takes the timeline's defaults, else og.Transform. |
| `field` | `string` | `""` | — | — | — | The numeric field or lane, as in TweenDesc. |
| `to` | `readonly number[] \| null` | `null` | — | — | — | The end value (to, fromTo, set); one for every lane, or one per lane. |
| `from` | `readonly number[] \| null` | `null` | — | — | — | The start value (from, fromTo). |
| `relative` | `boolean` | `false` | — | — | — | `to` is added to the start value. |
| `duration` | `number \| null` | `null` | — | `s` | — | Seconds per cycle; null takes the timeline's defaults. |
| `delay` | `number` | `0` | ≥ 0 | `s` | — | Seconds added to its position. |
| `ease` | `Ease \| null` | `null` | — | — | — | The curve; null takes the timeline's defaults. |
| `repeat` | `number` | `0` | ≥ -1 | — | — | Extra cycles: 0 once, -1 forever. |
| `yoyo` | `boolean` | `false` | — | — | — | Every other cycle plays backwards. |
| `repeatDelay` | `number` | `0` | ≥ 0 | `s` | — | Seconds held between cycles. |
| `stagger` | `number \| StaggerDesc \| null` | `null` | — | — | — | Several targets start one after another: seconds between them, or a StaggerDesc. |
| `keyframes` | `readonly TweenKeyframe[] \| null` | `null` | — | — | — | Steps played in order instead of `to` (repeat and yoyo then apply to the whole). |
| `onStart` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent when it starts (per target). |
| `onUpdate` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent every run it is written. |
| `onComplete` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent when it ends. |
| `onRepeat` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent when a cycle begins again. |

```ogt
struct TimelineTweenVars {
  doc "One timeline tween of one field of every target (TweenTimeline.to, from, fromTo and set): GSAP's vars, with the field named."
  desc true
  field component Option.AnyComponent "The component; null takes the timeline's defaults, else og.Transform."
  field field String "The numeric field or lane, as in TweenDesc."
  field to Option.List.f32 "The end value (to, fromTo, set); one for every lane, or one per lane."
  field from Option.List.f32 "The start value (from, fromTo)."
  field relative bool "`to` is added to the start value."
  field duration Option.f32 "Seconds per cycle; null takes the timeline's defaults." {
    unit s
  }
  field delay f32 "Seconds added to its position." {
    min 0
    unit s
  }
  field ease Option.Ease "The curve; null takes the timeline's defaults."
  field repeat i32 "Extra cycles: 0 once, -1 forever." {
    min -1
  }
  field yoyo bool "Every other cycle plays backwards."
  field repeatDelay f32 "Seconds held between cycles." {
    min 0
    unit s
  }
  field stagger Option.OneOf.f32.StaggerDesc "Several targets start one after another: seconds between them, or a StaggerDesc."
  field keyframes Option.List.TweenKeyframe "Steps played in order instead of `to` (repeat and yoyo then apply to the whole)."
  field onStart Option.String "The og.TweenEvent sent when it starts (per target)."
  field onUpdate Option.String "The og.TweenEvent sent every run it is written."
  field onComplete Option.String "The og.TweenEvent sent when it ends."
  field onRepeat Option.String "The og.TweenEvent sent when a cycle begins again."
}
```

<sub>Module `tween` line 200</sub>
