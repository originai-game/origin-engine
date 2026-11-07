<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SpriteDesc"></a>

### `SpriteDesc` (desc struct)

A textured or solid quad in the world's xy plane facing +z (or turned to the camera, `facing`), drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The texture from og.load.texture; null draws a solid quad of the colour. |
| `array` | `TextureArray \| null` | `null` | — | — | — | Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows, so a flipbook's frames packed as layers batch across frames. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`. |
| `arrayLayer` | `number` | `0` | — | — | — | With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the anchor point is. |
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its width in world units. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its height in world units. |
| `anchor` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | — | The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre. |
| `rotation` | `number` | `0` | — | `rad` | — | Counter-clockwise rotation about the anchor. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation). |
| `layer` | `number` | `0` | — | — | — | Draw order among sprites and text; higher draws on top. |
| `flash` | `number` | `0` | 0 … 1 | — | — | A hit flash: the colour mixed towards flashColor by this much, keeping the alpha. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `glow` | `number` | `0` | ≥ 0 | — | — | Strength in the 2D glow buffer (PostEffects.glow); 0 none. |
| `blend` | `SpriteBlend` | `"alpha"` | — | — | — | Alpha or additive. |
| `castShadow` | `boolean` | `false` | — | — | — | Block 2D light (Lighting2D) with the sprite's rectangle. |
| `dropShadow` | `number` | `0` | 0 … 1 | — | — | A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground under it (dropShadowHeight, or the og.Terrain there), drawn before every sprite of its layer: pixel sprites standing in a 3D set cast shadows that agree with it. Where the ground already lies in the sun's cascaded shadow the drop shadow fades, so a shadow inside a shadow is no darker. None with the sun below the horizon; at most five times the sprite's height long under a low one. World sprites with a perspective camera over the xz ground; a 2D camera sees the ground edge on. |
| `dropShadowHeight` | `number` | `0` | — | `m` | — | The height of the level ground the drop shadow lies on (a floor, a deck, a step). Where an og.Terrain under the sprite's anchor is higher, the shadow lies on the terrain instead, on the plane fitted to its slope under the shadow, or bent over a hollow or a ridge narrower than the shadow (the terrain's heights under a 4 x 2 grid of the shadow's cells). 0 with no terrain is the ground plane y = 0. |
| `toneMapped` | `boolean` | `true` | — | — | — | As three.js Material.toneMapped: true draws the sprite into the lit scene, carried by exposure (auto exposure included), bloom and the tone curve like any surface. false keeps its colours as given: still placed in the world, depth tested and sorted with the other sprites, but shown unexposed and uncurved (clamped to 0 to 1), outside bloom and fog; the grade, vignette, grain and screen flash still apply. For world UI that must read the same at noon and at dusk: speech bubbles, name tags, markers. Alpha blending only: an additive sprite adds light and is always tone mapped. |
| `facing` | `SpriteFacing` | `"none"` | — | — | — | A billboard: none keeps the quad in the world's xy plane; camera turns it into the view plane of each camera that draws it; cameraY stands it upright turned about y towards the camera (trees, characters). World sprites only (a screen sprite ignores it); see SpriteFacing for sorting, culling and its drop shadow. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror the region left to right. |
| `flipY` | `boolean` | `false` | — | — | — | Mirror the region top to bottom. |
| `sortY` | `boolean` | `false` | — | — | — | Within its layer, draw by y: lower on the screen draws in front (top-down games). |
| `slice` | `[number, number, number, number] \| null` | `null` | — | — | — | Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole. |
| `sliceScale` | `number` | `0.0625` | ≥ 0 | — | — | Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it. |
| `clip` | `[number, number, number, number] \| null` | `null` | — | — | — | Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole. |

```ogt
struct SpriteDesc {
  doc "A textured or solid quad in the world's xy plane facing +z (or turned to the camera, `facing`), drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene."
  desc true
  field texture Option.Texture "The texture from og.load.texture; null draws a solid quad of the colour."
  field array Option.TextureArray "Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows, so a flipbook's frames packed as layers batch across frames. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`."
  field arrayLayer u32 "With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer."
  field position Vector3 "Where the anchor point is." {
    unit m
  }
  field width f32 "Its width in world units." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its height in world units." {
    default 1
    min 0
    unit m
  }
  field anchor Vector2 "The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre." {
    default 0.5 0.5
  }
  field rotation f32 "Counter-clockwise rotation about the anchor." {
    unit rad
  }
  field color Color "A linear colour multiplied with the texture." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field uv Option.Array.f32.4 "The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation)."
  field layer i32 "Draw order among sprites and text; higher draws on top."
  field flash f32 "A hit flash: the colour mixed towards flashColor by this much, keeping the alpha." {
    min 0
    max 1
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
  field glow f32 "Strength in the 2D glow buffer (PostEffects.glow); 0 none." {
    min 0
  }
  field blend SpriteBlend "Alpha or additive."
  field castShadow bool "Block 2D light (Lighting2D) with the sprite's rectangle."
  field dropShadow f32 "A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground under it (dropShadowHeight, or the og.Terrain there), drawn before every sprite of its layer: pixel sprites standing in a 3D set cast shadows that agree with it. Where the ground already lies in the sun's cascaded shadow the drop shadow fades, so a shadow inside a shadow is no darker. None with the sun below the horizon; at most five times the sprite's height long under a low one. World sprites with a perspective camera over the xz ground; a 2D camera sees the ground edge on." {
    min 0
    max 1
  }
  field dropShadowHeight f32 "The height of the level ground the drop shadow lies on (a floor, a deck, a step). Where an og.Terrain under the sprite's anchor is higher, the shadow lies on the terrain instead, on the plane fitted to its slope under the shadow, or bent over a hollow or a ridge narrower than the shadow (the terrain's heights under a 4 x 2 grid of the shadow's cells). 0 with no terrain is the ground plane y = 0." {
    unit m
  }
  field toneMapped bool "As three.js Material.toneMapped: true draws the sprite into the lit scene, carried by exposure (auto exposure included), bloom and the tone curve like any surface. false keeps its colours as given: still placed in the world, depth tested and sorted with the other sprites, but shown unexposed and uncurved (clamped to 0 to 1), outside bloom and fog; the grade, vignette, grain and screen flash still apply. For world UI that must read the same at noon and at dusk: speech bubbles, name tags, markers. Alpha blending only: an additive sprite adds light and is always tone mapped." {
    default true
  }
  field facing SpriteFacing "A billboard: none keeps the quad in the world's xy plane; camera turns it into the view plane of each camera that draws it; cameraY stands it upright turned about y towards the camera (trees, characters). World sprites only (a screen sprite ignores it); see SpriteFacing for sorting, culling and its drop shadow."
  field flipX bool "Mirror the region left to right."
  field flipY bool "Mirror the region top to bottom."
  field sortY bool "Within its layer, draw by y: lower on the screen draws in front (top-down games)."
  field slice Option.Array.f32.4 "Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole."
  field sliceScale f32 "Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets." {
    default 0.0625
    min 0
  }
  field screen bool "Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it."
  field clip Option.Array.f32.4 "Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole."
}
```

<sub>Module `render` line 2216</sub>
