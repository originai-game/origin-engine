<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `timeline` module

Timelines: cutscenes and sequences as data, Unity's Timeline and Godot's AnimationPlayer as an `.ogt` the agent reads and diffs. A `type timeline` file has a `duration` in seconds, `path "<name>" { ... }` blocks (as og.path's) and `track <kind> ["<entity>"] { key <seconds> { ... } }` blocks; `og cook` cooks it (importing caption files) and og.load.timeline loads it. Tracks: `camera` (position and target keys, or `rail "<path>"` with `at` 0..1 along it, or `follow "<entity>"` with offsets; fov; eased), `transform "<entity>"` (position, rotation in degrees or quaternion, scale; eased), `animation "<entity>"` (clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model), `audio` (sound, volume, pitch, bus), `music` (sound, volume, fadeIn, fadeOut, end, loop, bus), `caption` (text from `key` to `end` with a UI type step `style` and an `anchor`; `captions "narration.srt" [offset]` imports a SubRip file or word timestamps `.json`), `event` (`do "<command-channel action>"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it), `transition` (kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from, over og.ui's screen cover) and `marker` (`name`, sending og.TimelineMarker when played past).

Between two keys that set a value the later key's `ease` (og.tween's Ease names, linear when it gives none) mixes them; before the first and after the last the value is held. Every value is evaluated at the timeline's time, never accumulated over ticks, and that time is kept exactly (in steps of a thousandth of a tick), so seeking to a time and playing to it give the same world: a seek is how a video render starts at any frame.

An og.Timeline component plays one on its entity, run by the engine's og.timeline system after the game's systems (and tweens) and before og.anim and physics, so what it sets is what animation, physics and the frame see that tick. While one with a camera track plays (or is paused) it owns og.Camera, and hands back the camera it found when it stops or ends. Its event keys' actions run at the start of the next tick, as og.ai's commands do. The command channel plays them too: `timeline:play:<name>` (`timelines/<name>.ogt`, or an asset id), `timeline:seek:<seconds>[:<name>]`, `timeline:pause[:<name>]`, `timeline:resume[:<name>]`, `timeline:stop[:<name>]`.

