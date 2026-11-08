<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="record-TweenEvent"></a>

### `og.TweenEvent` (event)

A timeline callback: a call crossed, or a tween or timeline started, updated, completed or repeated (read in the tick after).

Also known as: GSAP onComplete, onStart, onUpdate, onRepeat, timeline call, tween callback

Token `og.TweenEvent`, registered under the schema name `og.TweenEvent`; its value type is `TweenEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tween` | `Tween` | `0` | — | — | — | The timeline (TweenTimeline.tween). |
| `entity` | `Entity` | `0` | — | — | — | The tween's target; 0 for the timeline's own callbacks and calls. |
| `name` | `string` | `""` | — | — | — | The name the callback was given (at most 63 bytes). |
| `progress` | `number` | `0` | — | — | — | How far through its cycle it is, 0 to 1. |
| `tag` | `number` | `0` | — | — | — | The timeline's tag. |

```ogt
event TweenEvent {
  doc "A timeline callback: a call crossed, or a tween or timeline started, updated, completed or repeated (read in the tick after)."
  aliases "GSAP onComplete, onStart, onUpdate, onRepeat, timeline call, tween callback"
  field tween Tween "The timeline (TweenTimeline.tween)."
  field entity Entity "The tween's target; 0 for the timeline's own callbacks and calls."
  field name String "The name the callback was given (at most 63 bytes)."
  field progress f32 "How far through its cycle it is, 0 to 1."
  field tag u32 "The timeline's tag."
}
```

<sub>Module `tween` line 122</sub>
