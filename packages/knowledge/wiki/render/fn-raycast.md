<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-raycast"></a>

### `og.render.raycast`

Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). It tests og.Mesh and og.Model entities at their world transforms through a BVH per geometry (built when the geometry is made, loaded or updated), both faces of each triangle, skinned models in their bind pose. Read the world as of the call; inside a system, its transforms as earlier systems left them.

```ts
og.render.raycast(world: World, ray: Ray, desc?: RaycastGeometryDesc): Result<readonly GeometryHit[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |
| `ray` | `Ray` | no | The ray; its direction need not be normalised and must not be zero. |
| `desc` | `RaycastGeometryDesc` | yes | Reach, nearest only and layers. |

**Returns** `Result<readonly GeometryHit[]>`: Refused for a zero or non-finite direction.

Binding symbol `og_render_raycast` · publish builds: kept

```ogt
fn raycast {
  doc "Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). It tests og.Mesh and og.Model entities at their world transforms through a BVH per geometry (built when the geometry is made, loaded or updated), both faces of each triangle, skinned models in their bind pose. Read the world as of the call; inside a system, its transforms as earlier systems left them."
  param world World "The world asked."
  param ray Ray "The ray; its direction need not be normalised and must not be zero."
  param desc RaycastGeometryDesc "Reach, nearest only and layers."
  returns Result.List.GeometryHit "Refused for a zero or non-finite direction."
}
```

<sub>Module `render` line 3648</sub>
