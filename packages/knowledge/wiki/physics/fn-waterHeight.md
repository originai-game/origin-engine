<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-waterHeight"></a>

### `og.physics.waterHeight`

The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it, plus an og.Water's sea (og.Water.wind) at this tick, the surface it draws: for boats that bob and spray that starts on the surface.

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
  doc "The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it, plus an og.Water's sea (og.Water.wind) at this tick, the surface it draws: for boats that bob and spray that starts on the surface."
  param world World "The world."
  param entity Entity "The water's entity."
  param x f32 "World x, in metres."
  param z f32 "World z, in metres (3D)."
  returns Result.f32 "Refused for an entity without water; outside the water, its rest level."
}
```

<sub>Module `physics` line 2049</sub>
