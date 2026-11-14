<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-waterHeight"></a>

### `og.physics.waterHeight`

The water's surface at a point (world y), as it is drawn: an og.Water's level, with its sea's waves (og.Water.wind) or its river's course at this tick, plus the waves bodies raised on an og.WaterSurface or og.WaterLine (at `x`; `z` ignored) as the last step left them. For boats that bob and spray that starts on the surface.

```ts
og.physics.waterHeight(world: World, entity: Entity, x: number, z: number): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The water's entity. |
| `x` | `number` | no | World x, in metres. |
| `z` | `number` | no | World z, in metres (3D). |

**Returns** `Result<number>`: Refused for an entity without water; outside the water, its rest level.

Binding symbol `og_physics_water_height` · publish builds: kept

```ogt
fn waterHeight {
  doc "The water's surface at a point (world y), as it is drawn: an og.Water's level, with its sea's waves (og.Water.wind) or its river's course at this tick, plus the waves bodies raised on an og.WaterSurface or og.WaterLine (at `x`; `z` ignored) as the last step left them. For boats that bob and spray that starts on the surface."
  param world World "The world."
  param entity Entity "The water's entity."
  param x f32 "World x, in metres."
  param z f32 "World z, in metres (3D)."
  returns Result.f32 "Refused for an entity without water; outside the water, its rest level."
}
```

<sub>Module `physics` line 1772</sub>
