<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-compile"></a>

### `og.render.compile`

Compile every pipeline the renderer will draw this world with, as three.js renderer.compileAsync: a few each frame from the next one on, in the background, so a level never waits on a compile once its loading screen is gone. Call it behind a loading screen (after og.scene.change, or once the level's models are spawned) and keep the screen up until og.loading.idle is true; og.loading.status counts the pipelines (pipelinesRequested, pipelinesReady) for its bar. It covers the pipelines of everything the world draws now, wherever the camera points, with every lit feature (so rain or a decal that starts later draws at once) and the compute the frames run; a model, material or effect spawned afterwards compiles when it first draws. The play gate does the same behind the gate for the first scene. Presentation only, as og.render.snap: nothing a tick reads changes. Natively and without asynchronous pipelines it makes them on the frame's thread, a few a frame.

See `loading §12`.

Also known as: compileAsync, warm up shaders, prewarm pipelines, shader precompile, PSO precaching, Unity ShaderVariantCollection.WarmUp, Unreal PSO precache

```ts
og.render.compile(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer compiles. |

**Returns** `void`.

Binding symbol `og_render_compile` · publish builds: kept

```ogt
fn compile {
  doc "Compile every pipeline the renderer will draw this world with, as three.js renderer.compileAsync: a few each frame from the next one on, in the background, so a level never waits on a compile once its loading screen is gone. Call it behind a loading screen (after og.scene.change, or once the level's models are spawned) and keep the screen up until og.loading.idle is true; og.loading.status counts the pipelines (pipelinesRequested, pipelinesReady) for its bar. It covers the pipelines of everything the world draws now, wherever the camera points, with every lit feature (so rain or a decal that starts later draws at once) and the compute the frames run; a model, material or effect spawned afterwards compiles when it first draws. The play gate does the same behind the gate for the first scene. Presentation only, as og.render.snap: nothing a tick reads changes. Natively and without asynchronous pipelines it makes them on the frame's thread, a few a frame."
  guide "loading §12"
  aliases "compileAsync, warm up shaders, prewarm pipelines, shader precompile, PSO precaching, Unity ShaderVariantCollection.WarmUp, Unreal PSO precache"
  param world World "The world whose renderer compiles."
  returns Void
}
```

<sub>Module `render` line 904</sub>
