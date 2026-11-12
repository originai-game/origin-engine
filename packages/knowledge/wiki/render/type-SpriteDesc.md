<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SpriteDesc"></a>

### `SpriteDesc` (desc struct)

A textured or solid quad in the world's xy plane facing +z (or turned to the camera, `facing`), drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene.

A desc: every field may be omitted and takes its default; `{}` is valid.

#### Image

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The texture from og.load.texture; null draws a solid quad of the colour. |
| `array` | `TextureArray \| null` | `null` | — | — | — | Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`. See `looks §13`. |
| `arrayLayer` | `number` | `0` | — | — | — | With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer. |

#### Placement

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the anchor point is. |
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its width in world units. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its height in world units. |
| `anchor` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | — | The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre. |
| `rotation` | `number` | `0` | — | `rad` | — | Counter-clockwise rotation about the anchor. |

#### Colour

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation). |

#### Order

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `layer` | `number` | `0` | — | — | — | Draw order among sprites and text; higher draws on top. |

#### Effects

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flash` | `number` | `0` | 0 … 1 | — | — | A hit flash: the colour mixed towards flashColor by this much, keeping the alpha. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `glow` | `number` | `0` | ≥ 0 | — | — | Strength in the 2D glow buffer (PostEffects.glow); 0 none. |
| `blend` | `SpriteBlend` | `"alpha"` | — | — | — | Alpha or additive. |

#### Shadows and light

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `castShadow` | `boolean` | `false` | — | — | — | Block 2D light (Lighting2D) with the sprite's rectangle. |
| `dropShadow` | `number` | `0` | 0 … 1 | — | — | A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground (dropShadowHeight, or the og.Terrain), drawn before its layer's sprites, faded in the sun's shadow; none with the sun set. |
| `dropShadowHeight` | `number` | `0` | — | `m` | — | The height of the level ground the drop shadow lies on (a floor, a deck, a step); 0 with no terrain is y = 0. Where an og.Terrain under the anchor is higher, the shadow lies on it: on the plane fitted to its slope, or bent over a hollow or a ridge narrower than the shadow. See `looks §13`. |
| `toneMapped` | `boolean` | `true` | — | — | — | As three.js Material.toneMapped: true draws the sprite through exposure, bloom and the tone curve. false keeps its colours as given (clamped 0 to 1), still placed, depth tested and sorted, outside exposure, bloom, fog and the curve; grade and screen effects apply. Additive sprites always tone map. See `looks §13`. |

#### Billboard and flips

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `facing` | `SpriteFacing` | `"none"` | — | — | — | A billboard: none keeps the quad in the world's xy plane; camera turns it into the view plane of each camera that draws it; cameraY stands it upright turned about y towards the camera. World sprites only; see SpriteFacing for sorting, culling and its drop shadow. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror the region left to right. |
| `flipY` | `boolean` | `false` | — | — | — | Mirror the region top to bottom. |

#### Sort

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sortY` | `boolean` | `false` | — | — | — | Within its layer, draw by y: lower on the screen draws in front (top-down games). |

#### Nine-slice

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slice` | `[number, number, number, number] \| null` | `null` | — | — | — | Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole. |
| `sliceScale` | `number` | `0.0625` | ≥ 0 | — | — | Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets. |

#### Screen space

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame: position in CSS pixels from the top left, y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested. flash, glow, blend and castShadow are world-only. UI_LAYER splits it from og.ui. |
| `clip` | `[number, number, number, number] \| null` | `null` | — | — | — | Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole. |

```ogt
struct SpriteDesc {
  doc "A textured or solid quad in the world's xy plane facing +z (or turned to the camera, `facing`), drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene."
  desc true
  group "Image"
  field texture Option.Texture "The texture from og.load.texture; null draws a solid quad of the colour."
  field array Option.TextureArray "Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`." {
    guide "looks §13"
  }
  field arrayLayer u32 "With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer."
  group "Placement"
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
  group "Colour"
  field color Color "A linear colour multiplied with the texture." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field uv Option.Array.f32.4 "The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation)."
  group "Order"
  field layer i32 "Draw order among sprites and text; higher draws on top."
  group "Effects"
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
  group "Shadows and light"
  field castShadow bool "Block 2D light (Lighting2D) with the sprite's rectangle."
  field dropShadow f32 "A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground (dropShadowHeight, or the og.Terrain), drawn before its layer's sprites, faded in the sun's shadow; none with the sun set." {
    min 0
    max 1
  }
  field dropShadowHeight f32 "The height of the level ground the drop shadow lies on (a floor, a deck, a step); 0 with no terrain is y = 0. Where an og.Terrain under the anchor is higher, the shadow lies on it: on the plane fitted to its slope, or bent over a hollow or a ridge narrower than the shadow." {
    guide "looks §13"
    unit m
  }
  field toneMapped bool "As three.js Material.toneMapped: true draws the sprite through exposure, bloom and the tone curve. false keeps its colours as given (clamped 0 to 1), still placed, depth tested and sorted, outside exposure, bloom, fog and the curve; grade and screen effects apply. Additive sprites always tone map." {
    guide "looks §13"
    default true
  }
  group "Billboard and flips"
  field facing SpriteFacing "A billboard: none keeps the quad in the world's xy plane; camera turns it into the view plane of each camera that draws it; cameraY stands it upright turned about y towards the camera. World sprites only; see SpriteFacing for sorting, culling and its drop shadow."
  field flipX bool "Mirror the region left to right."
  field flipY bool "Mirror the region top to bottom."
  group "Sort"
  field sortY bool "Within its layer, draw by y: lower on the screen draws in front (top-down games)."
  group "Nine-slice"
  field slice Option.Array.f32.4 "Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole."
  field sliceScale f32 "Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets." {
    default 0.0625
    min 0
  }
  group "Screen space"
  field screen bool "Draw in screen space over the finished frame: position in CSS pixels from the top left, y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested. flash, glow, blend and castShadow are world-only. UI_LAYER splits it from og.ui."
  field clip Option.Array.f32.4 "Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole."
}
```

<sub>Module `render` line 2386</sub>
