<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `timeline` module

Timelines: cutscenes and sequences as data, Unity's Timeline and Godot's AnimationPlayer as an `.ogt` the agent reads and diffs. A `type timeline` file has a `duration` in seconds, `path "<name>" { ... }` blocks (as og.path's) and `track <kind> ["<entity>"] { key <seconds> { ... } }` blocks; `og cook` cooks it (importing caption files) and og.load.timeline loads it. Tracks: `camera` (position and target keys, or `rail "<path>"` with `at` 0..1 along it, or `follow "<entity>"` with offsets; fov; eased), `transform "<entity>"` (position, rotation in degrees or quaternion, scale; eased), `animation "<entity>"` (clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model), `audio` (sound, volume, pitch, bus), `music` (sound, volume, fadeIn, fadeOut, end, loop, bus), `caption` (text from `key` to `end` with a UI type step `style` and an `anchor`; `captions "narration.srt" [offset]` imports a SubRip file or word timestamps `.json`), `event` (`do "<command-channel action>"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it), `transition` (kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from, over og.ui's screen cover) `marker` (`name`, sending og.TimelineMarker when played past), `property ["<entity>"]` (`field "og.PointLight.distance"`, keys of `value`: any reflected field of a component on the entity, or of a resource without one), `light "<entity>"` (its og.DirectionalLight, og.PointLight, og.SpotLight, og.RectLight or og.Light2D: color, intensity, distance, angle, ...), `visibility "<entity>"` (`visible true|false` of whatever it draws, stepped), `post` (og.PostEffects' fields), `blendshape "<entity>"` (og.Blendshapes w0 to w7; its source becomes component), `material "<entity>"` (its og.Mesh's material, or its og.Model's material `index N`: StandardMaterialDesc fields by name, other names the material shader's parameters) and `timeline` (`play "<timeline>"`, speed, loop: from its key until the next, another timeline plays nested at (time − key) × speed, its markers sent as `<file>/<marker>`, at most four deep) and `shot` (`camera "<entity>"`, a virtual camera's name, or `none`; `blend` seconds and `ease`: from its key until the next, that og.VirtualCamera is og.CameraBrain.solo, blended to over the key's blend by its ease or by the camera's own; `none` and the timeline's end hand back the solo it found). Camera keys also take `roll` (degrees about the view direction), `focus` (a distance) or `focusOn "<entity>"` and `focusRange`, which set og.PostEffects' depth of field while the track owns the camera. The cook checks every field against the engine's records (a field the record lacks, one a timeline cannot drive, a value of the wrong shape) and refuses nesting in a cycle or deeper than four; a track kind this engine does not read is refused naming it and the engine's version.

Between two keys that set a value the later key's `ease` (og.tween's Ease names, linear when it gives none) mixes them; before the first and after the last the value is held. Every value is evaluated at the timeline's time, never accumulated over ticks, and that time is kept exactly (in steps of a thousandth of a tick), so seeking to a time and playing to it give the same world: a seek is how a video render starts at any frame.

An og.Timeline component plays one on its entity, run by the engine's og.timeline system after the game's systems (and tweens) and before og.anim and physics, so what it sets is what animation, physics and the frame see that tick. While one with a camera track plays (or is paused) it owns og.Camera, and hands back the camera it found when it stops or ends. Its event keys' actions run at the start of the next tick, as og.ai's commands do. Loading a timeline (og.load.timeline, outside a system) grants og.timeline the records its tracks write; values they wrote stay when it stops, while the camera, its depth of field and the materials go back. A material track is presentation, not world state: it is applied every frame from the timeline's time, so a restored save draws the right material on its next frame. The command channel plays them too: `timeline:play:<name>` (`timelines/<name>.ogt`, or an asset id), `timeline:seek:<seconds>[:<name>]`, `timeline:pause[:<name>]`, `timeline:resume[:<name>]`, `timeline:stop[:<name>]`.

