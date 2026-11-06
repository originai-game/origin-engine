<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="type-ArtBrush"></a>

### `ArtBrush` (enum)

The brush a sprite's baked line is drawn with (ArtLine.brush), as Pixelorama's outline effect draws it: which texels within ArtLine.width of the shape's edge take the line.

A string: one of the variants below. The zero value is `"diamond"`.

| Variant | Doc |
|---|---|
| `"diamond"` | The texels within width steps along the grid (\|dx\| + \|dy\| at most width): a 1-pixel line touches the four sides and leaves the corners, the classic pixel-art outline. |
| `"circle"` | The texels within a disc of radius width + 0.5: rounder corners on wide lines. |
| `"square"` | Every texel within width across and down: the line fills the corners, so a 1-pixel line touches all eight neighbours. |

```ogt
enum ArtBrush {
  doc "The brush a sprite's baked line is drawn with (ArtLine.brush), as Pixelorama's outline effect draws it: which texels within ArtLine.width of the shape's edge take the line."
  variant diamond "The texels within width steps along the grid (|dx| + |dy| at most width): a 1-pixel line touches the four sides and leaves the corners, the classic pixel-art outline."
  variant circle "The texels within a disc of radius width + 0.5: rounder corners on wide lines."
  variant square "Every texel within width across and down: the line fills the corners, so a 1-pixel line touches all eight neighbours."
}
```

<sub>`api.ogt` line 26551</sub>
