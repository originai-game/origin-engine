<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-VolumetricFog"></a>

### `VolumetricFog` (enum)

How the device draws Fog.volumetric (RenderCapabilities.volumetricFog).

A string: one of the variants below. The zero value is `"froxel"`.

| Variant | Doc |
|---|---|
| `"froxel"` | A grid of view-aligned cells over the camera's frustum (160 by 90 by 64, slices deepening with the square of their index) lit in compute by the sun through its shadow cascades and by the clustered point, spot and rect lights through their shadows and cookies (each cell the mean of eight samples over its depth), then integrated front to back: what the engine draws with. |
| `"raymarch"` | Each pixel's view ray marched through the fog in 24 jittered steps, each lit by the sun through its cascades and by the first 8 point, spot and rect lights of its screen cluster through their shadows and cookies (RenderCapabilities.volumetricLights), with no compute pass: og-render's alternative to the froxel grid, which the engine draws with. |

```ogt
enum VolumetricFog {
  doc "How the device draws Fog.volumetric (RenderCapabilities.volumetricFog)."
  variant froxel "A grid of view-aligned cells over the camera's frustum (160 by 90 by 64, slices deepening with the square of their index) lit in compute by the sun through its shadow cascades and by the clustered point, spot and rect lights through their shadows and cookies (each cell the mean of eight samples over its depth), then integrated front to back: what the engine draws with."
  variant raymarch "Each pixel's view ray marched through the fog in 24 jittered steps, each lit by the sun through its cascades and by the first 8 point, spot and rect lights of its screen cluster through their shadows and cookies (RenderCapabilities.volumetricLights), with no compute pass: og-render's alternative to the froxel grid, which the engine draws with."
}
```

<sub>Module `render` line 3860</sub>
