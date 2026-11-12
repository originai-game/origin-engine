<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-uiMaterial"></a>

### `og.load.uiMaterial`

A custom UI material for og.ui's LayerDesc, from a cooked `.ogt` of `type ui-material`: `code` or `source` (a `.wgsl`) defining `fn material(m: MaterialInput, p: Params) -> vec4<f32>`, which returns the composite's straight-alpha linear colour at m.uv (0 to 1 over the layer; m.pixel, m.size in CSS px, m.time the UI clock), reading the layer with layer(uv) and layer_texel(), and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most four). The cook compiles it for every back end. Loaded once per path.

```ts
og.load.uiMaterial(world: World, path: string): Result<UiMaterial>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `ui/duotone.ogt`. |

**Returns** `Result<UiMaterial>`: Refused for a path not in the bundle or not a UI material.

Binding symbol `og_load_ui_material` · publish builds: kept

```ogt
fn uiMaterial {
  doc "A custom UI material for og.ui's LayerDesc, from a cooked `.ogt` of `type ui-material`: `code` or `source` (a `.wgsl`) defining `fn material(m: MaterialInput, p: Params) -> vec4<f32>`, which returns the composite's straight-alpha linear colour at m.uv (0 to 1 over the layer; m.pixel, m.size in CSS px, m.time the UI clock), reading the layer with layer(uv) and layer_texel(), and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most four). The cook compiles it for every back end. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `ui/duotone.ogt`."
  returns Result.UiMaterial "Refused for a path not in the bundle or not a UI material."
}
```

<sub>Module `load` line 327</sub>
