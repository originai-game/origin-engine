<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-setMesh"></a>

### `og.physics.setMesh`

Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state: world.snapshot carries a mesh, convex-hull or height-field collider by its handle and scale, not its triangles, and world.restore rebuilds the collider from the geometry registered then (so load or register it before restoring; a save naming one that is not is refused).

```ts
og.physics.setMesh(world: World, mesh: Geometry, desc?: MeshDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Geometry` | no | The handle colliders name in Collider.mesh. |
| `desc` | `MeshDesc` | yes | Positions and triangles. |

**Returns** `Result<void>`: Refused for an empty, non-finite or out-of-range mesh.

Binding symbol `og_physics_set_mesh` · publish builds: kept

```ogt
fn setMesh {
  doc "Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state: world.snapshot carries a mesh, convex-hull or height-field collider by its handle and scale, not its triangles, and world.restore rebuilds the collider from the geometry registered then (so load or register it before restoring; a save naming one that is not is refused)."
  param world World "The world."
  param mesh Geometry "The handle colliders name in Collider.mesh."
  param desc MeshDesc "Positions and triangles."
  returns Result.Void "Refused for an empty, non-finite or out-of-range mesh."
}
```

<sub>Module `physics` line 2082</sub>
