<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-EffectMesh"></a>

### `og.EffectMesh` (component)

A mesh drawn with an effect material (og.load.effect) at the entity's MatrixWorld: a shockwave dome, a slash arc, a portal's swirl, with the material's scrolling textures, erosion and fresnel. Its curves run on `life`, which the game or a tween drives from 0 to 1 (a composed effect's `mesh` element is one og.vfx drives).

Also known as: effect mesh, VFX mesh, shockwave mesh, Effekseer model, Niagara mesh renderer

Token `og.EffectMesh`, registered under the schema name `og.EffectMesh`; its value type is `EffectMesh`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The mesh (og.render's geometries, a glTF's); 0 draws nothing. |
| `effect` | `EffectMaterial` | `0` | — | — | — | The material (og.load.effect); 0 draws nothing. |
| `life` | `number` | `0` | 0 … 1 | — | tunable | Where it is in its life, 0 to 1, for the material's curves: tween it from 0 to 1 for a one-shot. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint times the material's; above 1 blooms. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha it is multiplied by. |
| `seed` | `number` | `0` | — | — | — | Its random value's seed (the material's `random` and noise offset). |
| `blend` | `Blend` | `"additive"` | — | — | — | Its blend when the material names none: additive (the default, a glow) or alpha. |
| `billboard` | `boolean` | `false` | — | — | — | Turned to face the camera: its rotation and scale are taken in the camera's frame (x right, y up, z towards the eye), so a ring in its xy plane faces the eye from any side. |

```ogt
component EffectMesh {
  doc "A mesh drawn with an effect material (og.load.effect) at the entity's MatrixWorld: a shockwave dome, a slash arc, a portal's swirl, with the material's scrolling textures, erosion and fresnel. Its curves run on `life`, which the game or a tween drives from 0 to 1 (a composed effect's `mesh` element is one og.vfx drives)."
  aliases "effect mesh, VFX mesh, shockwave mesh, Effekseer model, Niagara mesh renderer"
  field geometry Geometry "The mesh (og.render's geometries, a glTF's); 0 draws nothing."
  field effect EffectMaterial "The material (og.load.effect); 0 draws nothing."
  field life f32 "Where it is in its life, 0 to 1, for the material's curves: tween it from 0 to 1 for a one-shot." {
    min 0
    max 1
    tunable true
  }
  field color Color "A linear tint times the material's; above 1 blooms." {
    default 1 1 1
  }
  field opacity f32 "The alpha it is multiplied by." {
    default 1
    min 0
    max 1
    tunable true
  }
  field seed u32 "Its random value's seed (the material's `random` and noise offset)."
  field blend Blend "Its blend when the material names none: additive (the default, a glow) or alpha." {
    default additive
  }
  field billboard bool "Turned to face the camera: its rotation and scale are taken in the camera's frame (x right, y up, z towards the eye), so a ring in its xy plane faces the eye from any side."
}
```

<sub>Module `render` line 2457</sub>
