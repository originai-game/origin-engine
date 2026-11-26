<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-compile"></a>

### `og.render.compile`

Prepare this world's render and compute pipelines, as three.js renderer.compileAsync, a few per frame starting next frame. Covers what the world draws now, regardless of camera direction, with every lit feature; content spawned later compiles on first draw. Asynchronous where supported; otherwise runs on the frame's thread. After the next frame it also builds the UI's documents on a copy of the UI and physics' height-field chunk shapes, once, so the first played frame does not. Presentation only: nothing a tick reads changes.

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
  doc "Prepare this world's render and compute pipelines, as three.js renderer.compileAsync, a few per frame starting next frame. Covers what the world draws now, regardless of camera direction, with every lit feature; content spawned later compiles on first draw. Asynchronous where supported; otherwise runs on the frame's thread. After the next frame it also builds the UI's documents on a copy of the UI and physics' height-field chunk shapes, once, so the first played frame does not. Presentation only: nothing a tick reads changes."
  guide "loading §12"
  aliases "compileAsync, warm up shaders, prewarm pipelines, shader precompile, PSO precaching, Unity ShaderVariantCollection.WarmUp, Unreal PSO precache"
  param world World "The world whose renderer compiles."
  returns Void
}
```

<sub>Module `render` line 967</sub>
