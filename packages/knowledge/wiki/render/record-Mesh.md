<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Mesh"></a>

### `og.Mesh` (component)

Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh.

The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say.

Token `og.Mesh`, registered under the schema name `og.Mesh`; its value type is `Mesh`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The geometry. |
| `material` | `Material` | `0` | — | — | — | The material. |
| `castShadow` | `boolean` | `false` | — | — | — | Cast shadows, as three.js Object3D.castShadow: drawn into the sun's cascades and the shadowed lights' maps. Off by default: a lamp's own glass around its light, a glow card or a decal-like slab casts none. |
| `receiveShadow` | `boolean` | `true` | — | — | — | Receive shadows, as three.js Object3D.receiveShadow: darkened where the sun's cascades and the shadowed lights' maps say something stands between it and the light. On by default (three.js's is off). false lights it as if unshadowed by every light, while it still casts as castShadow says. See `render §3`. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash on this mesh only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Carried with its transform to the lit shaders, as og.Model's, so a flashing mesh is one draw with the other meshes of its geometry and material, and the amount is continuous. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour, 0 to 1 a channel (8 bits a channel). |
| `materialLayer` | `number` | `-1` | ≥ -1 | — | tunable | The texture-array layer this mesh's material reads (StandardMaterialDesc.mapArray and normalMapArray), in place of the material's mapLayer and normalMapLayer: one material, one draw, a layer per mesh (a tile set, a crowd's skins). -1 keeps the material's layers; a material without arrays ignores it. |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Object3D.layers.mask: bit n set puts it on layer n. A camera (og.Camera, og.CameraView, og.Mirror) draws it when their masks share a bit, and og.render.raycast tests it when its own `layers` does. Shadow maps take every caster whatever its layers. See `render §3`. |

```ogt
component Mesh {
  doc "Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh."
  doc "The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say."
  field geometry Geometry "The geometry."
  field material Material "The material."
  field castShadow bool "Cast shadows, as three.js Object3D.castShadow: drawn into the sun's cascades and the shadowed lights' maps. Off by default: a lamp's own glass around its light, a glow card or a decal-like slab casts none."
  field receiveShadow bool "Receive shadows, as three.js Object3D.receiveShadow: darkened where the sun's cascades and the shadowed lights' maps say something stands between it and the light. On by default (three.js's is off). false lights it as if unshadowed by every light, while it still casts as castShadow says." {
    default true
    guide "render §3"
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field flash f32 "A hit flash on this mesh only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Carried with its transform to the lit shaders, as og.Model's, so a flashing mesh is one draw with the other meshes of its geometry and material, and the amount is continuous." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour, 0 to 1 a channel (8 bits a channel)." {
    default 1 1 1
  }
  field materialLayer i32 "The texture-array layer this mesh's material reads (StandardMaterialDesc.mapArray and normalMapArray), in place of the material's mapLayer and normalMapLayer: one material, one draw, a layer per mesh (a tile set, a crowd's skins). -1 keeps the material's layers; a material without arrays ignores it." {
    default -1
    min -1
    tunable true
  }
  field layers u32 "Three.js layers, as Object3D.layers.mask: bit n set puts it on layer n. A camera (og.Camera, og.CameraView, og.Mirror) draws it when their masks share a bit, and og.render.raycast tests it when its own `layers` does. Shadow maps take every caster whatever its layers." {
    default 1
    guide "render §3"
  }
}
```

<sub>Module `render` line 477</sub>
