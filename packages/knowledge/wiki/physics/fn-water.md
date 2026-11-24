<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-water"></a>

### `og.physics.water`

The highest water volume enclosing the world point, between its surface and depth below it: its entity, surface height, submerged depth, motion and slope. Null above, below or outside all volumes. Inside a system reads og.Water and og.Transform.

See `physics §8`.

Also known as: is underwater, water depth, in water, swimming, water current, flow velocity, wave normal, water at point

```ts
og.physics.water(world: World, point: Vector3): Result<WaterPoint | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `point` | `Vector3` | no | The point, in world space, in metres (an og.WaterLine reads x and y). |

**Returns** `Result<WaterPoint | null>`: Null over no water. An og.Water drawn over a mesh (`geometry`) is water only over its triangles, still, at their height.

Binding symbol `og_physics_water` · publish builds: kept

```ogt
fn water {
  doc "The highest water volume enclosing the world point, between its surface and depth below it: its entity, surface height, submerged depth, motion and slope. Null above, below or outside all volumes. Inside a system reads og.Water and og.Transform."
  guide "physics §8"
  aliases "is underwater, water depth, in water, swimming, water current, flow velocity, wave normal, water at point"
  param world World "The world."
  param point Vector3 "The point, in world space, in metres (an og.WaterLine reads x and y)."
  returns Result.Option.WaterPoint "Null over no water. An og.Water drawn over a mesh (`geometry`) is water only over its triangles, still, at their height."
}
```

<sub>Module `physics` line 2168</sub>
