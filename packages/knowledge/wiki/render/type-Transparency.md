<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Transparency"></a>

### `Transparency` (enum)

How the main camera's blended surfaces (transparent materials, alpha billboards, alpha particles and trails) are composited, as og-render's Transparency: in any order, never sorted. Additive effects stay additive and drawn after them; 2D sprites, text and the UI keep their layer order; secondary cameras, reflections and probe faces still sort back to front. RenderCapabilities.transparency reports what the device resolved.

See `render §2`.

A string: one of the variants below. The zero value is `"weighted"`.

| Variant | Doc |
|---|---|
| `"weighted"` | Weighted blended OIT (McGuire and Bavoil 2013) with each pixel's nearest fragment at least 1/8 opaque kept exact (Maule et al. 2013); the layers behind it show as their weighted mean. Any two submission orders agree within half-float rounding. Without fragment storage writes, all are weighted. |
| `"linkedList"` | Per-pixel fragment lists: up to RenderOptions.transparencyLayers a pixel, sorted by depth (ties by a stable draw and primitive key) and composited exactly in any order. A pixel with more is resolved as weighted and counted (render dump transparency.fallbackPixels). Weighted where lists cannot run. |

```ogt
enum Transparency {
  doc "How the main camera's blended surfaces (transparent materials, alpha billboards, alpha particles and trails) are composited, as og-render's Transparency: in any order, never sorted. Additive effects stay additive and drawn after them; 2D sprites, text and the UI keep their layer order; secondary cameras, reflections and probe faces still sort back to front. RenderCapabilities.transparency reports what the device resolved."
  variant weighted "Weighted blended OIT (McGuire and Bavoil 2013) with each pixel's nearest fragment at least 1/8 opaque kept exact (Maule et al. 2013); the layers behind it show as their weighted mean. Any two submission orders agree within half-float rounding. Without fragment storage writes, all are weighted."
  variant linkedList "Per-pixel fragment lists: up to RenderOptions.transparencyLayers a pixel, sorted by depth (ties by a stable draw and primitive key) and composited exactly in any order. A pixel with more is resolved as weighted and counted (render dump transparency.fallbackPixels). Weighted where lists cannot run."
  guide "render §2"
}
```

<sub>Module `render` line 101</sub>
