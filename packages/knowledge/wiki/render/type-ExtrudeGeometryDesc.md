<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ExtrudeGeometryDesc"></a>

### `ExtrudeGeometryDesc` (desc struct)

Shapes pushed along +Z, or along a path, as three.js ExtrudeGeometry(shapes, options) with its option names and defaults: a closed solid, its lids from earcut and its walls ring by ring, rounded by a bevel of bevelSegments rings at each face. Unindexed as three.js's (every triangle its own vertices, flat normals), three.js's vertex order and counts; uvs from three.js's WorldUVGenerator (v flipped to run down the image).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shapes` | `Path2 \| readonly Path2[] \| readonly Vector2[] \| readonly ShapePoints[] \| null` | `null` | — | — | — | One og.Shape, a list of them, the outline's points, or a list of ShapePoints (og.Shape.extractPoints). Without it, three.js's default square of side 1. |
| `depth` | `number` | `1` | ≥ 0 | `m` | — | How far the shape is pushed along +Z (without extrudePath). |
| `steps` | `number` | `1` | ≥ 1 | — | — | Rings along the depth or the path. |
| `bevelEnabled` | `boolean` | `true` | — | — | — | Round the edges of both faces. |
| `bevelThickness` | `number` | `0.2` | ≥ 0 | `m` | — | How far the bevel reaches beyond each face, along z. |
| `bevelSize` | `number \| null` | `null` | — | `m` | — | How far the bevel reaches out from the outline. Null or unset, bevelThickness - 0.1, as three.js derives it (0.1 at the default thickness, 0 at a thickness of 0.1). |
| `bevelOffset` | `number` | `0` | — | `m` | — | How far the bevel starts out from the outline. |
| `bevelSegments` | `number` | `3` | ≥ 1 | — | — | Rings in each bevel. |
| `curveSegments` | `number` | `12` | ≥ 1 | — | — | Points per curve when an og.Shape's curves are flattened. |
| `extrudePath` | `Path` | `0` | — | — | — | A path (og.path.create or og.path.find) to extrude along instead of +Z: the shape's x along the path's normal and y along its binormal (three.js's Frenet frames), steps rings evenly spaced along it. A path turns the bevel off, as three.js. 0 extrudes along +Z. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct ExtrudeGeometryDesc {
  doc "Shapes pushed along +Z, or along a path, as three.js ExtrudeGeometry(shapes, options) with its option names and defaults: a closed solid, its lids from earcut and its walls ring by ring, rounded by a bevel of bevelSegments rings at each face. Unindexed as three.js's (every triangle its own vertices, flat normals), three.js's vertex order and counts; uvs from three.js's WorldUVGenerator (v flipped to run down the image)."
  desc true
  field shapes Option.OneOf.Path2.OneOf.List.Path2.OneOf.List.Vector2.List.ShapePoints "One og.Shape, a list of them, the outline's points, or a list of ShapePoints (og.Shape.extractPoints). Without it, three.js's default square of side 1."
  field depth f32 "How far the shape is pushed along +Z (without extrudePath)." {
    default 1
    min 0
    unit m
  }
  field steps u32 "Rings along the depth or the path." {
    default 1
    min 1
  }
  field bevelEnabled bool "Round the edges of both faces." {
    default true
  }
  field bevelThickness f32 "How far the bevel reaches beyond each face, along z." {
    default 0.2
    min 0
    unit m
  }
  field bevelSize Option.f32 "How far the bevel reaches out from the outline. Null or unset, bevelThickness - 0.1, as three.js derives it (0.1 at the default thickness, 0 at a thickness of 0.1)." {
    unit m
  }
  field bevelOffset f32 "How far the bevel starts out from the outline." {
    unit m
  }
  field bevelSegments u32 "Rings in each bevel." {
    default 3
    min 1
  }
  field curveSegments u32 "Points per curve when an og.Shape's curves are flattened." {
    default 12
    min 1
  }
  field extrudePath Path "A path (og.path.create or og.path.find) to extrude along instead of +Z: the shape's x along the path's normal and y along its binormal (three.js's Frenet frames), steps rings evenly spaced along it. A path turns the bevel off, as three.js. 0 extrudes along +Z."
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later."
}
```

<sub>Module `render` line 3208</sub>
