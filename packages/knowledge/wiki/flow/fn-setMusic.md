<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-setMusic"></a>

### `og.flow.setMusic`

Start, crossfade, or stop persistent cross-segment background music with automatic ducking over segment audio. Systems may call it.

```ts
og.flow.setMusic(world: World, flow: Entity, desc?: FlowMusicDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `desc` | `FlowMusicDesc` | yes | The sound asset id (empty stops), volume, crossfade seconds, and ducking ratio. |

**Returns** `Result<void>`: Refused for an entity that is no flow.

Binding symbol `og_flow_set_music` · publish builds: kept

```ogt
fn setMusic {
  doc "Start, crossfade, or stop persistent cross-segment background music with automatic ducking over segment audio. Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  param desc FlowMusicDesc "The sound asset id (empty stops), volume, crossfade seconds, and ducking ratio."
  returns Result.Void "Refused for an entity that is no flow."
}
```

<sub>Module `flow` line 406</sub>
