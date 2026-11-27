<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-uiMaterial"></a>

### `og.load.uiMaterial`

A custom UI material resource from a cooked TS file['ui-material'] declaration: code (inline WGSL or input.text of a module-relative source) defining fn material(m: MaterialInput, p: Params) -> vec4<f32>, and up to four params with name, kind (float|vec2|vec3|vec4|color), and optional default. The cook compiles it for every back end. Loaded once per path.

```ts
og.load.uiMaterial(world: World, path: string): Result<UiMaterial>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The canonical asset id, such as `base/materials/materials/rarity`. |

**Returns** `Result<UiMaterial>`: Refused for a path not in the bundle or not a UI material.

Binding symbol `og_load_ui_material` · publish builds: kept

```ogt
fn uiMaterial {
  doc "A custom UI material resource from a cooked TS file['ui-material'] declaration: code (inline WGSL or input.text of a module-relative source) defining fn material(m: MaterialInput, p: Params) -> vec4<f32>, and up to four params with name, kind (float|vec2|vec3|vec4|color), and optional default. The cook compiles it for every back end. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The canonical asset id, such as `base/materials/materials/rarity`."
  returns Result.UiMaterial "Refused for a path not in the bundle or not a UI material."
}
```

<sub>Module `load` line 393</sub>
