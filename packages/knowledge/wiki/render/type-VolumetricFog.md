<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-VolumetricFog"></a>

### `VolumetricFog` (enum)

How the device draws Fog.volumetric (RenderCapabilities.volumetricFog).

See `render §7`.

A string: one of the variants below. The zero value is `"froxel"`.

| Variant | Doc |
|---|---|
| `"froxel"` | A grid of view-aligned cells over the camera's frustum (160 by 90 by 64, slices deepening quadratically) lit in compute by the sun through its cascades and by the clustered lights through their shadows and cookies (eight samples a cell), integrated front to back: what the engine draws with. |
| `"raymarch"` | Each pixel's view ray marched through the fog in 24 jittered steps, lit by the sun through its cascades and by the first 8 point, spot and rect lights of its screen cluster (RenderCapabilities.volumetricLights), with no compute pass: og-render's alternative to the froxel grid. |

```ogt
enum VolumetricFog {
  doc "How the device draws Fog.volumetric (RenderCapabilities.volumetricFog)."
  guide "render §7"
  variant froxel "A grid of view-aligned cells over the camera's frustum (160 by 90 by 64, slices deepening quadratically) lit in compute by the sun through its cascades and by the clustered lights through their shadows and cookies (eight samples a cell), integrated front to back: what the engine draws with."
  variant raymarch "Each pixel's view ray marched through the fog in 24 jittered steps, lit by the sun through its cascades and by the first 8 point, spot and rect lights of its screen cluster (RenderCapabilities.volumetricLights), with no compute pass: og-render's alternative to the froxel grid."
}
```

<sub>Module `render` line 4122</sub>
