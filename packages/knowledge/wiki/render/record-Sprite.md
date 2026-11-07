<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Sprite"></a>

### `og.Sprite` (component)

A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z, in the xy plane or turned to the camera by `facing`; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame.

Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer.

Also known as: 2D image, SpriteRenderer, Sprite2D

Token `og.Sprite`, registered under the schema name `og.Sprite`; its value type is `Sprite`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture (og.load.texture); 0 draws a solid quad of the colour. |
| `array` | `TextureArray` | `0` | — | — | — | Draw from a layer of this texture array (og.load.textureArray) instead of `texture`; 0 uses `texture`. Every sprite of one array is one draw whichever layer each shows. While the array's streaming group is not resident the sprite draws nothing. |
| `arrayLayer` | `number` | `0` | — | — | — | With `array`: the layer drawn (past the last layer, the last). uvMin and uvMax are the region within the layer; a nine-slice measures the layer. |
| `width` | `number` | `1` | ≥ 0 | `m` | tunable | Its width in world units (times Transform.scale.x). |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | Its height in world units (times Transform.scale.y). |
| `anchor` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | worldspace `none` | The point of the quad at the entity's position, as a fraction from the bottom-left corner: (0.5, 0) stands it on its feet. A flipbook frame's pivot replaces it. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha. |
| `uvMin` | `Vector2` | `{ x: 0, y: 0 }` | — | — | worldspace `none` | The texture region's top-left corner, v down. |
| `uvMax` | `Vector2` | `{ x: 1, y: 1 }` | — | — | worldspace `none` | The texture region's bottom-right corner, v down. |
| `layer` | `number` | `0` | — | — | — | Draw order among sprites, tiles and text; higher draws on top. |
| `sortY` | `boolean` | `false` | — | — | — | Within its layer, lower y draws in front: top-down characters, props and trees overlap by their feet. |
| `facing` | `SpriteFacing` | `"none"` | — | — | — | A billboard (SpriteDesc.facing): none keeps the quad in the world's xy plane; camera turns it into each camera's view plane; cameraY stands it upright turned about y towards the camera, for trees and characters in a 3D set. Its afterimages face as it does; its blob shadow stays flat. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror left to right (a character facing left). |
| `flipY` | `boolean` | `false` | — | — | — | Mirror top to bottom. |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash: the colour mixed towards flashColor by this much (og.tween.flash fades it out). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `glow` | `number` | `0` | ≥ 0 | — | — | Strength in the 2D glow buffer (PostEffects.glow); 0 none. |
| `blend` | `SpriteBlend` | `"alpha"` | — | — | — | Alpha or additive. |
| `castShadow` | `boolean` | `false` | — | — | — | Block 2D light (Lighting2D) with the sprite's rectangle. |
| `sliceX` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Nine-slice: the left and right insets in texture pixels of the region (0 0 with sliceY 0 0 draws it whole). World panels, health bars, speech bubbles. |
| `sliceY` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Nine-slice: the top and bottom insets in texture pixels. |
| `sliceScale` | `number` | `0.0625` | ≥ 0 | — | — | Nine-slice: world units per texture pixel of the insets. |
| `shadow` | `number` | `0` | 0 … 1 | — | tunable | A soft blob shadow on the ground under the sprite's anchor, this dark (0 none, 0.4 typical), drawn under every sprite of its layer. |
| `shadowSize` | `Vector2` | `{ x: 0, y: 0 }` | ≥ 0 | `m` | — | The blob's width and height; 0 0 is 0.8 by 0.3 of the sprite's width. |
| `shadowOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The blob's centre relative to the anchor. |
| `dropShadow` | `number` | `0` | 0 … 1 | — | tunable | A drop shadow this dark (0 none, 0.5 typical): the sprite's shape flattened along the sun onto the ground under it (dropShadowHeight, or the og.Terrain there), faded where that ground already lies in the sun's shadow, drawn before every sprite of its layer (SpriteDesc.dropShadow): pixel sprites standing in a 3D set. |
| `dropShadowHeight` | `number` | `0` | — | `m` | — | The height of the level ground the drop shadow lies on; where an og.Terrain under the anchor is higher, the terrain (SpriteDesc.dropShadowHeight). |
| `toneMapped` | `boolean` | `true` | — | — | — | As three.js Material.toneMapped: false keeps the sprite's colours as given, outside exposure, bloom, fog and the tone curve, still placed and depth tested in the world (SpriteDesc.toneMapped): speech bubbles and markers that must not blow out at dusk. Alpha blending only. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Mesh.layers: a world sprite is drawn by the cameras whose masks share a bit with it (a screen sprite draws on every camera's frame). |
| `outline` | `number` | `0` | 0 … 4 | — | — | A pixel outline this many logical pixels wide, drawn by the sprite shader round the texture's opaque texels (a diamond of whole logical pixels outside the art, the quad grown to hold it; a sheet's neighbouring frame never lines it), for a sprite whose line cannot be baked by the cook: scaled, tinted or made at run time. A logical pixel is 1 / ArtPixels.unit world units when the art contract sets a unit, else one texel of the region. 0 is none, or, under the project's art contract (project.ogt's `look`), the look's ArtLine.width for a sprite whose texture the cook did not line (one not cooked as pixel art) when the contract sets ArtPixels.unit. Untextured sprites take none. |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The outline's linear colour; at its default, the art contract's ArtLine.color when the line comes from the contract. |
| `palette` | `Texture` | `0` | — | — | — | A palette swap (docs/art.md §2.4): a texture of N rows of colours (load it as `kind pixel`), row r's texel i the colour palette index i takes in row r. The sprite then draws its texture's index texture (the cook's `<id>#index`, written beside a texture the art contract maps to a palette; a texture that is itself an index texture is drawn as it is) through row paletteRow: team colours, a hit flash in palette, a day and a night ramp. Flipbook frames swap too. 0 is none; a texture without an index texture draws as it is, with a warning. |
| `paletteRow` | `number` | `0` | — | — | — | The row of `palette` drawn (past the last, the last). |
| `material` | `Material` | `0` | — | — | — | Draw through a material shader's hooks: a material from og.render.shaderMaterial (an og.load.shader of `mode hooks`, the same asset a mesh draws with, or `mode sprite`), its parameters set with og.render.setShaderParam. vertex_offset moves the quad's corners, surface_color changes its colour before the 2D light (texture times tint and flash; an alpha of 0 cuts it, a dissolve), surface_emissive adds light after it; surface_normal is not called. The hooks see `local` as the point on the quad (x and y 0 to 1 from its bottom-left corner), `uv` the texture coordinate, `color` the tint and `time` the scene clock; a `mode sprite` shader may also call `sprite_texel(uv)`, the sprite's texture anywhere (distortion, a chromatic split). Dissolve, freeze, heat shimmer and palette effects are shaders, not engine features. Sprites of one material, texture and blend are one draw; a material per sprite (its own parameters) is a draw per sprite. Its drop shadow takes the shader's alpha; its glow, blob shadow and afterimages draw plain, as does a sprite drawn from a texture array, and a faced one keeps the sprite order. A hooks shader that calls a helper only meshes have draws its sprites plain, named in dump_render's spriteShaderErrors. 0 is none. |

