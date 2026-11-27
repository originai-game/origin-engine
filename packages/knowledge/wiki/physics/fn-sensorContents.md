<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-sensorContents"></a>

### `og.physics.sensorContents`

Every entity inside a sensor now, each once, sorted by entity id: who stands on a pressure plate, what lies in a pickup zone, how many are in a capture area. It is what og.SensorEntered and og.SensorExited have added up to, read from og.Contact (the pairs of the last step), so inside a system the system lists og.Contact in its reads; it is world state, the same after a restore and while a hit stop holds the step. Something that passes through a sensor within one step is never in it, and sends neither event.

Also known as: trigger contents, who is in the trigger, Godot get_overlapping_bodies, Unity OnTriggerStay, Unreal GetOverlappingActors, pressure plate occupants, entities in zone

```ts
og.physics.sensorContents(world: World, sensor: Entity): Result<readonly Entity[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `sensor` | `Entity` | no | The sensor: an entity whose og.Collider has `sensor` set. |

**Returns** `Result<readonly Entity[]>`: Empty when nothing is inside; refused for an entity without a sensor collider, and inside a system that does not read og.Contact.

Binding symbol `og_physics_sensor_contents` · publish builds: kept

```ogt
fn sensorContents {
  doc "Every entity inside a sensor now, each once, sorted by entity id: who stands on a pressure plate, what lies in a pickup zone, how many are in a capture area. It is what og.SensorEntered and og.SensorExited have added up to, read from og.Contact (the pairs of the last step), so inside a system the system lists og.Contact in its reads; it is world state, the same after a restore and while a hit stop holds the step. Something that passes through a sensor within one step is never in it, and sends neither event."
  aliases "trigger contents, who is in the trigger, Godot get_overlapping_bodies, Unity OnTriggerStay, Unreal GetOverlappingActors, pressure plate occupants, entities in zone"
  param world World "The world."
  param sensor Entity "The sensor: an entity whose og.Collider has `sensor` set."
  returns Result.List.Entity "Empty when nothing is inside; refused for an entity without a sensor collider, and inside a system that does not read og.Contact."
}
```

<sub>Module `physics` line 2430</sub>
