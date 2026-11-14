<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-renderTarget"></a>

### `og.render.renderTarget`

Make a render target: a texture an og.CameraView or og.Mirror draws into, its colour (RenderTargetInfo.texture) usable by any map, sprite or UI image, as three.js WebGLRenderTarget and its texture. Targets are drawn before the frame, so a material shows this frame's image; one a camera draws while it sees itself (a monitor in the room it films) shows the frame before.

```ts
og.render.renderTarget(world: World, desc?: RenderTargetDesc): Result<RenderTargetInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `desc` | `RenderTargetDesc` | yes | Its size, format and depth. |

**Returns** `Result<RenderTargetInfo>`: Refused for a side past RenderCapabilities.maxRenderTargetSize.

Binding symbol `og_render_render_target` · publish builds: kept

```ogt
fn renderTarget {
  doc "Make a render target: a texture an og.CameraView or og.Mirror draws into, its colour (RenderTargetInfo.texture) usable by any map, sprite or UI image, as three.js WebGLRenderTarget and its texture. Targets are drawn before the frame, so a material shows this frame's image; one a camera draws while it sees itself (a monitor in the room it films) shows the frame before."
  param world World "The world whose renderer owns it."
  param desc RenderTargetDesc "Its size, format and depth."
  returns Result.RenderTargetInfo "Refused for a side past RenderCapabilities.maxRenderTargetSize."
}
```

<sub>Module `render` line 4015</sub>
