<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-setMaterialShader"></a>

### `og.render.setMaterialShader`

Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes.

```ts
og.render.setMaterialShader(world: World, material: Material, shader: Shader | null): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `material` | `Material` | no | The material, from og.render.standardMaterial, og.render.shaderMaterial or a loaded glTF. |
| `shader` | `Shader \| null` | no | The shader, or null to draw the material plain again. |

**Returns** `Result<void>`: Refused for a dead material or a shader not loaded in this world.

Binding symbol `og_render_set_material_shader` · publish builds: kept

```ogt
fn setMaterialShader {
  doc "Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes."
  param world World "The world whose renderer owns the material."
  param material Material "The material, from og.render.standardMaterial, og.render.shaderMaterial or a loaded glTF."
  param shader Option.Shader "The shader, or null to draw the material plain again."
  returns Result.Void "Refused for a dead material or a shader not loaded in this world."
}
```

<sub>Module `render` line 2932</sub>
