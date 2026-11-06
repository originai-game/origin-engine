<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

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
| `brush` | `ArtBrush` | `"diamond"` | — | — | — | Round a sprite: the brush the cook draws the line into its cells with: diamond (the zero value), circle or square (docs/art.md §2.3). |

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
  field brush ArtBrush "Round a sprite: the brush the cook draws the line into its cells with: diamond (the zero value), circle or square (docs/art.md §2.3)."
}
```

<sub>`api.ogt` line 26687</sub>
