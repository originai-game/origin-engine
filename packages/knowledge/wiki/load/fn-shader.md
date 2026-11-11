<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-shader"></a>

### `og.load.shader`

A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end): `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines its own vertex and fragment entries over the binding contract, which the prepass and every shadow pass draw through too. Loaded once per path.

See `loading §6`.

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
  doc "A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end): `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines its own vertex and fragment entries over the binding contract, which the prepass and every shadow pass draw through too. Loaded once per path."
  guide "loading §6"
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `shaders/dissolve.ogt`."
  returns Result.Shader "Refused for a path not in the bundle or not a shader."
}
```

<sub>Module `load` line 290</sub>
