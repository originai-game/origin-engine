<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-shaderMaterial"></a>

### `og.render.shaderMaterial`

Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest.

```ts
og.render.shaderMaterial(world: World, shader: Shader, desc?: StandardMaterialDesc): Result<Material>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `shader` | `Shader` | no | The shader. |
| `desc` | `StandardMaterialDesc` | yes | The base material; omitted parameters take three.js defaults. A dissolve sets alphaTest so that alpha below it is cut. |

**Returns** `Result<Material>`: Refused for a shader not loaded in this world.

Binding symbol `og_render_shader_material` · publish builds: kept

```ogt
fn shaderMaterial {
  doc "Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest."
  param world World "The world whose renderer owns the material."
  param shader Shader "The shader."
  param desc StandardMaterialDesc "The base material; omitted parameters take three.js defaults. A dissolve sets alphaTest so that alpha below it is cut."
  returns Result.Material "Refused for a shader not loaded in this world."
}
```

<sub>Module `render` line 2958</sub>
