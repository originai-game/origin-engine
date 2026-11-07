<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SpecularModel"></a>

### `SpecularModel` (enum)

Which specular the lit surfaces draw (RenderSettings.specular), as og-render's Specular.

A string: one of the variants below. The zero value is `"engine"`.

| Variant | Doc |
|---|---|
| `"engine"` | The engine's: GGX with the uncorrelated Schlick-Smith visibility (k = (r + 1)² / 8) and the image-based light's split sum from the integrated BRDF LUT. The default. |
| `"three"` | three.js's MeshStandardMaterial and MeshPhysicalMaterial, for a game tuned under three.js: the height-correlated Smith visibility (V_GGX_SmithCorrelated) on the direct light and the clear coat, DFGApprox's analytic split sum in place of the LUT (the clear coat's too), and geometryRoughness, a roughness of at least 0.0525 plus how fast the mesh's normal turns across the pixel, on meshes, scattered copies and custom lit materials (terrain, grass, crowd cards and impostors keep their roughness: their normals are a heightmap's, a dome's or a bake's). At grazing view (a thin card, a strand seen edge-on) about twice the engine's direct highlight, and less of the sky on rough surfaces seen edge-on. |

```ogt
enum SpecularModel {
  doc "Which specular the lit surfaces draw (RenderSettings.specular), as og-render's Specular."
  variant engine "The engine's: GGX with the uncorrelated Schlick-Smith visibility (k = (r + 1)² / 8) and the image-based light's split sum from the integrated BRDF LUT. The default."
  variant three "three.js's MeshStandardMaterial and MeshPhysicalMaterial, for a game tuned under three.js: the height-correlated Smith visibility (V_GGX_SmithCorrelated) on the direct light and the clear coat, DFGApprox's analytic split sum in place of the LUT (the clear coat's too), and geometryRoughness, a roughness of at least 0.0525 plus how fast the mesh's normal turns across the pixel, on meshes, scattered copies and custom lit materials (terrain, grass, crowd cards and impostors keep their roughness: their normals are a heightmap's, a dome's or a bake's). At grazing view (a thin card, a strand seen edge-on) about twice the engine's direct highlight, and less of the sky on rough surfaces seen edge-on."
}
```

<sub>Module `render` line 537</sub>
