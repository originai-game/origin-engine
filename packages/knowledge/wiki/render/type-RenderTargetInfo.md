<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderTargetInfo"></a>

### `RenderTargetInfo` (struct)

What og.render.renderTarget made.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `RenderTarget` | `0` | — | — | — | The target, for CameraView.target and Mirror.target. |
| `texture` | `Texture` | `0` | — | — | — | Its colour: any material map, og.Sprite, og.render.sprite or og.ui image shows it, as drawn by the last camera that drew into it (transparent black before). |
| `depthTexture` | `Texture` | `0` | — | — | — | Its depth (RenderTargetDesc.depth); 0 without. |
| `width` | `number` | `0` | — | — | — | Its width in texels. |
| `height` | `number` | `0` | — | — | — | Its height in texels. |

```ogt
struct RenderTargetInfo {
  doc "What og.render.renderTarget made."
  field target RenderTarget "The target, for CameraView.target and Mirror.target."
  field texture Texture "Its colour: any material map, og.Sprite, og.render.sprite or og.ui image shows it, as drawn by the last camera that drew into it (transparent black before)."
  field depthTexture Texture "Its depth (RenderTargetDesc.depth); 0 without."
  field width u32 "Its width in texels."
  field height u32 "Its height in texels."
}
```

<sub>Module `render` line 3976</sub>
