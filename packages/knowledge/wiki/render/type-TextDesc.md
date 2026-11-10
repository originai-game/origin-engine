<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-TextDesc"></a>

### `TextDesc` (desc struct)

A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `font` | `Font \| null` | `null` | — | — | — | The font from og.load.font; null draws nothing. |
| `text` | `string` | `""` | — | — | — | The text; `\n` starts a new line. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The baseline origin of the first line. |
| `size` | `number` | `1` | ≥ 0 | — | — | The em size in world units. |
| `rotation` | `number` | `0` | — | `rad` | — | Rotation about +z. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `layer` | `number` | `0` | — | — | — | Draw order among sprites and text; higher draws on top. |
| `align` | `TextAlign` | `"start"` | — | — | — | How each line sits on `position`: start puts its left end there, center its middle, end its right end. |
| `outline` | `number` | `0` | 0 … 1 | — | — | World text: an outline around the glyphs, as a fraction (0 to 1) of the widest the font's distance range allows (a font cooked with a larger px_range takes thicker outlines). |
| `outlineColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The outline's linear colour. |
| `facing` | `SpriteFacing` | `"none"` | — | — | — | World text as a billboard (SpriteFacing): none keeps it in the world's xy plane; camera turns it into each camera's view plane; cameraY stands it upright turned about y towards the camera (a name over a character). It turns as one panel about `position`, sorts among the transparent meshes by that point's view depth and is not culled. Screen text ignores it. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it. |
| `clip` | `[number, number, number, number] \| null` | `null` | — | — | — | Screen text: cut to [x, y, width, height] in CSS pixels, per pixel; null draws it whole. |

```ogt
struct TextDesc {
  doc "A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show)."
  desc true
  field font Option.Font "The font from og.load.font; null draws nothing."
  field text String "The text; `\\n` starts a new line."
  field position Vector3 "The baseline origin of the first line." {
    unit m
  }
  field size f32 "The em size in world units." {
    default 1
    min 0
  }
  field rotation f32 "Rotation about +z." {
    unit rad
  }
  field color Color "The linear colour." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field layer i32 "Draw order among sprites and text; higher draws on top."
  field align TextAlign "How each line sits on `position`: start puts its left end there, center its middle, end its right end."
  field outline f32 "World text: an outline around the glyphs, as a fraction (0 to 1) of the widest the font's distance range allows (a font cooked with a larger px_range takes thicker outlines)." {
    min 0
    max 1
  }
  field outlineColor Color "The outline's linear colour."
  field facing SpriteFacing "World text as a billboard (SpriteFacing): none keeps it in the world's xy plane; camera turns it into each camera's view plane; cameraY stands it upright turned about y towards the camera (a name over a character). It turns as one panel about `position`, sorts among the transparent meshes by that point's view depth and is not culled. Screen text ignores it."
  field screen bool "Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it."
  field clip Option.Array.f32.4 "Screen text: cut to [x, y, width, height] in CSS pixels, per pixel; null draws it whole."
}
```

<sub>Module `render` line 2159</sub>
