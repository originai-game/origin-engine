<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TweenFromDesc"></a>

### `TweenFromDesc` (desc struct)

Tween from an explicit value to the field captured when og.tween.from is called, before any delay. The start value is written on the first og.tween tick, not synchronously during the call.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose component is tweened. |
| `component` | `AnyComponent \| null` | `null` | — | — | — | The component token; null is og.Transform. |
| `field` | `string` | `""` | — | — | — | The numeric field or lane, as in TweenDesc. |
| `from` | `number \| readonly number[] \| null` | `null` | — | — | — | The explicit start value: one number for all lanes, or a list, one per lane. Held through the delay. Required: a desc without one is refused. |
| `duration` | `number` | `0.3` | ≥ 0 | `s` | — | Seconds per cycle. |
| `delay` | `number` | `0` | ≥ 0 | `s` | — | Seconds to hold the start value before interpolating. |
| `ease` | `Ease` | `"power1.out"` | — | — | — | The GSAP-named curve. |
| `repeat` | `number` | `0` | ≥ -1 | — | — | Extra cycles: 0 once, n more, -1 forever. |
| `yoyo` | `boolean` | `false` | — | — | — | Alternate cycles play backwards. |
| `unscaled` | `boolean` | `false` | — | — | — | Ignore Time.scale and hit stop. |
| `tag` | `number` | `0` | — | — | — | Carried by TweenCompleted. |

```ogt
struct TweenFromDesc {
  doc "Tween from an explicit value to the field captured when og.tween.from is called, before any delay. The start value is written on the first og.tween tick, not synchronously during the call."
  desc true
  field entity Entity "The entity whose component is tweened."
  field component Option.AnyComponent "The component token; null is og.Transform."
  field field String "The numeric field or lane, as in TweenDesc."
  field from Option.OneOf.f32.List.f32 "The explicit start value: one number for all lanes, or a list, one per lane. Held through the delay. Required: a desc without one is refused."
  field duration f32 "Seconds per cycle." {
    default 0.3
    min 0
    unit s
  }
  field delay f32 "Seconds to hold the start value before interpolating." {
    min 0
    unit s
  }
  field ease Ease "The GSAP-named curve."
  field repeat i32 "Extra cycles: 0 once, n more, -1 forever." {
    min -1
  }
  field yoyo bool "Alternate cycles play backwards."
  field unscaled bool "Ignore Time.scale and hit stop."
  field tag u32 "Carried by TweenCompleted."
}
```

<sub>Module `tween` line 78</sub>