```ogt
component Sprite {
  doc "A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z, in the xy plane or turned to the camera by `facing`; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame."
  doc "Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer."
  aliases "2D image, SpriteRenderer, Sprite2D"
  field texture Texture "The texture (og.load.texture); 0 draws a solid quad of the colour."
  field array TextureArray "Draw from a layer of this texture array (og.load.textureArray) instead of `texture`; 0 uses `texture`. Every sprite of one array is one draw whichever layer each shows. While the array's streaming group is not resident the sprite draws nothing."
  field arrayLayer u32 "With `array`: the layer drawn (past the last layer, the last). uvMin and uvMax are the region within the layer; a nine-slice measures the layer."
  field width f32 "Its width in world units (times Transform.scale.x)." {
    default 1
    min 0
    unit m
    tunable true
  }
  field height f32 "Its height in world units (times Transform.scale.y)." {
    default 1
    min 0
    unit m
    tunable true
  }
  field anchor Vector2 "The point of the quad at the entity's position, as a fraction from the bottom-left corner: (0.5, 0) stands it on its feet. A flipbook frame's pivot replaces it." {
    worldspace none
    default 0.5 0.5
  }
  field color Color "A linear colour multiplied with the texture." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
    tunable true
  }
  field uvMin Vector2 "The texture region's top-left corner, v down." {
    worldspace none
  }
  field uvMax Vector2 "The texture region's bottom-right corner, v down." {
    worldspace none
    default 1 1
  }
  field layer i32 "Draw order among sprites, tiles and text; higher draws on top."
  field sortY bool "Within its layer, lower y draws in front: top-down characters, props and trees overlap by their feet."
  field facing SpriteFacing "A billboard (SpriteDesc.facing): none keeps the quad in the world's xy plane; camera turns it into each camera's view plane; cameraY stands it upright turned about y towards the camera, for trees and characters in a 3D set. Its afterimages face as it does; its blob shadow stays flat."
  field flipX bool "Mirror left to right (a character facing left)."
  field flipY bool "Mirror top to bottom."
  field flash f32 "A hit flash: the colour mixed towards flashColor by this much (og.tween.flash fades it out)." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
  field glow f32 "Strength in the 2D glow buffer (PostEffects.glow); 0 none." {
    min 0
  }
  field blend SpriteBlend "Alpha or additive."
  field castShadow bool "Block 2D light (Lighting2D) with the sprite's rectangle."
  field sliceX Vector2 "Nine-slice: the left and right insets in texture pixels of the region (0 0 with sliceY 0 0 draws it whole). World panels, health bars, speech bubbles."
  field sliceY Vector2 "Nine-slice: the top and bottom insets in texture pixels."
  field sliceScale f32 "Nine-slice: world units per texture pixel of the insets." {
    default 0.0625
    min 0
  }
  field shadow f32 "A soft blob shadow on the ground under the sprite's anchor, this dark (0 none, 0.4 typical), drawn under every sprite of its layer." {
    min 0
    max 1
    tunable true
  }
  field shadowSize Vector2 "The blob's width and height; 0 0 is 0.8 by 0.3 of the sprite's width." {
    min 0
    unit m
  }
  field shadowOffset Vector2 "The blob's centre relative to the anchor." {
    unit m
  }
  field dropShadow f32 "A drop shadow this dark (0 none, 0.5 typical): the sprite's shape flattened along the sun onto the ground under it (dropShadowHeight, or the og.Terrain there), faded where that ground already lies in the sun's shadow, drawn before every sprite of its layer (SpriteDesc.dropShadow): pixel sprites standing in a 3D set." {
    min 0
    max 1
    tunable true
  }
  field dropShadowHeight f32 "The height of the level ground the drop shadow lies on; where an og.Terrain under the anchor is higher, the terrain (SpriteDesc.dropShadowHeight)." {
    unit m
  }
  field toneMapped bool "As three.js Material.toneMapped: false keeps the sprite's colours as given, outside exposure, bloom, fog and the tone curve, still placed and depth tested in the world (SpriteDesc.toneMapped): speech bubbles and markers that must not blow out at dusk. Alpha blending only." {
    default true
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field layers u32 "Three.js layers, as Mesh.layers: a world sprite is drawn by the cameras whose masks share a bit with it (a screen sprite draws on every camera's frame)." {
    default 1
  }
  field outline f32 "A pixel outline this many logical pixels wide, drawn by the sprite shader round the texture's opaque texels (a diamond of whole logical pixels outside the art, the quad grown to hold it; a sheet's neighbouring frame never lines it), for a sprite whose line cannot be baked by the cook: scaled, tinted or made at run time. A logical pixel is 1 / ArtPixels.unit world units when the art contract sets a unit, else one texel of the region. 0 is none, or, under the project's art contract (project.ogt's `look`), the look's ArtLine.width for a sprite whose texture the cook did not line (one not cooked as pixel art) when the contract sets ArtPixels.unit. Untextured sprites take none." {
    min 0
    max 4
  }
  field outlineColor Color "The outline's linear colour; at its default, the art contract's ArtLine.color when the line comes from the contract." {
    default 0.02 0.02 0.03
  }
  field palette Texture "A palette swap (docs/art.md §2.4): a texture of N rows of colours (load it as `kind pixel`), row r's texel i the colour palette index i takes in row r. The sprite then draws its texture's index texture (the cook's `<id>#index`, written beside a texture the art contract maps to a palette; a texture that is itself an index texture is drawn as it is) through row paletteRow: team colours, a hit flash in palette, a day and a night ramp. Flipbook frames swap too. 0 is none; a texture without an index texture draws as it is, with a warning."
  field paletteRow u32 "The row of `palette` drawn (past the last, the last)."
  field material Material "Draw through a material shader's hooks: a material from og.render.shaderMaterial (an og.load.shader of `mode hooks`, the same asset a mesh draws with, or `mode sprite`), its parameters set with og.render.setShaderParam. vertex_offset moves the quad's corners, surface_color changes its colour before the 2D light (texture times tint and flash; an alpha of 0 cuts it, a dissolve), surface_emissive adds light after it; surface_normal is not called. The hooks see `local` as the point on the quad (x and y 0 to 1 from its bottom-left corner), `uv` the texture coordinate, `color` the tint and `time` the scene clock; a `mode sprite` shader may also call `sprite_texel(uv)`, the sprite's texture anywhere (distortion, a chromatic split). Dissolve, freeze, heat shimmer and palette effects are shaders, not engine features. Sprites of one material, texture and blend are one draw; a material per sprite (its own parameters) is a draw per sprite. Its drop shadow takes the shader's alpha; its glow, blob shadow and afterimages draw plain, as does a sprite drawn from a texture array, and a faced one keeps the sprite order. A hooks shader that calls a helper only meshes have draws its sprites plain, named in dump_render's spriteShaderErrors. 0 is none."
}
```

<sub>Module `render` line 2294</sub>
