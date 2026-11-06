<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-standardMaterial"></a>

### `og.render.standardMaterial`

Create a standard material.

Also known as: PBR material, MeshStandardMaterial, Unity Standard shader, Godot StandardMaterial3D

```ts
og.render.standardMaterial(world: World, desc?: StandardMaterialDesc): Result<Material>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `desc` | `StandardMaterialDesc` | yes | The parameters; omitted ones take three.js defaults. |

**Returns** `Result<Material>`.

Binding symbol `og_render_standard_material` · publish builds: kept

```ogt
fn standardMaterial {
  doc "Create a standard material."
  aliases "PBR material, MeshStandardMaterial, Unity Standard shader, Godot StandardMaterial3D"
  param world World "The world whose renderer owns the material."
  param desc StandardMaterialDesc "The parameters; omitted ones take three.js defaults."
  returns Result.Material
}
```

<sub>`api.ogt` line 6446</sub>
