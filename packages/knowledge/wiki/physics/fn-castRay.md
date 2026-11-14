<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-castRay"></a>

### `og.physics.castRay`

The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them.

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
  doc "The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them."
  guide "physics §4"
  aliases "ray, line trace, hit test, mouse picking, Unity Physics.Raycast, Godot intersect_ray, Unreal LineTraceSingle"
  param world World "The world."
  param desc RaycastDesc "The ray and the filter."
  returns Result.Option.RaycastHit "The hit, or null when nothing is hit; refused for a zero direction."
}
```

<sub>Module `physics` line 2004</sub>
