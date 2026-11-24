<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-castRay"></a>

### `og.physics.castRay`

The nearest collider along a ray in the live Rapier simulation. Before og.physics a system sees tick-start colliders; after it, this tick's stepped colliders. Between ticks it sees the last simulation state, not pending ECS edits. Before the first step queries initialize the simulation without advancing time; a restore immediately replaces it.

See `physics §4`.

Also known as: ray, line trace, hit test, mouse picking, Unity Physics.Raycast, Godot intersect_ray, Unreal LineTraceSingle

```ts
og.physics.castRay(world: World, desc?: RaycastDesc): Result<RaycastHit | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RaycastDesc` | yes | The ray and the filter. |

**Returns** `Result<RaycastHit | null>`: The hit, or null when nothing is hit; refused for a zero direction.

Binding symbol `og_physics_cast_ray` · publish builds: kept

```ogt
fn castRay {
  doc "The nearest collider along a ray in the live Rapier simulation. Before og.physics a system sees tick-start colliders; after it, this tick's stepped colliders. Between ticks it sees the last simulation state, not pending ECS edits. Before the first step queries initialize the simulation without advancing time; a restore immediately replaces it."
  guide "physics §4"
  aliases "ray, line trace, hit test, mouse picking, Unity Physics.Raycast, Godot intersect_ray, Unreal LineTraceSingle"
  param world World "The world."
  param desc RaycastDesc "The ray and the filter."
  returns Result.Option.RaycastHit "The hit, or null when nothing is hit; refused for a zero direction."
}
```

<sub>Module `physics` line 2387</sub>