- **TypeScript**: functions are the namespace `og.timeline`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module timeline {
  doc "Timelines: cutscenes and sequences as data, Unity's Timeline and Godot's AnimationPlayer as an `.ogt` the agent reads and diffs. A `type timeline` file has a `duration` in seconds, `path \"<name>\" { ... }` blocks (as og.path's) and `track <kind> [\"<entity>\"] { key <seconds> { ... } }` blocks; `og cook` cooks it (importing caption files) and og.load.timeline loads it. Tracks: `camera` (position and target keys, or `rail \"<path>\"` with `at` 0..1 along it, or `follow \"<entity>\"` with offsets; fov; eased), `transform \"<entity>\"` (position, rotation in degrees or quaternion, scale; eased), `animation \"<entity>\"` (clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model), `audio` (sound, volume, pitch, bus), `music` (sound, volume, fadeIn, fadeOut, end, loop, bus), `caption` (text from `key` to `end` with a UI type step `style` and an `anchor`; `captions \"narration.srt\" [offset]` imports a SubRip file or word timestamps `.json`), `event` (`do \"<command-channel action>\"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it), `transition` (kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from, over og.ui's screen cover) `marker` (`name`, sending og.TimelineMarker when played past), `property [\"<entity>\"]` (`field \"og.PointLight.distance\"`, keys of `value`: any reflected field of a component on the entity, or of a resource without one), `light \"<entity>\"` (its og.DirectionalLight, og.PointLight, og.SpotLight, og.RectLight or og.Light2D: color, intensity, distance, angle, ...), `visibility \"<entity>\"` (`visible true|false` of whatever it draws, stepped), `post` (og.PostEffects' fields), `blendshape \"<entity>\"` (og.Blendshapes w0 to w7; its source becomes component), `material \"<entity>\"` (its og.Mesh's material, or its og.Model's material `index N`: StandardMaterialDesc fields by name, other names the material shader's parameters) and `timeline` (`play \"<timeline>\"`, speed, loop: from its key until the next, another timeline plays nested at (time − key) × speed, its markers sent as `<file>/<marker>`, at most four deep) and `shot` (`camera \"<entity>\"`, a virtual camera's name, or `none`; `blend` seconds and `ease`: from its key until the next, that og.VirtualCamera is og.CameraBrain.solo, blended to over the key's blend by its ease or by the camera's own; `none` and the timeline's end hand back the solo it found). Camera keys also take `roll` (degrees about the view direction), `focus` (a distance) or `focusOn \"<entity>\"` and `focusRange`, which set og.PostEffects' depth of field while the track owns the camera. The cook checks every field against the engine's records (a field the record lacks, one a timeline cannot drive, a value of the wrong shape) and refuses nesting in a cycle or deeper than four; a track kind this engine does not read is refused naming it and the engine's version."
  doc "Between two keys that set a value the later key's `ease` (og.tween's Ease names, linear when it gives none) mixes them; before the first and after the last the value is held. Every value is evaluated at the timeline's time, never accumulated over ticks, and that time is kept exactly (in steps of a thousandth of a tick), so seeking to a time and playing to it give the same world: a seek is how a video render starts at any frame."
  doc "An og.Timeline component plays one on its entity, run by the engine's og.timeline system after the game's systems (and tweens) and before og.anim and physics, so what it sets is what animation, physics and the frame see that tick. While one with a camera track plays (or is paused) it owns og.Camera, and hands back the camera it found when it stops or ends. Its event keys' actions run at the start of the next tick, as og.ai's commands do. Loading a timeline (og.load.timeline, outside a system) grants og.timeline the records its tracks write; values they wrote stay when it stops, while the camera, its depth of field and the materials go back. A material track is presentation, not world state: it is applied every frame from the timeline's time, so a restored save draws the right material on its next frame. The command channel plays them too: `timeline:play:<name>` (`timelines/<name>.ogt`, or an asset id), `timeline:seek:<seconds>[:<name>]`, `timeline:pause[:<name>]`, `timeline:resume[:<name>]`, `timeline:stop[:<name>]`."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 26130</sub>

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

<sub>`api.ogt` line 26137</sub>
