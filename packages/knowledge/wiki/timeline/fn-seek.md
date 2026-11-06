<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

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

<sub>`api.ogt` line 26247</sub>
