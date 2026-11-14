<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-setShaderParam"></a>

### `og.render.setShaderParam`

Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color.

```ts
og.render.setShaderParam(world: World, material: Material, name: string, value: readonly number[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `material` | `Material` | no | A material made by og.render.shaderMaterial or given a shader by og.render.setMaterialShader. |
| `name` | `string` | no | The parameter's name. |
| `value` | `readonly number[]` | no | Its value; missing components are 0. |

**Returns** `Result<void>`: Refused for a material without a shader or a parameter its shader does not declare.

Binding symbol `og_render_set_shader_param` · publish builds: kept

```ogt
fn setShaderParam {
  doc "Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color."
  param world World "The world whose renderer owns the material."
  param material Material "A material made by og.render.shaderMaterial or given a shader by og.render.setMaterialShader."
  param name String "The parameter's name."
  param value List.f32 "Its value; missing components are 0."
  returns Result.Void "Refused for a material without a shader or a parameter its shader does not declare."
}
```

<sub>Module `render` line 2970</sub>
