<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `art` module

A game's art contract: what every asset of its style holds, as seven resources a look sets beside the render ones (docs/art.md §2.1): the palette its texels sit on (ArtPalette), one pixel size for 2D art (ArtPixels), one texel density for 3D surfaces (ArtTexels), the shape language of its models (ArtShape), its line (ArtLine), the step of its motion (ArtMotion) and its camera (ArtCamera). A style is the assets, not a filter on the frame: the look's render resources treat the frame, these say what the assets themselves must be.

Every field is no constraint at its zero value (0, false, the first variant, an empty list), so a look without these blocks, and every game made before them, cooks and draws exactly as before. The built-in presets (templates/looks/<slug>.ogt) carry each catalogue style's values and a game's look overrides them field by field. They are resources, so game code reads the contract too (a sprite snapping to ArtPixels.unit, a spawner picking colours from ArtPalette); project.ogt's `look` names the look whose contract the cook reads. Writing them changes neither the frame nor the simulation: the cook's conformance of assets, the runtime defaults from them and og style check are docs/art.md §2.3 to §2.5.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module art {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 15124</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`ArtRotation`](#type-ArtRotation) | enum | How the style's pixel art turns (ArtPixels.rotation). |
| [`ArtRotation()`](#column-ArtRotation) | column kind | An ArtRotation field: the variant's index. |
| [`ArtFilter`](#type-ArtFilter) | enum | How the style's textures are sampled (ArtTexels.filter). |
| [`ArtFilter()`](#column-ArtFilter) | column kind | An ArtFilter field: the variant's index. |
| [`ArtMips`](#type-ArtMips) | enum | The mip maps of the style's textures (ArtTexels.mips). |
| [`ArtMips()`](#column-ArtMips) | column kind | An ArtMips field: the variant's index. |
| [`ArtNormals`](#type-ArtNormals) | enum | How the style's models are shaded across their faces (ArtShape.normals). |
| [`ArtNormals()`](#column-ArtNormals) | column kind | An ArtNormals field: the variant's index. |
| [`ArtLinePlace`](#type-ArtLinePlace) | enum | Where the style's line lies against a shape (ArtLine.place). |
| [`ArtLinePlace()`](#column-ArtLinePlace) | column kind | An ArtLinePlace field: the variant's index. |
| [`ArtProjection`](#type-ArtProjection) | enum | The projection of the style's camera (ArtCamera.projection). |
| [`ArtProjection()`](#column-ArtProjection) | column kind | An ArtProjection field: the variant's index. |
| [`og.ArtPalette`](#record-ArtPalette) | resource | The style's colours: the palette the assets' texels sit on, how many a single asset may use, how many shades a hue ramps through, which colours are kept for signals, and how two colours mix where a tone falls between them. The frame's own quantise is Stylise.palette; this is what the assets are made of, 2D and 3D alike. Every field is no constraint at its zero value. |
| [`og.ArtPixels`](#record-ArtPixels) | resource | The style's pixel grid for 2D art: one logical pixel size for every sprite, tile and backdrop, the resolution the art is drawn for, the cell sizes sprites come in, how pixel art turns and whether it stands on whole pixels. The frame's grid is Stylise.pixelHeight; this is what the assets are drawn at. Every field is no constraint at its zero value. |
| [`og.ArtTexels`](#record-ArtTexels) | resource | The style's texel density on 3D surfaces: one density for the whole world, so no model is sharper or blurrier than its neighbours, the largest texture, and how textures are filtered and mipped. Every field is no constraint at its zero value. |
| [`og.ArtShape`](#record-ArtShape) | resource | The style's shape language for 3D models: triangle budgets, how faces are shaded, bevelled edges, a voxel grid and the smallest feature a silhouette keeps. Every field is no constraint at its zero value (normalAngle matters only with normals angle). |
| [`og.ArtLine`](#record-ArtLine) | resource | The style's line: the outline round its sprites and meshes, its colour, where it lies against the shape and how it thins with distance. The frame's own lines are MaterialStyle.outlineWidth and Stylise.ink; this is the line the assets carry. Every field is no constraint at its zero value (color matters only with a width). |
| [`og.ArtMotion`](#record-ArtMotion) | resource | The style's motion: animation held on steps as hand animation is, how many frames a clip may hold, and how much squash and stretch its movement takes. Every field is no constraint at its zero value. |
| [`og.ArtCamera`](#record-ArtCamera) | resource | The style's camera: its projection, a fixed angle (side-on, top-down, the isometric presets), the range of its field of view and whether a 2D camera moves by whole pixels. Every field is no constraint at its zero value (pitch and yaw matter only with fixedAngle). |

## Components, resources and events

<a id="record-ArtPalette"></a>

### `og.ArtPalette` (resource)

The style's colours: the palette the assets' texels sit on, how many a single asset may use, how many shades a hue ramps through, which colours are kept for signals, and how two colours mix where a tone falls between them. The frame's own quantise is Stylise.palette; this is what the assets are made of, 2D and 3D alike. Every field is no constraint at its zero value.

Token `og.ArtPalette`, registered under the schema name `og.ArtPalette`; its value type is `ArtPalette`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `colors` | `Palette` | `0` | — | — | — | The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour. |
| `strict` | `boolean` | `false` | — | — | — | Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette. |
| `maxColors` | `number` | `0` | 0 … 256 | — | — | The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit. |
| `rampSteps` | `number` | `0` | 0 … 32 | — | — | The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit. |
| `accents` | `readonly number[]` | `[]` | — | — | a list of at most 8 `u32` (`List.u32.8`): reads a frozen copy, writes a whole array | The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none. |
| `dither` | `DitherPattern` | `"none"` | — | — | — | How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither. |

```ogt
resource ArtPalette {
  doc "The style's colours: the palette the assets' texels sit on, how many a single asset may use, how many shades a hue ramps through, which colours are kept for signals, and how two colours mix where a tone falls between them. The frame's own quantise is Stylise.palette; this is what the assets are made of, 2D and 3D alike. Every field is no constraint at its zero value."
  field colors Palette "The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour."
  field strict bool "Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette."
  field maxColors u32 "The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit." {
    min 0
    max 256
  }
  field rampSteps u32 "The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit." {
    min 0
    max 32
  }
  field accents List.u32.8 "The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none."
  field dither DitherPattern "How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither."
}
```

<sub>`api.ogt` line 15215</sub>

<a id="record-ArtPixels"></a>

### `og.ArtPixels` (resource)

The style's pixel grid for 2D art: one logical pixel size for every sprite, tile and backdrop, the resolution the art is drawn for, the cell sizes sprites come in, how pixel art turns and whether it stands on whole pixels. The frame's grid is Stylise.pixelHeight; this is what the assets are drawn at. Every field is no constraint at its zero value.

Token `og.ArtPixels`, registered under the schema name `og.ArtPixels`; its value type is `ArtPixels`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `unit` | `number` | `0` | 0 … 4096 | — | — | Logical pixels per metre (per world unit in 2D): one pixel of every sprite, tile and backdrop covers 1/unit metres, so no two assets show pixels of different sizes (16 with 16-pixel tiles one metre wide); 0 is no constraint. |
| `logicalHeight` | `number` | `0` | 0 … 2160 | — | — | The rows of the logical resolution the art is drawn for, as Stylise.pixelHeight counts them (180, 224, 240); 0 is no constraint. |
| `spriteSizes` | `readonly number[]` | `[]` | — | — | a list of at most 8 `u32` (`List.u32.8`): reads a frozen copy, writes a whole array | The cell sizes, in logical pixels, a sprite, a flipbook frame or a tile may have: each side is one of them (`spriteSizes 16 32` takes 16 x 16, 16 x 32 and 32 x 32). A look gives them on one line; empty is any size. |
| `rotation` | `ArtRotation` | `"free"` | — | — | — | How pixel art turns: free (no constraint), nearest, cleanEdge, or prerotated in rotationSteps. |
| `rotationSteps` | `number` | `0` | 0 … 64 | — | — | With rotation prerotated: the rotations baked per full turn (8 is one every 45 degrees); 0 is none. |
| `snap` | `boolean` | `false` | — | — | — | Sprites and tiles stand on whole logical pixels: their positions are multiples of 1/unit metres, so no pixel of the art lies between two of the grid. |

```ogt
resource ArtPixels {
  doc "The style's pixel grid for 2D art: one logical pixel size for every sprite, tile and backdrop, the resolution the art is drawn for, the cell sizes sprites come in, how pixel art turns and whether it stands on whole pixels. The frame's grid is Stylise.pixelHeight; this is what the assets are drawn at. Every field is no constraint at its zero value."
  field unit f32 "Logical pixels per metre (per world unit in 2D): one pixel of every sprite, tile and backdrop covers 1/unit metres, so no two assets show pixels of different sizes (16 with 16-pixel tiles one metre wide); 0 is no constraint." {
    min 0
    max 4096
  }
  field logicalHeight u32 "The rows of the logical resolution the art is drawn for, as Stylise.pixelHeight counts them (180, 224, 240); 0 is no constraint." {
    min 0
    max 2160
  }
  field spriteSizes List.u32.8 "The cell sizes, in logical pixels, a sprite, a flipbook frame or a tile may have: each side is one of them (`spriteSizes 16 32` takes 16 x 16, 16 x 32 and 32 x 32). A look gives them on one line; empty is any size."
  field rotation ArtRotation "How pixel art turns: free (no constraint), nearest, cleanEdge, or prerotated in rotationSteps."
  field rotationSteps u32 "With rotation prerotated: the rotations baked per full turn (8 is one every 45 degrees); 0 is none." {
    min 0
    max 64
  }
  field snap bool "Sprites and tiles stand on whole logical pixels: their positions are multiples of 1/unit metres, so no pixel of the art lies between two of the grid."
}
```

<sub>`api.ogt` line 15231</sub>

<a id="record-ArtTexels"></a>

### `og.ArtTexels` (resource)

The style's texel density on 3D surfaces: one density for the whole world, so no model is sharper or blurrier than its neighbours, the largest texture, and how textures are filtered and mipped. Every field is no constraint at its zero value.

Token `og.ArtTexels`, registered under the schema name `og.ArtTexels`; its value type is `ArtTexels`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `density` | `number` | `0` | 0 … 16384 | — | — | Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint. |
| `maxSize` | `number` | `0` | 0 … 16384 | — | — | The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit. |
| `filter` | `ArtFilter` | `"any"` | — | — | — | How the style's textures are sampled: any (no constraint), nearest or linear. |
| `mips` | `ArtMips` | `"any"` | — | — | — | Their mip maps: any (no constraint), full or none. |

```ogt
resource ArtTexels {
  doc "The style's texel density on 3D surfaces: one density for the whole world, so no model is sharper or blurrier than its neighbours, the largest texture, and how textures are filtered and mipped. Every field is no constraint at its zero value."
  field density f32 "Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint." {
    min 0
    max 16384
  }
  field maxSize u32 "The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit." {
    min 0
    max 16384
  }
  field filter ArtFilter "How the style's textures are sampled: any (no constraint), nearest or linear."
  field mips ArtMips "Their mip maps: any (no constraint), full or none."
}
```

<sub>`api.ogt` line 15250</sub>

<a id="record-ArtShape"></a>

### `og.ArtShape` (resource)

The style's shape language for 3D models: triangle budgets, how faces are shaded, bevelled edges, a voxel grid and the smallest feature a silhouette keeps. Every field is no constraint at its zero value (normalAngle matters only with normals angle).

Token `og.ArtShape`, registered under the schema name `og.ArtShape`; its value type is `ArtShape`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `propFaces` | `number` | `0` | ≥ 0 | — | — | The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget. |
| `characterFaces` | `number` | `0` | ≥ 0 | — | — | The most triangles one character may have; 0 is no budget. |
| `normals` | `ArtNormals` | `"any"` | — | — | — | How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle). |
| `normalAngle` | `number` | `30` | 0 … 180 | `deg` | — | With normals angle: the angle between two faces past which their edge is shaded hard. |
| `bevel` | `number` | `0` | ≥ 0 | `m` | — | Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none. |
| `voxel` | `number` | `0` | ≥ 0 | `m` | — | Models built of cubes of this edge (voxel worlds); 0 is no voxel grid. |
| `silhouette` | `number` | `0` | ≥ 0 | `m` | — | The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint. |

```ogt
resource ArtShape {
  doc "The style's shape language for 3D models: triangle budgets, how faces are shaded, bevelled edges, a voxel grid and the smallest feature a silhouette keeps. Every field is no constraint at its zero value (normalAngle matters only with normals angle)."
  field propFaces u32 "The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget." {
    min 0
  }
  field characterFaces u32 "The most triangles one character may have; 0 is no budget." {
    min 0
  }
  field normals ArtNormals "How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle)."
  field normalAngle f32 "With normals angle: the angle between two faces past which their edge is shaded hard." {
    default 30
    min 0
    max 180
    unit deg
  }
  field bevel f32 "Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none." {
    min 0
    unit m
  }
  field voxel f32 "Models built of cubes of this edge (voxel worlds); 0 is no voxel grid." {
    min 0
    unit m
  }
  field silhouette f32 "The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint." {
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 15264</sub>

<a id="record-ArtLine"></a>

### `og.ArtLine` (resource)

The style's line: the outline round its sprites and meshes, its colour, where it lies against the shape and how it thins with distance. The frame's own lines are MaterialStyle.outlineWidth and Stylise.ink; this is the line the assets carry. Every field is no constraint at its zero value (color matters only with a width).

Token `og.ArtLine`, registered under the schema name `og.ArtLine`; its value type is `ArtLine`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | 0 … 32 | — | — | The line's width: in logical pixels round a sprite (1 is one dark pixel round the shape), in pixels at distanceFalloff round a mesh (as MaterialStyle.outlineWidth); 0 is no line. |
| `color` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The line's colour (linear; a look may write an sRGB "#rrggbb"). |
| `place` | `ArtLinePlace` | `"outside"` | — | — | — | Where the line lies against the shape: outside (the zero value), inside, or both. |
| `distanceFalloff` | `number` | `0` | ≥ 0 | `m` | — | Round a mesh: the distance at which the line is width wide, nearer wider and farther thinner (as MaterialStyle.outlineDistance); 0 keeps one width. |

```ogt
resource ArtLine {
  doc "The style's line: the outline round its sprites and meshes, its colour, where it lies against the shape and how it thins with distance. The frame's own lines are MaterialStyle.outlineWidth and Stylise.ink; this is the line the assets carry. Every field is no constraint at its zero value (color matters only with a width)."
  field width f32 "The line's width: in logical pixels round a sprite (1 is one dark pixel round the shape), in pixels at distanceFalloff round a mesh (as MaterialStyle.outlineWidth); 0 is no line." {
    min 0
    max 32
  }
  field color Color "The line's colour (linear; a look may write an sRGB \"#rrggbb\")." {
    default 0.02 0.02 0.03
  }
  field place ArtLinePlace "Where the line lies against the shape: outside (the zero value), inside, or both."
  field distanceFalloff f32 "Round a mesh: the distance at which the line is width wide, nearer wider and farther thinner (as MaterialStyle.outlineDistance); 0 keeps one width." {
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 15293</sub>

<a id="record-ArtMotion"></a>

### `og.ArtMotion` (resource)

The style's motion: animation held on steps as hand animation is, how many frames a clip may hold, and how much squash and stretch its movement takes. Every field is no constraint at its zero value.

Token `og.ArtMotion`, registered under the schema name `og.ArtMotion`; its value type is `ArtMotion`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stepFps` | `number` | `0` | 0 … 120 | `hz` | — | Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint. |
| `maxFrames` | `number` | `0` | ≥ 0 | — | — | The most frames one clip or flipbook may hold; 0 is no limit. |
| `squash` | `number` | `0` | 0 … 1 | `ratio` | — | The default squash and stretch of the style's movement (og.tween presets, landings, hits): 0 rigid, 1 as rubber hose. |

```ogt
resource ArtMotion {
  doc "The style's motion: animation held on steps as hand animation is, how many frames a clip may hold, and how much squash and stretch its movement takes. Every field is no constraint at its zero value."
  field stepFps f32 "Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint." {
    min 0
    max 120
    unit hz
  }
  field maxFrames u32 "The most frames one clip or flipbook may hold; 0 is no limit." {
    min 0
  }
  field squash f32 "The default squash and stretch of the style's movement (og.tween presets, landings, hits): 0 rigid, 1 as rubber hose." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>`api.ogt` line 15309</sub>

<a id="record-ArtCamera"></a>

### `og.ArtCamera` (resource)

The style's camera: its projection, a fixed angle (side-on, top-down, the isometric presets), the range of its field of view and whether a 2D camera moves by whole pixels. Every field is no constraint at its zero value (pitch and yaw matter only with fixedAngle).

Token `og.ArtCamera`, registered under the schema name `og.ArtCamera`; its value type is `ArtCamera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `projection` | `ArtProjection` | `"any"` | — | — | — | The projection: any (no constraint), perspective or orthographic. |
| `fixedAngle` | `boolean` | `false` | — | — | — | The camera keeps one angle, pitch and yaw: false is no constraint. |
| `pitch` | `number` | `0` | -90 … 90 | `deg` | — | With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric). |
| `yaw` | `number` | `0` | -180 … 180 | `deg` | — | With fixedAngle: its turn about the vertical axis (45 for isometric). |
| `fovMin` | `number` | `0` | 0 … 179 | `deg` | — | The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit. |
| `fovMax` | `number` | `0` | 0 … 179 | `deg` | — | The widest vertical field of view a perspective camera of the style takes; 0 is no limit. |
| `pixelPerfect` | `boolean` | `false` | — | — | — | A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls. |

```ogt
resource ArtCamera {
  doc "The style's camera: its projection, a fixed angle (side-on, top-down, the isometric presets), the range of its field of view and whether a 2D camera moves by whole pixels. Every field is no constraint at its zero value (pitch and yaw matter only with fixedAngle)."
  field projection ArtProjection "The projection: any (no constraint), perspective or orthographic."
  field fixedAngle bool "The camera keeps one angle, pitch and yaw: false is no constraint."
  field pitch f32 "With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric)." {
    min -90
    max 90
    unit deg
  }
  field yaw f32 "With fixedAngle: its turn about the vertical axis (45 for isometric)." {
    min -180
    max 180
    unit deg
  }
  field fovMin f32 "The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit." {
    min 0
    max 179
    unit deg
  }
  field fovMax f32 "The widest vertical field of view a perspective camera of the style takes; 0 is no limit." {
    min 0
    max 179
    unit deg
  }
  field pixelPerfect bool "A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls."
}
```

<sub>`api.ogt` line 15326</sub>

## Types

<a id="type-ArtRotation"></a>

### `ArtRotation` (enum)

How the style's pixel art turns (ArtPixels.rotation).

A string: one of the variants below. The zero value is `"free"`.

| Variant | Doc |
|---|---|
| `"free"` | No constraint: sprites turn with their texture's filter. |
| `"nearest"` | Rotated with nearest sampling: pixels stay square and hard, with the jagged edges of a rotated grid. |
| `"cleanEdge"` | Rotated through the cleanEdge filter (torcado, MIT): no blur and no jaggies, the edges of the art redrawn at the angle. |
| `"prerotated"` | Rotations baked into the sheet, ArtPixels.rotationSteps of them per turn, each redrawn as pixel art: the sprite shows the nearest one and never turns between. |

```ogt
enum ArtRotation {
  doc "How the style's pixel art turns (ArtPixels.rotation)."
  variant free "No constraint: sprites turn with their texture's filter."
  variant nearest "Rotated with nearest sampling: pixels stay square and hard, with the jagged edges of a rotated grid."
  variant cleanEdge "Rotated through the cleanEdge filter (torcado, MIT): no blur and no jaggies, the edges of the art redrawn at the angle."
  variant prerotated "Rotations baked into the sheet, ArtPixels.rotationSteps of them per turn, each redrawn as pixel art: the sprite shows the nearest one and never turns between."
}
```

<sub>`api.ogt` line 15129</sub>

<a id="type-ArtFilter"></a>

### `ArtFilter` (enum)

How the style's textures are sampled (ArtTexels.filter).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: each texture's own import settings. |
| `"nearest"` | Nearest: texels stay square and crisp up close, as low-resolution 3D and pixel art. |
| `"linear"` | Linear: texels blend smoothly. |

```ogt
enum ArtFilter {
  doc "How the style's textures are sampled (ArtTexels.filter)."
  variant any "No constraint: each texture's own import settings."
  variant nearest "Nearest: texels stay square and crisp up close, as low-resolution 3D and pixel art."
  variant linear "Linear: texels blend smoothly."
}
```

<sub>`api.ogt` line 15144</sub>

<a id="type-ArtMips"></a>

### `ArtMips` (enum)

The mip maps of the style's textures (ArtTexels.mips).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: each texture's own import settings. |
| `"full"` | A full mip chain: no shimmer in the distance. |
| `"none"` | The base level only: texels stay as crisp far away as near, as on early 3D hardware. |

```ogt
enum ArtMips {
  doc "The mip maps of the style's textures (ArtTexels.mips)."
  variant any "No constraint: each texture's own import settings."
  variant full "A full mip chain: no shimmer in the distance."
  variant none "The base level only: texels stay as crisp far away as near, as on early 3D hardware."
}
```

<sub>`api.ogt` line 15158</sub>

<a id="type-ArtNormals"></a>

### `ArtNormals` (enum)

How the style's models are shaded across their faces (ArtShape.normals).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: each model's own normals. |
| `"smooth"` | Smooth: vertex normals averaged over the faces that share them, soft forms. |
| `"flat"` | Flat: one normal per face, every facet visible (faceted low poly, paper craft, voxels). |
| `"angle"` | Split where neighbouring faces meet at more than ArtShape.normalAngle: hard edges stay hard, curves stay smooth. |

```ogt
enum ArtNormals {
  doc "How the style's models are shaded across their faces (ArtShape.normals)."
  variant any "No constraint: each model's own normals."
  variant smooth "Smooth: vertex normals averaged over the faces that share them, soft forms."
  variant flat "Flat: one normal per face, every facet visible (faceted low poly, paper craft, voxels)."
  variant angle "Split where neighbouring faces meet at more than ArtShape.normalAngle: hard edges stay hard, curves stay smooth."
}
```

<sub>`api.ogt` line 15172</sub>

<a id="type-ArtLinePlace"></a>

### `ArtLinePlace` (enum)

Where the style's line lies against a shape (ArtLine.place).

A string: one of the variants below. The zero value is `"outside"`.

| Variant | Doc |
|---|---|
| `"outside"` | Round the shape: the line grows the silhouette by its width. |
| `"inside"` | On the shape's own edge pixels: its size is unchanged. |
| `"both"` | Centred on the edge: half outside, half inside. |

```ogt
enum ArtLinePlace {
  doc "Where the style's line lies against a shape (ArtLine.place)."
  variant outside "Round the shape: the line grows the silhouette by its width."
  variant inside "On the shape's own edge pixels: its size is unchanged."
  variant both "Centred on the edge: half outside, half inside."
}
```

<sub>`api.ogt` line 15187</sub>

<a id="type-ArtProjection"></a>

### `ArtProjection` (enum)

The projection of the style's camera (ArtCamera.projection).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: the camera as the game makes it. |
| `"perspective"` | Perspective: far things smaller. |
| `"orthographic"` | Orthographic: every distance at one scale (2D, isometric, top-down tactics). |

```ogt
enum ArtProjection {
  doc "The projection of the style's camera (ArtCamera.projection)."
  variant any "No constraint: the camera as the game makes it."
  variant perspective "Perspective: far things smaller."
  variant orthographic "Orthographic: every distance at one scale (2D, isometric, top-down tactics)."
}
```

<sub>`api.ogt` line 15201</sub>

## Column kinds

Each is a factory, `ArtRotation(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-ArtRotation"></a>`ArtRotation` | `ArtRotation` | `Uint32Array` | 1 | An ArtRotation field: the variant's index. |
| <a id="column-ArtFilter"></a>`ArtFilter` | `ArtFilter` | `Uint32Array` | 1 | An ArtFilter field: the variant's index. |
| <a id="column-ArtMips"></a>`ArtMips` | `ArtMips` | `Uint32Array` | 1 | An ArtMips field: the variant's index. |
| <a id="column-ArtNormals"></a>`ArtNormals` | `ArtNormals` | `Uint32Array` | 1 | An ArtNormals field: the variant's index. |
| <a id="column-ArtLinePlace"></a>`ArtLinePlace` | `ArtLinePlace` | `Uint32Array` | 1 | An ArtLinePlace field: the variant's index. |
| <a id="column-ArtProjection"></a>`ArtProjection` | `ArtProjection` | `Uint32Array` | 1 | An ArtProjection field: the variant's index. |

```ogt
column ArtRotation {
  doc "An ArtRotation field: the variant's index."
  array Uint32Array
  stride 1
  value ArtRotation
}
```

<sub>`api.ogt` line 15137</sub>

```ogt
column ArtFilter {
  doc "An ArtFilter field: the variant's index."
  array Uint32Array
  stride 1
  value ArtFilter
}
```

<sub>`api.ogt` line 15151</sub>

```ogt
column ArtMips {
  doc "An ArtMips field: the variant's index."
  array Uint32Array
  stride 1
  value ArtMips
}
```

<sub>`api.ogt` line 15165</sub>

```ogt
column ArtNormals {
  doc "An ArtNormals field: the variant's index."
  array Uint32Array
  stride 1
  value ArtNormals
}
```

<sub>`api.ogt` line 15180</sub>

```ogt
column ArtLinePlace {
  doc "An ArtLinePlace field: the variant's index."
  array Uint32Array
  stride 1
  value ArtLinePlace
}
```

<sub>`api.ogt` line 15194</sub>

```ogt
column ArtProjection {
  doc "An ArtProjection field: the variant's index."
  array Uint32Array
  stride 1
  value ArtProjection
}
```

<sub>`api.ogt` line 15208</sub>
