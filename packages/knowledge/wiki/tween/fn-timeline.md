<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-timeline"></a>

### `og.tween.timeline`

A GSAP timeline: tweens, calls, labels and nested timelines placed on its own clock, played, paused, reversed and sought as one. It starts playing in this tick's og.tween run unless paused.

Also known as: gsap.timeline, GSAP timeline, tween sequence, stagger, keyframes, choreography, DOTween Sequence

```ts
og.tween.timeline(world: World, desc?: TimelineDesc): TweenTimeline
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TimelineDesc` | yes | Defaults, repeat, yoyo, paused, callbacks. |

**Returns** `TweenTimeline`: Refused settings are reported in its `error`; it still answers the timeline.

Binding symbol `og_tween_timeline` · publish builds: kept

```ogt
fn timeline {
  doc "A GSAP timeline: tweens, calls, labels and nested timelines placed on its own clock, played, paused, reversed and sought as one. It starts playing in this tick's og.tween run unless paused."
  aliases "gsap.timeline, GSAP timeline, tween sequence, stagger, keyframes, choreography, DOTween Sequence"
  param world World "The world."
  param desc TimelineDesc "Defaults, repeat, yoyo, paused, callbacks." {
    optional true
  }
  returns TweenTimeline "Refused settings are reported in its `error`; it still answers the timeline."
}
```

<sub>Module `tween` line 444</sub>
