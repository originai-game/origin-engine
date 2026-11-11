<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-vehicleWheels"></a>

### `og.physics.vehicleWheels`

An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right; or its og.VehicleWheel children in entity order), for drawing wheel models.

```ts
og.physics.vehicleWheels(world: World, entity: Entity): Result<readonly WheelPose[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The vehicle's entity. |

**Returns** `Result<readonly WheelPose[]>`: Refused for an entity without a vehicle.

Binding symbol `og_physics_vehicle_wheels` · publish builds: kept

```ogt
fn vehicleWheels {
  doc "An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right; or its og.VehicleWheel children in entity order), for drawing wheel models."
  param world World "The world."
  param entity Entity "The vehicle's entity."
  returns Result.List.WheelPose "Refused for an entity without a vehicle."
}
```

<sub>Module `physics` line 2057</sub>
