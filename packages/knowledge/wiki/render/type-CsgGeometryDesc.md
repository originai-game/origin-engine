<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-CsgGeometryDesc"></a>

### `CsgGeometryDesc` (desc struct)

Two closed geometries combined by a boolean operation, each placed first: a box with a hole bored through it, a dice, a window cut in a wall. The result is closed when both are (every edge shared by two triangles, wound opposite), every piece keeps its source's normals, uvs and colours (b's turned inside out where it is subtracted). Where a plane side or an edge crossing is decided, the decision is exact (Shewchuk's predicates), so every host makes the same cuts; faces that coincide keep one copy or none, as the operation says.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `op` | `CsgOp` | `"union"` | — | — | — | The operation. |
| `a` | `GeometryPart` | `{}` | — | — | — | The first operand. |
| `b` | `GeometryPart` | `{}` | — | — | — | The second operand. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct CsgGeometryDesc {
  doc "Two closed geometries combined by a boolean operation, each placed first: a box with a hole bored through it, a dice, a window cut in a wall. The result is closed when both are (every edge shared by two triangles, wound opposite), every piece keeps its source's normals, uvs and colours (b's turned inside out where it is subtracted). Where a plane side or an edge crossing is decided, the decision is exact (Shewchuk's predicates), so every host makes the same cuts; faces that coincide keep one copy or none, as the operation says."
  desc true
  field op CsgOp "The operation."
  field a GeometryPart "The first operand."
  field b GeometryPart "The second operand."
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later."
}
```

<sub>Module `render` line 3367</sub>
