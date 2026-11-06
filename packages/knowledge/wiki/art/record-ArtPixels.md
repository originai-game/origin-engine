<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

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

<sub>`api.ogt` line 26506</sub>
