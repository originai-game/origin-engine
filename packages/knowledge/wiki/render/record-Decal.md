<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Decal"></a>

### `og.Decal` (component)

A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one); a normalMin to normalMax region of the same atlas is a tangent-space normal map. Later decals blend over earlier ones; at most 255 overlap at one place (the newest are kept).

See `looks §14`.

Token `og.Decal`, registered under the schema name `og.Decal`; its value type is `Decal`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the atlas. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The decal's opacity. |
| `uvMin` | `Vector2` | `{ x: 0, y: 0 }` | — | — | worldspace `none` | The atlas region's top-left corner, v down. |
| `uvMax` | `Vector2` | `{ x: 1, y: 1 }` | — | — | worldspace `none` | The atlas region's bottom-right corner, v down. |
| `emissive` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | Linear light added where the decal covers (glowing runes, lava). |
| `roughness` | `number` | `-1` | -1 … 1 | — | — | The surface roughness under the decal; -1 keeps the surface's. |
| `angleFade` | `number` | `75` | 0 … 90 | `deg` | — | Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade). |
| `normalMin` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The top-left corner of the decal's normal map in RenderSettings.decalAtlas, v down (glTF's convention: +X along u, +Y up the image). Equal to normalMax (the default), the decal has no normal map. |
| `normalMax` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The bottom-right corner of the decal's normal map in the atlas, v down. |
| `normalScale` | `number` | `1` | 0 … 4 | — | tunable | How strongly the normal map bends the surface, as three.js normalScale. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Decal {
  doc "A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one); a normalMin to normalMax region of the same atlas is a tangent-space normal map. Later decals blend over earlier ones; at most 255 overlap at one place (the newest are kept)."
  guide "looks §14"
  field color Color "A linear colour multiplied with the atlas." {
    default 1 1 1
  }
  field opacity f32 "The decal's opacity." {
    default 1
    min 0
    max 1
    tunable true
  }
  field uvMin Vector2 "The atlas region's top-left corner, v down." {
    worldspace none
  }
  field uvMax Vector2 "The atlas region's bottom-right corner, v down." {
    worldspace none
    default 1 1
  }
  field emissive Color "Linear light added where the decal covers (glowing runes, lava)."
  field roughness f32 "The surface roughness under the decal; -1 keeps the surface's." {
    default -1
    min -1
    max 1
  }
  field angleFade f32 "Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade)." {
    default 75
    min 0
    max 90
    unit deg
  }
  field normalMin Vector2 "The top-left corner of the decal's normal map in RenderSettings.decalAtlas, v down (glTF's convention: +X along u, +Y up the image). Equal to normalMax (the default), the decal has no normal map."
  field normalMax Vector2 "The bottom-right corner of the decal's normal map in the atlas, v down."
  field normalScale f32 "How strongly the normal map bends the surface, as three.js normalScale." {
    default 1
    min 0
    max 4
    tunable true
  }
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>Module `render` line 2154</sub>
