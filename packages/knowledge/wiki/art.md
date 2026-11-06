<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `art` module

A game's art contract: what every asset of its style holds, as seven resources a look sets beside the render ones (docs/art.md §2.1): the palette its texels sit on (ArtPalette), one pixel size for 2D art (ArtPixels), one texel density for 3D surfaces (ArtTexels), the shape language of its models (ArtShape), its line (ArtLine), the step of its motion (ArtMotion) and its camera (ArtCamera). A style is the assets, not a filter on the frame: the look's render resources treat the frame, these say what the assets themselves must be.

Every field is no constraint at its zero value (0, false, the first variant, an empty list), so a look without these blocks, and every game made before them, cooks and draws exactly as before. The built-in presets (templates/looks/<slug>.ogt) carry each catalogue style's values and a game's look overrides them field by field. They are resources, so game code reads the contract too (a sprite snapping to ArtPixels.unit, a spawner picking colours from ArtPalette); project.ogt's `look` names the look whose contract the cook reads. Writing them changes neither the frame nor the simulation: the cook's conformance of assets, the runtime defaults from them and og style check are docs/art.md §2.3 to §2.5.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module art {
  doc "A game's art contract: what every asset of its style holds, as seven resources a look sets beside the render ones (docs/art.md §2.1): the palette its texels sit on (ArtPalette), one pixel size for 2D art (ArtPixels), one texel density for 3D surfaces (ArtTexels), the shape language of its models (ArtShape), its line (ArtLine), the step of its motion (ArtMotion) and its camera (ArtCamera). A style is the assets, not a filter on the frame: the look's render resources treat the frame, these say what the assets themselves must be."
  doc "Every field is no constraint at its zero value (0, false, the first variant, an empty list), so a look without these blocks, and every game made before them, cooks and draws exactly as before. The built-in presets (templates/looks/<slug>.ogt) carry each catalogue style's values and a game's look overrides them field by field. They are resources, so game code reads the contract too (a sprite snapping to ArtPixels.unit, a spawner picking colours from ArtPalette); project.ogt's `look` names the look whose contract the cook reads. Writing them changes neither the frame nor the simulation: the cook's conformance of assets, the runtime defaults from them and og style check are docs/art.md §2.3 to §2.5."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 26504</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`ArtRotation`](art/type-ArtRotation.md) | enum |  | How the style's pixel art turns (ArtPixels.rotation). |