- **TypeScript**: functions are the namespace `og.timeline`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module timeline {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 14541</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`TimelineAsset`](#type-TimelineAsset) | handle | A loaded timeline: its tracks, paths, sounds and clips. |
| [`TimelineAsset()`](#column-TimelineAsset) | column kind | A TimelineAsset handle field (0 is none). |
| [`og.Timeline`](#record-Timeline) | component | Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns. |
| [`og.TimelineStarted`](#record-TimelineStarted) | event | A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop). |
| [`og.TimelineMarker`](#record-TimelineMarker) | event | Playback passed a marker key (a seek over one does not send it). |
| [`og.TimelineEnded`](#record-TimelineEnded) | event | A timeline without loop reached its duration and stopped playing. |
| [`TimelineTrackInfo`](#type-TimelineTrackInfo) | struct | One track of a loaded timeline. |
| [`TimelineInfo`](#type-TimelineInfo) | struct | A loaded timeline. |
| [`og.timeline.info`](#fn-info) | function | What a loaded timeline holds. |
| [`og.timeline.play`](#fn-play) | function | Play the entity's og.Timeline from its time (0 after a stop or an end); systems may call it, it takes effect at this tick's og.timeline. |
| [`og.timeline.pause`](#fn-pause) | function | Hold the entity's og.Timeline where it is (it keeps the camera). |
| [`og.timeline.seek`](#fn-seek) | function | Jump to a time: this tick evaluates every track there exactly as playing to it would, without sending the markers or running the events passed (except `retroactive` ones). |
| [`og.timeline.stop`](#fn-stop) | function | Stop at 0: the camera is handed back, its sounds and music stop, its caption and cover go. |

## Functions

<a id="fn-info"></a>

### `og.timeline.info`

What a loaded timeline holds.

```ts
og.timeline.info(world: World, asset: TimelineAsset): Result<TimelineInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `asset` | `TimelineAsset` | no | The timeline. |

**Returns** `Result<TimelineInfo>`: Refused for a timeline not loaded in this world.

Binding symbol `og_timeline_info` · publish builds: kept

```ogt
fn info {
  doc "What a loaded timeline holds."
  param world World "The world."
  param asset TimelineAsset "The timeline."
  returns Result.TimelineInfo "Refused for a timeline not loaded in this world."
}
```

<sub>`api.ogt` line 14614</sub>

<a id="fn-play"></a>

### `og.timeline.play`

Play the entity's og.Timeline from its time (0 after a stop or an end); systems may call it, it takes effect at this tick's og.timeline.

```ts
og.timeline.play(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Timeline. |

**Returns** `Result<void>`: Refused for an entity without og.Timeline, or one whose asset is not loaded.

Binding symbol `og_timeline_play` · publish builds: kept

```ogt
fn play {
  doc "Play the entity's og.Timeline from its time (0 after a stop or an end); systems may call it, it takes effect at this tick's og.timeline."
  param world World "The world."
  param entity Entity "The entity with the og.Timeline."
  returns Result.Void "Refused for an entity without og.Timeline, or one whose asset is not loaded."
}
```

<sub>`api.ogt` line 14621</sub>

<a id="fn-pause"></a>

### `og.timeline.pause`

Hold the entity's og.Timeline where it is (it keeps the camera).

```ts
og.timeline.pause(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Timeline. |

**Returns** `Result<void>`: Refused for an entity without og.Timeline.

Binding symbol `og_timeline_pause` · publish builds: kept

```ogt
fn pause {
  doc "Hold the entity's og.Timeline where it is (it keeps the camera)."
  param world World "The world."
  param entity Entity "The entity with the og.Timeline."
  returns Result.Void "Refused for an entity without og.Timeline."
}
```

<sub>`api.ogt` line 14628</sub>

<a id="fn-seek"></a>

### `og.timeline.seek`

Jump to a time: this tick evaluates every track there exactly as playing to it would, without sending the markers or running the events passed (except `retroactive` ones).

```ts
og.timeline.seek(world: World, entity: Entity, time: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Timeline. |
| `time` | `number` | no | Seconds, 0 to the duration. |

**Returns** `Result<void>`: Refused for an entity without og.Timeline or a time outside 0 to its duration.

Binding symbol `og_timeline_seek` · publish builds: kept

```ogt
fn seek {
  doc "Jump to a time: this tick evaluates every track there exactly as playing to it would, without sending the markers or running the events passed (except `retroactive` ones)."
  param world World "The world."
  param entity Entity "The entity with the og.Timeline."
  param time f32 "Seconds, 0 to the duration."
  returns Result.Void "Refused for an entity without og.Timeline or a time outside 0 to its duration."
}
```

<sub>`api.ogt` line 14635</sub>

<a id="fn-stop"></a>

### `og.timeline.stop`

Stop at 0: the camera is handed back, its sounds and music stop, its caption and cover go.

```ts
og.timeline.stop(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Timeline. |

**Returns** `Result<void>`: Refused for an entity without og.Timeline.

Binding symbol `og_timeline_stop` · publish builds: kept

```ogt
fn stop {
  doc "Stop at 0: the camera is handed back, its sounds and music stop, its caption and cover go."
  param world World "The world."
  param entity Entity "The entity with the og.Timeline."
  returns Result.Void "Refused for an entity without og.Timeline."
}
```

<sub>`api.ogt` line 14643</sub>

## Components, resources and events

<a id="record-Timeline"></a>

### `og.Timeline` (component)

Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns.

Token `og.Timeline`, registered under the schema name `og.Timeline`; its value type is `Timeline`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline (og.load.timeline); 0 plays nothing. |
| `playing` | `boolean` | `false` | — | — | — | Advances while true. |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Where it is, in seconds; written every tick it plays, set it to seek. |
| `speed` | `number` | `1` | 0 … 16 | — | tunable | Playback rate: 1 as authored, 0.5 half speed (0 holds the time while playing). |
| `loop` | `boolean` | `false` | — | — | — | Start over at the end; otherwise stop playing there, hand the camera back and send og.TimelineEnded. |

```ogt
component Timeline {
  doc "Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns."
  field asset TimelineAsset "The timeline (og.load.timeline); 0 plays nothing."
  field playing bool "Advances while true."
  field time f32 "Where it is, in seconds; written every tick it plays, set it to seek." {
    min 0
    unit s
  }
  field speed f32 "Playback rate: 1 as authored, 0.5 half speed (0 holds the time while playing)." {
    default 1
    min 0
    max 16
    tunable true
  }
  field loop bool "Start over at the end; otherwise stop playing there, hand the camera back and send og.TimelineEnded."
}
```

<sub>`api.ogt` line 14556</sub>

<a id="record-TimelineStarted"></a>

### `og.TimelineStarted` (event)

A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop).

Token `og.TimelineStarted`, registered under the schema name `og.TimelineStarted`; its value type is `TimelineStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Timeline's entity. |
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline. |

```ogt
event TimelineStarted {
  doc "A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop)."
  field entity Entity "The og.Timeline's entity."
  field asset TimelineAsset "The timeline."
}
```

<sub>`api.ogt` line 14573</sub>

<a id="record-TimelineMarker"></a>

### `og.TimelineMarker` (event)

Playback passed a marker key (a seek over one does not send it).

Token `og.TimelineMarker`, registered under the schema name `og.TimelineMarker`; its value type is `TimelineMarker`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Timeline's entity. |
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline. |
| `name` | `string` | `""` | — | — | — | The marker's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The marker's time. |

```ogt
event TimelineMarker {
  doc "Playback passed a marker key (a seek over one does not send it)."
  field entity Entity "The og.Timeline's entity."
  field asset TimelineAsset "The timeline."
  field name String "The marker's name (at most 63 bytes)."
  field time f32 "The marker's time." {
    unit s
  }
}
```

<sub>`api.ogt` line 14579</sub>

<a id="record-TimelineEnded"></a>

### `og.TimelineEnded` (event)

A timeline without loop reached its duration and stopped playing.

Token `og.TimelineEnded`, registered under the schema name `og.TimelineEnded`; its value type is `TimelineEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Timeline's entity. |
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline. |

```ogt
event TimelineEnded {
  doc "A timeline without loop reached its duration and stopped playing."
  field entity Entity "The og.Timeline's entity."
  field asset TimelineAsset "The timeline."
}
```

<sub>`api.ogt` line 14589</sub>

## Types

<a id="type-TimelineAsset"></a>

### `TimelineAsset` (handle)

A loaded timeline: its tracks, paths, sounds and clips.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TimelineAsset "A loaded timeline: its tracks, paths, sounds and clips."
```

<sub>`api.ogt` line 14547</sub>

<a id="type-TimelineTrackInfo"></a>

### `TimelineTrackInfo` (struct)

One track of a loaded timeline.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | camera, transform, animation, audio, music, caption, event, transition or marker. |
| `target` | `string` | `""` | — | — | — | The entity a transform or animation track drives, the path a rail rides, the entity a camera follows; empty otherwise. |
| `keys` | `number` | `0` | — | — | — | Its keys. |

```ogt
struct TimelineTrackInfo {
  doc "One track of a loaded timeline."
  field kind String "camera, transform, animation, audio, music, caption, event, transition or marker."
  field target String "The entity a transform or animation track drives, the path a rail rides, the entity a camera follows; empty otherwise."
  field keys u32 "Its keys."
}
```

<sub>`api.ogt` line 14595</sub>

<a id="type-TimelineInfo"></a>

### `TimelineInfo` (struct)

A loaded timeline.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `TimelineAsset` | `0` | — | — | — | Its handle. |
| `name` | `string` | `""` | — | — | — | Its `name`. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `tracks` | `readonly TimelineTrackInfo[]` | `[]` | — | — | — | Its tracks, in file order. |
| `paths` | `readonly string[]` | `[]` | — | — | — | The paths it declares. |
| `markers` | `readonly string[]` | `[]` | — | — | — | Its markers' names, in time order. |

```ogt
struct TimelineInfo {
  doc "A loaded timeline."
  field asset TimelineAsset "Its handle."
  field name String "Its `name`."
  field duration f32 "Its length." {
    unit s
  }
  field tracks List.TimelineTrackInfo "Its tracks, in file order."
  field paths List.String "The paths it declares."
  field markers List.String "Its markers' names, in time order."
}
```

<sub>`api.ogt` line 14602</sub>

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

<sub>`api.ogt` line 14549</sub>
