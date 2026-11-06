<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-overlap"></a>

### `og.physics.overlap`

Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast.

Also known as: overlap sphere, area query, shapes in range, Unity Physics.OverlapSphere, Godot intersect_shape

```ts
og.physics.overlap(world: World, desc?: OverlapDesc): Result<readonly Entity[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `OverlapDesc` | yes | The shape, its placement and the filter. |

**Returns** `Result<readonly Entity[]>`.

Binding symbol `og_physics_overlap` · publish builds: kept

```ogt
fn overlap {
  doc "Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast."
  aliases "overlap sphere, area query, shapes in range, Unity Physics.OverlapSphere, Godot intersect_shape"
  param world World "The world."
  param desc OverlapDesc "The shape, its placement and the filter."
  returns Result.List.Entity
}
```

<sub>Module `physics` line 1851</sub>
