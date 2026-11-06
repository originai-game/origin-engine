<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

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

<sub>`api.ogt` line 26232</sub>