| [`ArtRotation()`](#column-ArtRotation) | column kind |  | An ArtRotation field: the variant's index. |
| [`ArtFilter`](art/type-ArtFilter.md) | enum |  | How the style's textures are sampled (ArtTexels.filter). |
| [`ArtFilter()`](#column-ArtFilter) | column kind |  | An ArtFilter field: the variant's index. |
| [`ArtMips`](art/type-ArtMips.md) | enum |  | The mip maps of the style's textures (ArtTexels.mips). |
| [`ArtMips()`](#column-ArtMips) | column kind |  | An ArtMips field: the variant's index. |
| [`ArtNormals`](art/type-ArtNormals.md) | enum |  | How the style's models are shaded across their faces (ArtShape.normals). |
| [`ArtNormals()`](#column-ArtNormals) | column kind |  | An ArtNormals field: the variant's index. |
| [`ArtLinePlace`](art/type-ArtLinePlace.md) | enum |  | Where the style's line lies against a shape (ArtLine.place). |
| [`ArtLinePlace()`](#column-ArtLinePlace) | column kind |  | An ArtLinePlace field: the variant's index. |
| [`ArtBrush`](art/type-ArtBrush.md) | enum |  | The brush a sprite's baked line is drawn with (ArtLine.brush), as Pixelorama's outline effect draws it: which texels within ArtLine.width of the shape's edge take the line. |
| [`ArtBrush()`](#column-ArtBrush) | column kind |  | An ArtBrush field: the variant's index. |
| [`ArtProjection`](art/type-ArtProjection.md) | enum |  | The projection of the style's camera (ArtCamera.projection). |
| [`ArtProjection()`](#column-ArtProjection) | column kind |  | An ArtProjection field: the variant's index. |
| [`og.ArtPalette`](art/record-ArtPalette.md) | resource |  | The style's colours: the palette the assets' texels sit on, how many a single asset may use, how many shades a hue ramps through, which colours are kept for signals, and how two colours mix where a t… |
| [`og.ArtPixels`](art/record-ArtPixels.md) | resource |  | The style's pixel grid for 2D art: one logical pixel size for every sprite, tile and backdrop, the resolution the art is drawn for, the cell sizes sprites come in, how pixel art turns and whether it … |
| [`og.ArtTexels`](art/record-ArtTexels.md) | resource |  | The style's texel density on 3D surfaces: one density for the whole world, so no model is sharper or blurrier than its neighbours, the largest texture, and how textures are filtered and mipped. |
| [`og.ArtShape`](art/record-ArtShape.md) | resource |  | The style's shape language for 3D models: triangle budgets, how faces are shaded, bevelled edges, a voxel grid and the smallest feature a silhouette keeps. |
| [`og.ArtLine`](art/record-ArtLine.md) | resource |  | The style's line: the outline round its sprites and meshes, its colour, where it lies against the shape and how it thins with distance. |
| [`og.ArtMotion`](art/record-ArtMotion.md) | resource |  | The style's motion: animation held on steps as hand animation is, how many frames a clip may hold, and how much squash and stretch its movement takes. |
| [`og.ArtCamera`](art/record-ArtCamera.md) | resource |  | The style's camera: its projection, a fixed angle (side-on, top-down, the isometric presets), the range of its field of view and whether a 2D camera moves by whole pixels. |

## Column kinds

Each is a factory, `ArtRotation(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-ArtRotation"></a>`ArtRotation` | `ArtRotation` | `Uint32Array` | 1 | An ArtRotation field: the variant's index. |
| <a id="column-ArtFilter"></a>`ArtFilter` | `ArtFilter` | `Uint32Array` | 1 | An ArtFilter field: the variant's index. |
| <a id="column-ArtMips"></a>`ArtMips` | `ArtMips` | `Uint32Array` | 1 | An ArtMips field: the variant's index. |
| <a id="column-ArtNormals"></a>`ArtNormals` | `ArtNormals` | `Uint32Array` | 1 | An ArtNormals field: the variant's index. |
| <a id="column-ArtLinePlace"></a>`ArtLinePlace` | `ArtLinePlace` | `Uint32Array` | 1 | An ArtLinePlace field: the variant's index. |
| <a id="column-ArtBrush"></a>`ArtBrush` | `ArtBrush` | `Uint32Array` | 1 | An ArtBrush field: the variant's index. |
| <a id="column-ArtProjection"></a>`ArtProjection` | `ArtProjection` | `Uint32Array` | 1 | An ArtProjection field: the variant's index. |

```ogt
column ArtRotation {
  doc "An ArtRotation field: the variant's index."
  array Uint32Array
  stride 1
  value ArtRotation
}
```

<sub>`api.ogt` line 26516</sub>

```ogt
column ArtFilter {
  doc "An ArtFilter field: the variant's index."
  array Uint32Array
  stride 1
  value ArtFilter
}
```

<sub>`api.ogt` line 26530</sub>

```ogt
column ArtMips {
  doc "An ArtMips field: the variant's index."
  array Uint32Array
  stride 1
  value ArtMips
}
```

<sub>`api.ogt` line 26544</sub>

```ogt
column ArtNormals {
  doc "An ArtNormals field: the variant's index."
  array Uint32Array
  stride 1
  value ArtNormals
}
```

<sub>`api.ogt` line 26559</sub>

```ogt
column ArtLinePlace {
  doc "An ArtLinePlace field: the variant's index."
  array Uint32Array
  stride 1
  value ArtLinePlace
}
```

<sub>`api.ogt` line 26573</sub>

```ogt
column ArtBrush {
  doc "An ArtBrush field: the variant's index."
  array Uint32Array
  stride 1
  value ArtBrush
}
```

<sub>`api.ogt` line 26587</sub>

```ogt
column ArtProjection {
  doc "An ArtProjection field: the variant's index."
  array Uint32Array
  stride 1
  value ArtProjection
}
```

<sub>`api.ogt` line 26601</sub>
