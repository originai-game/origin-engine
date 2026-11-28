<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Decal"></a>

### `og.Decal` (component)

A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of its texture (a solid colour without one); the same region of normalMap is a tangent-space normal map. Later decals blend over earlier ones; at most 255 overlap at one place (the newest are kept).

Token `og.Decal`, registered under the schema name `og.Decal`; its value type is `Decal`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The image (og.load.texture): its colour times color, its alpha the coverage. A sheet of several marks is one texture, each decal a uvMin to uvMax region of it. 0 is a solid colour. |
| `normalMap` | `Texture` | `0` | — | — | — | A tangent-space normal map over the same region (og.load.texture; glTF's convention: +X along u, +Y up the image) that bends the surface under the decal: a crack, a bullet hole's rim, embossed lettering. 0 is none. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The decal's opacity. |
| `uvMin` | `Vector2` | `{ x: 0, y: 0 }` | — | — | worldspace `none` | The top-left corner of the decal's region of texture and normalMap, v down. |
| `uvMax` | `Vector2` | `{ x: 1, y: 1 }` | — | — | worldspace `none` | The region's bottom-right corner, v down. |
| `emissive` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | Linear light added where the decal covers (glowing runes, lava). |
| `roughness` | `number` | `-1` | -1 … 1 | — | — | The surface roughness under the decal; -1 keeps the surface's. |
| `metallic` | `number` | `-1` | -1 … 1 | — | — | The surface metalness under the decal; -1 keeps the surface's. |
| `wet` | `boolean` | `false` | — | — | — | Multiply the receiver's albedo by the decal colour and blend its normal toward the decal's normal (flat without a map), using opacity as coverage. Set roughness low for standing water. False keeps the colour mix and additive normal bend. |
| `angleFade` | `number` | `75` | 0 … 90 | `deg` | — | Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade). |
| `normalScale` | `number` | `1` | 0 … 4 | — | tunable | How strongly normalMap bends the surface, as three.js normalScale. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Decal {
  doc "A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of its texture (a solid colour without one); the same region of normalMap is a tangent-space normal map. Later decals blend over earlier ones; at most 255 overlap at one place (the newest are kept)."
  field texture Texture "The image (og.load.texture): its colour times color, its alpha the coverage. A sheet of several marks is one texture, each decal a uvMin to uvMax region of it. 0 is a solid colour."
  field normalMap Texture "A tangent-space normal map over the same region (og.load.texture; glTF's convention: +X along u, +Y up the image) that bends the surface under the decal: a crack, a bullet hole's rim, embossed lettering. 0 is none."
  field color Color "A linear colour multiplied with the texture." {
    default 1 1 1
  }
  field opacity f32 "The decal's opacity." {
    default 1
    min 0
    max 1
    tunable true
  }
  field uvMin Vector2 "The top-left corner of the decal's region of texture and normalMap, v down." {
    worldspace none
  }
  field uvMax Vector2 "The region's bottom-right corner, v down." {
    worldspace none
    default 1 1
  }
  field emissive Color "Linear light added where the decal covers (glowing runes, lava)."
  field roughness f32 "The surface roughness under the decal; -1 keeps the surface's." {
    default -1
    min -1
    max 1
  }
  field metallic f32 "The surface metalness under the decal; -1 keeps the surface's." {
    default -1
    min -1
    max 1
  }
  field wet bool "Multiply the receiver's albedo by the decal colour and blend its normal toward the decal's normal (flat without a map), using opacity as coverage. Set roughness low for standing water. False keeps the colour mix and additive normal bend." {
    default false
  }
  field angleFade f32 "Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade)." {
    default 75
    min 0
    max 90
    unit deg
  }
  field normalScale f32 "How strongly normalMap bends the surface, as three.js normalScale." {
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

<sub>Module `render` line 2244</sub>
