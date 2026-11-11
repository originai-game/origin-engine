<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `timeline` module

Timelines: cutscenes and sequences as data, as an `.ogt` the agent reads and diffs (Unity's Timeline, Godot's AnimationPlayer): a `type timeline` file of keyed tracks (camera, transform, animation, audio, captions, events, shots, ...), cooked by `og cook` and loaded with og.load.timeline.

An og.Timeline component plays one on its entity, after the game's systems and tweens, before og.anim and physics. Every value is evaluated at the timeline's time, so seeking to a time and playing to it give the same world. A timeline with a camera track owns og.Camera while it plays or is paused.

See `animation §13`.

- **TypeScript**: functions are the namespace `og.timeline`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module timeline {
  doc "Timelines: cutscenes and sequences as data, as an `.ogt` the agent reads and diffs (Unity's Timeline, Godot's AnimationPlayer): a `type timeline` file of keyed tracks (camera, transform, animation, audio, captions, events, shots, ...), cooked by `og cook` and loaded with og.load.timeline."
  doc "An og.Timeline component plays one on its entity, after the game's systems and tweens, before og.anim and physics. Every value is evaluated at the timeline's time, so seeking to a time and playing to it give the same world. A timeline with a camera track owns og.Camera while it plays or is paused."
  # declarations follow, each on its own page
```

<sub>Module `timeline` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`TimelineAsset`](timeline/type-TimelineAsset.md) | handle |  | A loaded timeline: its tracks, paths, sounds and clips. |
| [`TimelineAsset()`](#column-TimelineAsset) | column kind |  | A TimelineAsset handle field (0 is none). |
| [`og.Timeline`](timeline/record-Timeline.md) | component |  | Plays a loaded timeline. |
| [`og.TimelineStarted`](timeline/record-TimelineStarted.md) | event |  | A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop). |
| [`og.TimelineMarker`](timeline/record-TimelineMarker.md) | event |  | Playback passed a marker key (a seek over one does not send it). |
| [`og.TimelineEnded`](timeline/record-TimelineEnded.md) | event |  | A timeline without loop reached its duration and stopped playing. |
| [`TimelineTrackInfo`](timeline/type-TimelineTrackInfo.md) | struct |  | One track of a loaded timeline. |
| [`TimelineInfo`](timeline/type-TimelineInfo.md) | struct |  | A loaded timeline. |
| [`og.timeline.info`](timeline/fn-info.md) | function | `(world: World, asset: TimelineAsset): Result<TimelineInfo>` | What a loaded timeline holds. |
| [`og.timeline.play`](timeline/fn-play.md) | function | `(world: World, entity: Entity): Result<void>` | Play the entity's og.Timeline from its time (0 after a stop or an end); systems may call it, it takes effect at this tick's og.timeline. |
| [`og.timeline.pause`](timeline/fn-pause.md) | function | `(world: World, entity: Entity): Result<void>` | Hold the entity's og.Timeline where it is (it keeps the camera). |
| [`og.timeline.seek`](timeline/fn-seek.md) | function | `(world: World, entity: Entity, time: number): Result<void>` | Jump to a time: this tick evaluates every track there exactly as playing to it would, without sending the markers or running the events passed (except `retroactive` ones). |
| [`og.timeline.stop`](timeline/fn-stop.md) | function | `(world: World, entity: Entity): Result<void>` | Stop at 0: the camera is handed back, its sounds and music stop, its caption and cover go. |

## Column kinds

Each is a factory, `TimelineAsset(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-TimelineAsset"></a>`TimelineAsset` | `TimelineAsset` | `Uint32Array` | 2 | A TimelineAsset handle field (0 is none). |

```ogt
column TimelineAsset {
  doc "A TimelineAsset handle field (0 is none)."
  array Uint32Array
  stride 2
  value TimelineAsset
}
```

<sub>Module `timeline` line 8</sub>
