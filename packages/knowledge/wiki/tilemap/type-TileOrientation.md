<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="type-TileOrientation"></a>

### `TileOrientation` (enum)

How a map's cells are laid out (Tiled's orientations).

A string: one of the variants below. The zero value is `"orthogonal"`.

| Variant | Doc |
|---|---|
| `"orthogonal"` | Rectangles in rows and columns. |
| `"isometric"` | Diamonds on a turned grid: x runs down to the right, y down to the left, cell 0 0 at the top. |
| `"staggered"` | Diamonds in rows half a tile apart, every odd row shifted right half a tile (Tiled's staggered, odd rows). |
| `"hex"` | Pointy-top hexagons with corners at a quarter of the tile, rows 3/4 of a tile apart, every odd row shifted right half a tile (Tiled's hexagonal, stagger axis y, odd, side half the tile's height). |
| `"hexFlat"` | Flat-top hexagons, columns 3/4 of a tile apart, every odd column shifted down half a tile (stagger axis x). |

```ogt
enum TileOrientation {
  doc "How a map's cells are laid out (Tiled's orientations)."
  variant orthogonal "Rectangles in rows and columns."
  variant isometric "Diamonds on a turned grid: x runs down to the right, y down to the left, cell 0 0 at the top."
  variant staggered "Diamonds in rows half a tile apart, every odd row shifted right half a tile (Tiled's staggered, odd rows)."
  variant hex "Pointy-top hexagons with corners at a quarter of the tile, rows 3/4 of a tile apart, every odd row shifted right half a tile (Tiled's hexagonal, stagger axis y, odd, side half the tile's height)."
  variant hexFlat "Flat-top hexagons, columns 3/4 of a tile apart, every odd column shifted down half a tile (stagger axis x)."
}
```

<sub>Module `tilemap` line 47</sub>
