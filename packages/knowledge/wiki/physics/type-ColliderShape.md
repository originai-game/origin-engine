<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-ColliderShape"></a>

### `ColliderShape` (enum)

What a collider is shaped as: Collider.shape and ShapeDesc.shape.

See `physics §1`.

A string: one of the variants below. The zero value is `"box"`.

| Variant | Doc |
|---|---|
| `"box"` | A box of `size` (both profiles). |
| `"sphere"` | A sphere of `radius` (a circle in 2D). |
| `"capsule"` | A capsule along y of `radius`, with `height` between the cap centres (both profiles). |
| `"cylinder"` | A cylinder along y of `radius` and `height` (3D only). |
| `"convex"` | The convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D). |
| `"trimesh"` | The triangles of `mesh` (for static and kinematic bodies); in 2D the outline of their xy (the edges no two triangles share) as a hollow polyline, which stops what comes from outside. |
| `"heightfield"` | The height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry). |
| `"diamond"` | A rhombus whose diagonals are size.x (along x) and size.y (along y), an isometric tile's diamond; in 3D a prism size.z deep (both profiles). |
| `"hexagon"` | A pointy-top hexagon in the box size.x by size.y, corners at the top and bottom middles and a quarter down each side, a hex tile's (turn it a quarter about z for a flat-top one); in 3D a prism size.z deep (both profiles). |

```ogt
enum ColliderShape {
  doc "What a collider is shaped as: Collider.shape and ShapeDesc.shape."
  guide "physics §1"
  variant box "A box of `size` (both profiles)."
  variant sphere "A sphere of `radius` (a circle in 2D)."
  variant capsule "A capsule along y of `radius`, with `height` between the cap centres (both profiles)."
  variant cylinder "A cylinder along y of `radius` and `height` (3D only)."
  variant convex "The convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D)."
  variant trimesh "The triangles of `mesh` (for static and kinematic bodies); in 2D the outline of their xy (the edges no two triangles share) as a hollow polyline, which stops what comes from outside."
  variant heightfield "The height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry)."
  variant diamond "A rhombus whose diagonals are size.x (along x) and size.y (along y), an isometric tile's diamond; in 3D a prism size.z deep (both profiles)."
  variant hexagon "A pointy-top hexagon in the box size.x by size.y, corners at the top and bottom middles and a quarter down each side, a hex tile's (turn it a quarter about z for a flat-top one); in 3D a prism size.z deep (both profiles)."
}
```

<sub>Module `physics` line 87</sub>
