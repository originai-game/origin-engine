<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="type-ArtNormals"></a>

### `ArtNormals` (enum)

How the style's models are shaded across their faces (ArtShape.normals).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: each model's own normals. |
| `"smooth"` | Smooth: vertex normals averaged over the faces that share them, soft forms. |
| `"flat"` | Flat: one normal per face, every facet visible (faceted low poly, paper craft, voxels). |
| `"angle"` | Split where neighbouring faces meet at more than ArtShape.normalAngle: hard edges stay hard, curves stay smooth. |

```ogt
enum ArtNormals {
  doc "How the style's models are shaded across their faces (ArtShape.normals)."
  variant any "No constraint: each model's own normals."
  variant smooth "Smooth: vertex normals averaged over the faces that share them, soft forms."
  variant flat "Flat: one normal per face, every facet visible (faceted low poly, paper craft, voxels)."
  variant angle "Split where neighbouring faces meet at more than ArtShape.normalAngle: hard edges stay hard, curves stay smooth."
}
```

<sub>`api.ogt` line 26551</sub>
