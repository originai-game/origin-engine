<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ShapeGeometryDesc"></a>

### `ShapeGeometryDesc` (desc struct)

A flat shape with holes in the xy plane facing +Z, as three.js ShapeGeometry(shapes, curveSegments): triangulated by earcut (three.js's own port), the outline made clockwise and the holes counter-clockwise first, so its vertices and triangles are three.js's for the same shape. uvs are world x and y (v flipped to run down the image).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shapes` | `Path2 \| readonly Path2[] \| readonly Vector2[] \| null` | `null` | — | — | — | One og.Shape (an og.Path2 is a shape without holes), a list of them, or the outline's points. Without it, three.js's default triangle (0, 0.5), (-0.5, -0.5), (0.5, -0.5). |
| `curveSegments` | `number` | `12` | ≥ 1 | — | — | Points per curve when an og.Shape's curves are flattened (three.js extractPoints(curveSegments); an ellipse takes twice as many, a spline as many per point). |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too (BufferGeometryDesc.physics). |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later (BufferGeometryDesc.dynamic). |

```ogt
struct ShapeGeometryDesc {
  doc "A flat shape with holes in the xy plane facing +Z, as three.js ShapeGeometry(shapes, curveSegments): triangulated by earcut (three.js's own port), the outline made clockwise and the holes counter-clockwise first, so its vertices and triangles are three.js's for the same shape. uvs are world x and y (v flipped to run down the image)."
  desc true
  field shapes Option.OneOf.Path2.OneOf.List.Path2.List.Vector2 "One og.Shape (an og.Path2 is a shape without holes), a list of them, or the outline's points. Without it, three.js's default triangle (0, 0.5), (-0.5, -0.5), (0.5, -0.5)."
  field curveSegments u32 "Points per curve when an og.Shape's curves are flattened (three.js extractPoints(curveSegments); an ellipse takes twice as many, a spline as many per point)." {
    default 12
    min 1
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too (BufferGeometryDesc.physics)." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later (BufferGeometryDesc.dynamic)."
}
```

<sub>`api.ogt` line 6950</sub>
