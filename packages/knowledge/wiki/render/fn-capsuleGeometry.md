<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-capsuleGeometry"></a>

### `og.render.capsuleGeometry`

Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height).

```ts
og.render.capsuleGeometry(world: World, desc?: CapsuleGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CapsuleGeometryDesc` | yes | Its radius, length and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_capsule_geometry` · publish builds: kept

```ogt
fn capsuleGeometry {
  doc "Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height)."
  param world World "The world whose renderer owns the geometry."
  param desc CapsuleGeometryDesc "Its radius, length and tessellation."
  returns Result.Geometry
}
```

<sub>Module `render` line 3383</sub>
