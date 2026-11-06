<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

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

<sub>`api.ogt` line 26530</sub>
