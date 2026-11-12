<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RaycastGeometryDesc"></a>

### `RaycastGeometryDesc` (desc struct)

Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far the ray reaches (three.js Raycaster.far); 0 is unbounded. |
| `first` | `boolean` | `false` | — | — | — | Answer only the nearest hit (cheaper: the BVH stops early). |
| `layers` | `number` | `1` | — | — | — | A bit mask of the layers tested, as three.js Raycaster.layers: an og.Mesh or og.Model is tested when its own layers (Mesh.layers, Model.layers, 1 by default) share a bit with it, so the default tests every mesh on layer 0. |
| `invisible` | `boolean` | `false` | — | — | — | Test meshes whose visible is false too. |

```ogt
struct RaycastGeometryDesc {
  doc "Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers."
  desc true
  field maxDistance f32 "How far the ray reaches (three.js Raycaster.far); 0 is unbounded." {
    min 0
    unit m
  }
  field first bool "Answer only the nearest hit (cheaper: the BVH stops early)."
  field layers u32 "A bit mask of the layers tested, as three.js Raycaster.layers: an og.Mesh or og.Model is tested when its own layers (Mesh.layers, Model.layers, 1 by default) share a bit with it, so the default tests every mesh on layer 0." {
    default 1
  }
  field invisible bool "Test meshes whose visible is false too."
}
```

<sub>Module `render` line 3863</sub>
