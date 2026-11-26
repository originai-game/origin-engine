<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantMeshPart"></a>

### `PlantMeshPart` (struct)

One mesh part of a procedural plant (wood or foliage) with vertex and foliage-wind TypedArrays ready for og.render.bufferGeometry.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `positions` | `Float32Array` | — | — | — | — | Vertex positions, x y z per vertex. |
| `normals` | `Float32Array` | — | — | — | — | Unit normals, x y z per vertex. |
| `uvs` | `Float32Array` | — | — | — | — | UV coordinates, u v per vertex. |
| `colors` | `Float32Array` | — | — | — | — | Linear RGBA vertex colours (r g b a per vertex; empty when uniform white). |
| `indices` | `Uint32Array` | — | — | — | — | Counter-clockwise triangle indices. |
| `windPivot` | `Float32Array` | — | — | — | — | Limb sway pivot and stiffness (px py pz stiffness per vertex). |
| `windLeaf` | `Float32Array` | — | — | — | — | Leaf attachment anchor and flutter weight (ax ay az flutter per vertex). |

```ogt
struct PlantMeshPart {
  doc "One mesh part of a procedural plant (wood or foliage) with vertex and foliage-wind TypedArrays ready for og.render.bufferGeometry."
  field positions Float32Array "Vertex positions, x y z per vertex."
  field normals Float32Array "Unit normals, x y z per vertex."
  field uvs Float32Array "UV coordinates, u v per vertex."
  field colors Float32Array "Linear RGBA vertex colours (r g b a per vertex; empty when uniform white)."
  field indices Uint32Array "Counter-clockwise triangle indices."
  field windPivot Float32Array "Limb sway pivot and stiffness (px py pz stiffness per vertex)."
  field windLeaf Float32Array "Leaf attachment anchor and flutter weight (ax ay az flutter per vertex)."
}
```

<sub>Module `procgen` line 783</sub>
