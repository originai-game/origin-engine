<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-water"></a>

### `og.physics.water`

The water at a point: of every og.Water (a lake, a sea, a river), og.WaterSurface and og.WaterLine over it, the one whose surface stands highest, with the point's depth under it and the water's motion and slope there, at this tick. One call answers whether something is in water, how deep, and where the water carries it: swimming, a float that tilts with the waves, spray that drifts with the current, a camera that knows it is under. Inside a system it reads og.Water and og.Transform.

See `physics §8`.

Also known as: is underwater, water depth, in water, swimming, water current, flow velocity, wave normal, water at point

```ts
og.physics.water(world: World, point: Vector3): Result<WaterPoint | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `point` | `Vector3` | no | The point, in world space, in metres (an og.WaterLine reads x and y). |

**Returns** `Result<WaterPoint | null>`: Null over no water; an og.Water drawn over a mesh (`geometry`) is not looked at.

Binding symbol `og_physics_water` · publish builds: kept

```ogt
fn water {
  doc "The water at a point: of every og.Water (a lake, a sea, a river), og.WaterSurface and og.WaterLine over it, the one whose surface stands highest, with the point's depth under it and the water's motion and slope there, at this tick. One call answers whether something is in water, how deep, and where the water carries it: swimming, a float that tilts with the waves, spray that drifts with the current, a camera that knows it is under. Inside a system it reads og.Water and og.Transform."
  guide "physics §8"
  aliases "is underwater, water depth, in water, swimming, water current, flow velocity, wave normal, water at point"
  param world World "The world."
  param point Vector3 "The point, in world space, in metres (an og.WaterLine reads x and y)."
  returns Result.Option.WaterPoint "Null over no water; an og.Water drawn over a mesh (`geometry`) is not looked at."
}
```

<sub>Module `physics` line 1799</sub>
