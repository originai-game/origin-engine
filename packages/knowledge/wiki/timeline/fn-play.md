<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="fn-play"></a>

### `og.timeline.play`

Play the entity's og.Timeline from its time (0 after a stop or an end); systems may call it, it takes effect at this tick's og.timeline.

Also known as: play cutscene, start sequence, cinematic, Unity PlayableDirector.Play, Unreal Sequencer play, Godot AnimationPlayer.play

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
  aliases "play cutscene, start sequence, cinematic, Unity PlayableDirector.Play, Unreal Sequencer play, Godot AnimationPlayer.play"
  param world World "The world."
  param entity Entity "The entity with the og.Timeline."
  returns Result.Void "Refused for an entity without og.Timeline, or one whose asset is not loaded."
}
```

<sub>`api.ogt` line 26232</sub>
