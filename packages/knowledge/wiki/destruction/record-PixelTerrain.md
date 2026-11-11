<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-PixelTerrain"></a>

### `og.PixelTerrain` (component)

Destructible 2D ground of square cells: ground, bedrock or empty. Its og.Transform's position is cell (0, 0)'s bottom-left corner, x right, y up (rotation and scale unused). Filled the first tick it is seen; carve2d, fill2d, explode2d and fracture2d change it in the tick (og.destruction2d, before og.physics).

Each `chunk` × `chunk` block with a solid cell is an entity with a static og.Collider (SHAPE_TRIMESH) of its outline smoothed within `tolerance`, rebuilt only when a change touches it. Ground no longer joined through cell sides to bedrock or the bottom row falls as og.PixelPiece entities.

See `physics §9`.

Also known as: destructible terrain, pixel terrain, worms terrain, noita, carve ground, deformable terrain, 2D destruction

Token `og.PixelTerrain`, registered under the schema name `og.PixelTerrain`; its value type is `PixelTerrain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `256` | 1 … 4096 | — | — | Its cells across. Changing it starts the terrain over, filled again. |
| `height` | `number` | `128` | 1 … 4096 | — | — | Its cells up. Changing it starts the terrain over, filled again. |
| `cellSize` | `number` | `0.0625` | ≥ 0.001 | `m` | — | A cell's side. |
| `chunk` | `number` | `32` | 4 … 256 | — | — | The side of the blocks of cells that collide and rebuild together: smaller rebuilds less per carve and makes more colliders. Changing it starts the terrain over, filled again. |
| `ground` | `number` | `0.5` | 0 … 1 | `ratio` | — | How much of its height is ground (bedrock included) when it is first filled; 0 starts it empty, to build with fill2d. |
| `bedrock` | `number` | `1` | — | — | — | The rows of bedrock at its bottom when it is first filled: never carved, and what the ground must stay joined to (as to the bottom row) to stand. |
| `seed` | `number` | `0` | — | — | — | The seed of its grain and of the pieces explode2d and fracture2d cut. |
| `color` | `Color` | `{ r: 0.30, g: 0.18, b: 0.09 }` | — | — | — | The ground's colour. |
| `bedrockColor` | `Color` | `{ r: 0.10, g: 0.10, b: 0.11 }` | — | — | — | The bedrock's colour. |
| `grain` | `number` | `0.15` | 0 … 1 | — | — | How much each ground cell's brightness varies, fixed per cell by the seed (pieces keep the grain of the ground they were). |
| `layer` | `number` | `0` | — | — | — | Its sprite's draw order (its pieces' too); higher draws on top. |
| `friction` | `number` | `0.6` | ≥ 0 | — | — | Its colliders' and its pieces' friction. |
| `density` | `number` | `1000` | ≥ 1 | `kgPerM3` | — | Its pieces' density (per square metre in 2D). |
| `layers` | `number` | `1` | — | — | — | Its colliders' and pieces' collision layers, as og.Collider.layers. |
| `mask` | `number` | `4294967295` | — | — | — | The layers they collide with, as og.Collider.mask. |
| `tolerance` | `number` | `0.5` | 0 … 4 | — | — | How far, in cells, the collision outline may cut across the cells' stair steps. |
| `minPiece` | `number` | `4` | — | — | — | An island cut loose with fewer cells vanishes instead of falling as an og.PixelPiece. |
| `maxPieces` | `number` | `64` | 1 … 1024 | — | — | The most og.PixelPiece entities of this terrain at once: past it the oldest despawn. |
| `solid` | `number` | `0` | — | — | read only | Written: its solid cells (ground and bedrock). |

```ogt
component PixelTerrain {
  doc "Destructible 2D ground of square cells: ground, bedrock or empty. Its og.Transform's position is cell (0, 0)'s bottom-left corner, x right, y up (rotation and scale unused). Filled the first tick it is seen; carve2d, fill2d, explode2d and fracture2d change it in the tick (og.destruction2d, before og.physics)."
  doc "Each `chunk` × `chunk` block with a solid cell is an entity with a static og.Collider (SHAPE_TRIMESH) of its outline smoothed within `tolerance`, rebuilt only when a change touches it. Ground no longer joined through cell sides to bedrock or the bottom row falls as og.PixelPiece entities."
  guide "physics §9"
  aliases "destructible terrain, pixel terrain, worms terrain, noita, carve ground, deformable terrain, 2D destruction"
  field width u32 "Its cells across. Changing it starts the terrain over, filled again." {
    default 256
    min 1
    max 4096
  }
  field height u32 "Its cells up. Changing it starts the terrain over, filled again." {
    default 128
    min 1
    max 4096
  }
  field cellSize f32 "A cell's side." {
    default 0.0625
    min 0.001
    unit m
  }
  field chunk u32 "The side of the blocks of cells that collide and rebuild together: smaller rebuilds less per carve and makes more colliders. Changing it starts the terrain over, filled again." {
    default 32
    min 4
    max 256
  }
  field ground f32 "How much of its height is ground (bedrock included) when it is first filled; 0 starts it empty, to build with fill2d." {
    default 0.5
    min 0
    max 1
    unit ratio
  }
  field bedrock u32 "The rows of bedrock at its bottom when it is first filled: never carved, and what the ground must stay joined to (as to the bottom row) to stand." {
    default 1
  }
  field seed u32 "The seed of its grain and of the pieces explode2d and fracture2d cut."
  field color Color "The ground's colour." {
    default 0.30 0.18 0.09
  }
  field bedrockColor Color "The bedrock's colour." {
    default 0.10 0.10 0.11
  }
  field grain f32 "How much each ground cell's brightness varies, fixed per cell by the seed (pieces keep the grain of the ground they were)." {
    default 0.15
    min 0
    max 1
  }
  field layer i32 "Its sprite's draw order (its pieces' too); higher draws on top."
  field friction f32 "Its colliders' and its pieces' friction." {
    default 0.6
    min 0
  }
  field density f32 "Its pieces' density (per square metre in 2D)." {
    default 1000
    min 1
    unit kgPerM3
  }
  field layers u32 "Its colliders' and pieces' collision layers, as og.Collider.layers." {
    default 1
  }
  field mask u32 "The layers they collide with, as og.Collider.mask." {
    default 4294967295
  }
  field tolerance f32 "How far, in cells, the collision outline may cut across the cells' stair steps." {
    default 0.5
    min 0
    max 4
  }
  field minPiece u32 "An island cut loose with fewer cells vanishes instead of falling as an og.PixelPiece." {
    default 4
  }
  field maxPieces u32 "The most og.PixelPiece entities of this terrain at once: past it the oldest despawn." {
    default 64
    min 1
    max 1024
  }
  field solid u32 "Written: its solid cells (ground and bedrock)." {
    readonly true
  }
}
```

<sub>Module `destruction` line 308</sub>
