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
| `castShadow` | `boolean` | `true` | — | — | — | Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's (a loaded model casts, as it always has); false takes this instance out of every shadow pass. world.instantiate of a glTF and a scene's `model` line set it from the load's GltfDesc.castShadow, unless the scene gives the entity a `Model` table (whose castShadow is on unless it says false). |
| `receiveShadow` | `boolean` | `true` | — | — | — | Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away. |
| `lodBias` | `number` | `1` | 0 … 16 | — | tunable | This instance's level of detail bias, multiplied with RenderSettings.lodBias: the cook simplifies every mesh into levels (a model's import settings, `model { lods 1 0.5 0.25 }` by default, the triangle fraction of each level; a glTF with MSFT_lod keeps its own levels) and measures each level's error against the full mesh, and the renderer draws the coarsest level whose error covers less than a pixel at the model's distance, with hysteresis so it does not flicker at the boundary. Above 1 keeps detail farther (a hero), below 1 drops it sooner (a crowd); 0 always draws the full mesh. dump_entity shows the level drawn. |
| `layers` | `number` | `1` | — | — | — | Three.js layers of every mesh of the model, as Mesh.layers. |
| `sway` | `number` | `0` | 0 … 2 | — | tunable | How much og.Wind sways this instance, as it sways og.TerrainScatter copies: 0 still (the default; a building, a rock), 1 as it blows (a tree, a bush), between for stiffer trunks. Each vertex above the model's origin moves downwind by its height (the root stays put, the top moves furthest), in the model's units, in gusts sweeping across the scene; its shadow and depth move with it. Visual only: nothing in the simulation reads it, and the same tick sways the same on every run. A material with its own shader asset keeps it and stays still; a posed (animated) model sways too. |
| `static` | `boolean` | `false` | — | — | — | Part of its scene's baked lighting: og bake lightmaps gives a static model lightmap UVs (xatlas, over its world-space triangles) and traces the light arriving at its surfaces from the scene into the scene's lightmap (`<scene>.lightmap.ogt` and `.ktx2` beside the scene). Once baked, the model draws with it (its ambient light from the lightmap instead of the sky; its lights' direct light, shadows and highlights still drawn every frame) for as long as the bake matches the scene. A scene entity says `static true` beside its `model` line; an unbaked static model draws as any model, and og doctor --project names the scene. |
| `variant` | `number` | `-1` | ≥ -1 | — | tunable | The glTF material variant this instance draws (KHR_materials_variants), an index into GltfInfo.variants: every mesh the variant maps takes its material, the others keep theirs, on this instance only (a car's paint, a chair's fabric picked at run time, one draw per material as before). -1 draws the glTF's own materials; an index past the model's variants does too. |

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
  field castShadow bool "Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's (a loaded model casts, as it always has); false takes this instance out of every shadow pass. world.instantiate of a glTF and a scene's `model` line set it from the load's GltfDesc.castShadow, unless the scene gives the entity a `Model` table (whose castShadow is on unless it says false)." {
    default true
  }
  field receiveShadow bool "Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table." {
    default true
  }
  field visible bool "Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away." {
    default true
  }
  field lodBias f32 "This instance's level of detail bias, multiplied with RenderSettings.lodBias: the cook simplifies every mesh into levels (a model's import settings, `model { lods 1 0.5 0.25 }` by default, the triangle fraction of each level; a glTF with MSFT_lod keeps its own levels) and measures each level's error against the full mesh, and the renderer draws the coarsest level whose error covers less than a pixel at the model's distance, with hysteresis so it does not flicker at the boundary. Above 1 keeps detail farther (a hero), below 1 drops it sooner (a crowd); 0 always draws the full mesh. dump_entity shows the level drawn." {
    default 1
    min 0
    max 16
    tunable true
  }
  field layers u32 "Three.js layers of every mesh of the model, as Mesh.layers." {
    default 1
  }
  field sway f32 "How much og.Wind sways this instance, as it sways og.TerrainScatter copies: 0 still (the default; a building, a rock), 1 as it blows (a tree, a bush), between for stiffer trunks. Each vertex above the model's origin moves downwind by its height (the root stays put, the top moves furthest), in the model's units, in gusts sweeping across the scene; its shadow and depth move with it. Visual only: nothing in the simulation reads it, and the same tick sways the same on every run. A material with its own shader asset keeps it and stays still; a posed (animated) model sways too." {
    min 0
    max 2
    tunable true
  }
  field static bool "Part of its scene's baked lighting: og bake lightmaps gives a static model lightmap UVs (xatlas, over its world-space triangles) and traces the light arriving at its surfaces from the scene into the scene's lightmap (`<scene>.lightmap.ogt` and `.ktx2` beside the scene). Once baked, the model draws with it (its ambient light from the lightmap instead of the sky; its lights' direct light, shadows and highlights still drawn every frame) for as long as the bake matches the scene. A scene entity says `static true` beside its `model` line; an unbaked static model draws as any model, and og doctor --project names the scene."
  field variant i32 "The glTF material variant this instance draws (KHR_materials_variants), an index into GltfInfo.variants: every mesh the variant maps takes its material, the others keep theirs, on this instance only (a car's paint, a chair's fabric picked at run time, one draw per material as before). -1 draws the glTF's own materials; an index past the model's variants does too." {
    default -1
    min -1
    tunable true
  }
}
```

<sub>`api.ogt` line 8012</sub>
