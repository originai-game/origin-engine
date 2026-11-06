<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-post"></a>

### `og.load.post`

A custom post pass for PostEffects.custom, from a cooked `.ogt` of `type post`: `code` or `source` (a `.wgsl` beside it) defining `fn post(f: PostInput, p: Params) -> vec4<f32>` and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most eight). The function reads previous(uv), scene_color(uv), depth(uv), normal(uv) and f.uv, f.pixel, f.size, f.time, and returns linear HDR colour. The cook compiles it for every back end. Loaded once per path.

```ts
og.load.post(world: World, path: string): Result<Post>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `post/crt.ogt`. |

**Returns** `Result<Post>`: Refused for a path not in the bundle or not a post pass.

Binding symbol `og_load_post` · publish builds: kept

```ogt
fn post {
  doc "A custom post pass for PostEffects.custom, from a cooked `.ogt` of `type post`: `code` or `source` (a `.wgsl` beside it) defining `fn post(f: PostInput, p: Params) -> vec4<f32>` and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most eight). The function reads previous(uv), scene_color(uv), depth(uv), normal(uv) and f.uv, f.pixel, f.size, f.time, and returns linear HDR colour. The cook compiles it for every back end. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `post/crt.ogt`."
  returns Result.Post "Refused for a path not in the bundle or not a post pass."
}
```

<sub>`api.ogt` line 8191</sub>
