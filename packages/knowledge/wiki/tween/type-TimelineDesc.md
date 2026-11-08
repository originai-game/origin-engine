<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TimelineDesc"></a>

### `TimelineDesc` (desc struct)

How a timeline plays (og.tween.timeline).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `defaults` | `TweenDefaults` | `{}` | — | — | — | What its tweens take when they leave it out. |
| `repeat` | `number` | `0` | ≥ -1 | — | — | Extra cycles of the whole timeline: 0 once, -1 forever. |
| `yoyo` | `boolean` | `false` | — | — | — | Every other cycle plays backwards. |
| `repeatDelay` | `number` | `0` | ≥ 0 | `s` | — | Seconds held between cycles. |
| `paused` | `boolean` | `false` | — | — | — | Created paused: play() starts it. Its from() values are still written. |
| `timeScale` | `number` | `1` | ≥ 0 | — | — | Playback rate: 2 twice as fast, 0.5 half. |
| `autoKill` | `boolean` | `false` | — | — | — | Removed when it completes, instead of staying to be reversed or restarted. |
| `unscaled` | `boolean` | `false` | — | — | — | Advance by real ticks, ignoring Time.scale and hit stop (UI motion). |
| `tag` | `number` | `0` | — | — | — | A number of yours, carried by og.TweenCompleted and og.TweenEvent. |
| `onStart` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent when the playhead leaves the start. |
| `onUpdate` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent every run the playhead moves. |
| `onComplete` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent when it reaches its end going forward. |
| `onRepeat` | `string \| null` | `null` | — | — | — | The og.TweenEvent sent when a cycle begins again. |

```ogt
struct TimelineDesc {
  doc "How a timeline plays (og.tween.timeline)."
  desc true
  field defaults TweenDefaults "What its tweens take when they leave it out."
  field repeat i32 "Extra cycles of the whole timeline: 0 once, -1 forever." {
    min -1
  }
  field yoyo bool "Every other cycle plays backwards."
  field repeatDelay f32 "Seconds held between cycles." {
    min 0
    unit s
  }
  field paused bool "Created paused: play() starts it. Its from() values are still written."
  field timeScale f32 "Playback rate: 2 twice as fast, 0.5 half." {
    default 1
    min 0
  }
  field autoKill bool "Removed when it completes, instead of staying to be reversed or restarted."
  field unscaled bool "Advance by real ticks, ignoring Time.scale and hit stop (UI motion)."
  field tag u32 "A number of yours, carried by og.TweenCompleted and og.TweenEvent."
  field onStart Option.String "The og.TweenEvent sent when the playhead leaves the start."
  field onUpdate Option.String "The og.TweenEvent sent every run the playhead moves."
  field onComplete Option.String "The og.TweenEvent sent when it reaches its end going forward."
  field onRepeat Option.String "The og.TweenEvent sent when a cycle begins again."
}
```

<sub>Module `tween` line 144</sub>
