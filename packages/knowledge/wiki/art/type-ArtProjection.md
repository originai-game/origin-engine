<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

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

<sub>`api.ogt` line 26476</sub>
