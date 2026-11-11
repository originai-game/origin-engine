<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderTargetDesc"></a>

### `RenderTargetDesc` (desc struct)

A render target's size and storage, as three.js WebGLRenderTarget's width, height, type and depthTexture.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `512` | ≥ 1 | — | — | Its width in texels. |
| `height` | `number` | `512` | ≥ 1 | — | — | Its height in texels. |
| `format` | `RenderTargetFormat` | `"ldr"` | — | — | — | How its colour is stored. |
| `depth` | `boolean` | `false` | — | — | — | Keep the view's depth too, as a second texture (RenderTargetInfo.depthTexture): its linear view depth from near (0) to far (1) in red, green and blue, of the opaque meshes and models, alpha-cut ones cut (transparent meshes, sprites and particles write none), drawn on every tier (RenderCapabilities.renderTargetDepth). It costs a second draw of the view's opaque meshes. |

```ogt
struct RenderTargetDesc {
  doc "A render target's size and storage, as three.js WebGLRenderTarget's width, height, type and depthTexture."
  desc true
  field width u32 "Its width in texels." {
    default 512
    min 1
  }
  field height u32 "Its height in texels." {
    default 512
    min 1
  }
  field format RenderTargetFormat "How its colour is stored."
  field depth bool "Keep the view's depth too, as a second texture (RenderTargetInfo.depthTexture): its linear view depth from near (0) to far (1) in red, green and blue, of the opaque meshes and models, alpha-cut ones cut (transparent meshes, sprites and particles write none), drawn on every tier (RenderCapabilities.renderTargetDepth). It costs a second draw of the view's opaque meshes."
}
```

<sub>Module `render` line 3723</sub>
