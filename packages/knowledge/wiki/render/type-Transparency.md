<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Transparency"></a>

### `Transparency` (enum)

How the main camera's blended surfaces (transparent materials, alpha billboards, alpha particles and trails) are composited, as og-render's Transparency: in any order, never sorted. Additive effects stay additive and drawn after them; 2D sprites, text and the UI keep their layer order; secondary cameras, reflections and probe faces still sort back to front. RenderCapabilities.transparency reports what the device resolved.

A string: one of the variants below. The zero value is `"weighted"`.

| Variant | Doc |
|---|---|
| `"weighted"` | Weighted blended order-independent transparency (McGuire and Bavoil 2013) with each pixel's nearest layer kept exact (hybrid transparency, Maule et al. 2013): a billboard or pane in front keeps its own colour over what lies behind it, and only the layers behind the nearest read as their weighted mean rather than strictly front over back. The nearest layer is the nearest fragment at least 1/8 opaque, marked by a depth pass of its own (where the device cannot write storage from fragment shaders, every layer is weighted). Two submission orders agree within the half-float rounding of the sums (at most one 8-bit step per channel for a few layers). |
| `"linkedList"` | Per-pixel fragment lists: each pixel keeps up to RenderOptions.transparencyLayers fragments, sorted by depth (equal depths by a stable key of the draw and its primitive) and composited exactly, the same in any submission order. A pixel with more fragments is resolved whole as weighted, and counted (the render dump's transparency.fallbackPixels). Weighted where the device cannot write storage from fragment shaders, or where the scene is too large for one node a pixel in its largest storage buffer. |

```ogt
enum Transparency {
  doc "How the main camera's blended surfaces (transparent materials, alpha billboards, alpha particles and trails) are composited, as og-render's Transparency: in any order, never sorted. Additive effects stay additive and drawn after them; 2D sprites, text and the UI keep their layer order; secondary cameras, reflections and probe faces still sort back to front. RenderCapabilities.transparency reports what the device resolved."
  variant weighted "Weighted blended order-independent transparency (McGuire and Bavoil 2013) with each pixel's nearest layer kept exact (hybrid transparency, Maule et al. 2013): a billboard or pane in front keeps its own colour over what lies behind it, and only the layers behind the nearest read as their weighted mean rather than strictly front over back. The nearest layer is the nearest fragment at least 1/8 opaque, marked by a depth pass of its own (where the device cannot write storage from fragment shaders, every layer is weighted). Two submission orders agree within the half-float rounding of the sums (at most one 8-bit step per channel for a few layers)."
  variant linkedList "Per-pixel fragment lists: each pixel keeps up to RenderOptions.transparencyLayers fragments, sorted by depth (equal depths by a stable key of the draw and its primitive) and composited exactly, the same in any submission order. A pixel with more fragments is resolved whole as weighted, and counted (the render dump's transparency.fallbackPixels). Weighted where the device cannot write storage from fragment shaders, or where the scene is too large for one node a pixel in its largest storage buffer."
}
```

<sub>`api.ogt` line 3872</sub>
