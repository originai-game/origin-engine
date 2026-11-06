<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-shader"></a>

### `og.load.shader`

A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput` and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over the binding contract (camera(), the model with its skin, Params, tex0 to tex7 for the material's eight texture slots), lit by the frame's lights or drawn as returned, the prepass and every shadow pass drawn through the same entries. Loaded once per path.

```ts
og.load.shader(world: World, path: string): Result<Shader>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `shaders/dissolve.ogt`. |

**Returns** `Result<Shader>`: Refused for a path not in the bundle or not a shader.

Binding symbol `og_load_shader` · publish builds: kept

```ogt
fn shader {
  doc "A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput` and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over the binding contract (camera(), the model with its skin, Params, tex0 to tex7 for the material's eight texture slots), lit by the frame's lights or drawn as returned, the prepass and every shadow pass drawn through the same entries. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `shaders/dissolve.ogt`."
  returns Result.Shader "Refused for a path not in the bundle or not a shader."
}
```

<sub>Module `load` line 272</sub>
