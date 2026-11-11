<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="record-Model"></a>

### `og.Model` (component)

Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.

The glTF's materials are shared by every instance of it; `tint` and `flash` are this instance's own, carried with its transform to the lit shaders (one draw for every instance, whatever each shows), so one of a crowd can flash when struck.

Token `og.Model`, registered under the schema name `og.Model`; its value type is `Model`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The glTF asset. |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour, 0 to 1 a channel, multiplied with every material's base colour on this instance only (white: as the glTF). |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash on this instance only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `castShadow` | `boolean` | `true` | — | — | — | Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's; false takes this instance out of every shadow pass. Set from GltfDesc.castShadow unless the scene gives the entity a `Model` table. See `loading §2`. |
| `receiveShadow` | `boolean` | `true` | — | — | — | Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away. |
| `lodBias` | `number` | `1` | 0 … 16 | — | tunable | This instance's level of detail bias, multiplied with RenderSettings.lodBias: the renderer draws the coarsest cooked level whose error covers less than a pixel at the model's distance. Above 1 keeps detail farther, below 1 drops it sooner; 0 always draws the full mesh. See `loading §2`. |
| `layers` | `number` | `1` | — | — | — | Three.js layers of every mesh of the model, as Mesh.layers. |
| `sway` | `number` | `0` | 0 … 2 | — | tunable | How much og.Wind sways this instance, as it sways og.TerrainScatter copies: 0 still (the default), 1 as it blows. Each vertex above the model's origin moves downwind by its height; its shadow and depth move with it. Visual only: the simulation never reads it, and a tick sways the same on every run. See `loading §2`. |
| `static` | `boolean` | `false` | — | — | — | Part of its scene's baked lighting: og bake lightmaps traces the light arriving at it into the scene's lightmap, and while the bake matches the scene it takes its ambient light from there (its lights' direct light, shadows and highlights still drawn every frame). Unbaked, it draws as any model. See `loading §2`. |
| `variant` | `number` | `-1` | ≥ -1 | — | tunable | The glTF material variant this instance draws (KHR_materials_variants), an index into GltfInfo.variants: every mesh the variant maps takes its material, the others keep theirs, on this instance only. -1 draws the glTF's own materials; an index past the model's variants does too. See `loading §2`. |

```ogt
component Model {
  doc "Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one."
  doc "The glTF's materials are shared by every instance of it; `tint` and `flash` are this instance's own, carried with its transform to the lit shaders (one draw for every instance, whatever each shows), so one of a crowd can flash when struck."
  field asset Asset "The glTF asset."
  field tint Color "A linear colour, 0 to 1 a channel, multiplied with every material's base colour on this instance only (white: as the glTF)." {
    default 1 1 1
  }
  field flash f32 "A hit flash on this instance only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out)." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
  field castShadow bool "Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's; false takes this instance out of every shadow pass. Set from GltfDesc.castShadow unless the scene gives the entity a `Model` table." {
    guide "loading §2"
    default true
  }
  field receiveShadow bool "Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table." {
    default true
  }
  field visible bool "Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away." {
    default true
  }
  field lodBias f32 "This instance's level of detail bias, multiplied with RenderSettings.lodBias: the renderer draws the coarsest cooked level whose error covers less than a pixel at the model's distance. Above 1 keeps detail farther, below 1 drops it sooner; 0 always draws the full mesh." {
    guide "loading §2"
    default 1
    min 0
    max 16
    tunable true
  }
  field layers u32 "Three.js layers of every mesh of the model, as Mesh.layers." {
    default 1
  }
  field sway f32 "How much og.Wind sways this instance, as it sways og.TerrainScatter copies: 0 still (the default), 1 as it blows. Each vertex above the model's origin moves downwind by its height; its shadow and depth move with it. Visual only: the simulation never reads it, and a tick sways the same on every run." {
    guide "loading §2"
    min 0
    max 2
    tunable true
  }
  field static bool "Part of its scene's baked lighting: og bake lightmaps traces the light arriving at it into the scene's lightmap, and while the bake matches the scene it takes its ambient light from there (its lights' direct light, shadows and highlights still drawn every frame). Unbaked, it draws as any model." {
    guide "loading §2"
  }
  field variant i32 "The glTF material variant this instance draws (KHR_materials_variants), an index into GltfInfo.variants: every mesh the variant maps takes its material, the others keep theirs, on this instance only. -1 draws the glTF's own materials; an index past the model's variants does too." {
    guide "loading §2"
    default -1
    min -1
    tunable true
  }
}
```

<sub>Module `load` line 133</sub>
