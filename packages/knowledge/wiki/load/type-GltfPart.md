<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-GltfPart"></a>

### `GltfPart` (struct)

One placed primitive in the model's default scene. Repeated nodes and instances produce separate parts.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | Mesh-local geometry; transform places it in model space. |
| `material` | `Material` | `0` | — | — | — | This part's material. |
| `transform` | `Matrix4` | `{ elements: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] }` | — | — | — | Primitive-to-model placement, including node ancestors and instance transforms, or the skinned node's bind placement. |

```ogt
struct GltfPart {
  doc "One placed primitive in the model's default scene. Repeated nodes and instances produce separate parts."
  field geometry Geometry "Mesh-local geometry; transform places it in model space."
  field material Material "This part's material."
  field transform Matrix4 "Primitive-to-model placement, including node ancestors and instance transforms, or the skinned node's bind placement."
}
```

<sub>Module `load` line 68</sub>
