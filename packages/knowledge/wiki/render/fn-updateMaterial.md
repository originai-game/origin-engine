<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-updateMaterial"></a>

### `og.render.updateMaterial`

Change some parameters of a standard material.

```ts
og.render.updateMaterial(world: World, material: Material, desc?: StandardMaterialDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `material` | `Material` | no | The material. |
| `desc` | `StandardMaterialDesc` | yes | The parameters to change; omitted ones keep their values. |

**Returns** `Result<void>`: Refused for a dead material.

Binding symbol `og_render_update_material` · publish builds: kept

```ogt
fn updateMaterial {
  doc "Change some parameters of a standard material."
  param world World "The world whose renderer owns the material."
  param material Material "The material."
  param desc StandardMaterialDesc "The parameters to change; omitted ones keep their values."
  returns Result.Void "Refused for a dead material."
}
```

<sub>`api.ogt` line 6453</sub>
