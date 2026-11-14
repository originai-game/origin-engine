<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderTargetFormat"></a>

### `RenderTargetFormat` (enum)

How a render target's colour is stored.

A string: one of the variants below. The zero value is `"ldr"`.

| Variant | Doc |
|---|---|
| `"ldr"` | 8-bit sRGB: the view's finished image (exposed with its camera's exposure and tone mapped with RenderSettings' curve), as a loaded colour texture: a monitor, a minimap, a portrait (show it with an unlit material's map or a sprite). |
| `"hdr"` | 16-bit float, linear: the view's radiance before exposure and tone mapping, exposed once with the frame that shows it: a mirror or a portal (show it as a material's emissiveMap with color black). |

```ogt
enum RenderTargetFormat {
  doc "How a render target's colour is stored."
  variant ldr "8-bit sRGB: the view's finished image (exposed with its camera's exposure and tone mapped with RenderSettings' curve), as a loaded colour texture: a monitor, a minimap, a portrait (show it with an unlit material's map or a sprite)."
  variant hdr "16-bit float, linear: the view's radiance before exposure and tone mapping, exposed once with the frame that shows it: a mirror or a portal (show it as a material's emissiveMap with color black)."
}
```

<sub>Module `render` line 3949</sub>
