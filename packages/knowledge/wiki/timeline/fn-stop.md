<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="fn-stop"></a>

### `og.timeline.stop`

Stop at 0: the camera is handed back, its sounds and music stop, its caption and cover go.

Also known as: stop cutscene, end sequence, skip cutscene

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
  aliases "stop cutscene, end sequence, skip cutscene"
  param world World "The world."
  param entity Entity "The entity with the og.Timeline."
  returns Result.Void "Refused for an entity without og.Timeline."
}
```

<sub>`api.ogt` line 26255</sub>
