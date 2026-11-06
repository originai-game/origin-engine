<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-DitherPattern"></a>

### `DitherPattern` (enum)

The threshold pattern Stylise.dither spreads the steps of posterize and palette with.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No dither: every pixel rounds to its nearest step or colour, in flat bands. |
| `"bayer2"` | Ordered dither by a 2 by 2 Bayer matrix: the coarsest checker, as on early handhelds. |
| `"bayer4"` | Ordered dither by a 4 by 4 Bayer matrix: the cross-hatched ramps of 8- and 16-bit art. |
| `"bayer8"` | Ordered dither by an 8 by 8 Bayer matrix: finer ramps with 64 levels between two colours. |
| `"blueNoise"` | Blue noise: an even grain without a visible grid (a 64 by 64 void-and-cluster tile the engine ships, repeated), fixed per pixel so a still frame does not crawl. |
| `"interleavedGradient"` | Interleaved gradient noise: computed per pixel without a texture; a finer diagonal structure than blueNoise up close. |

```ogt
enum DitherPattern {
  doc "The threshold pattern Stylise.dither spreads the steps of posterize and palette with."
  variant none "No dither: every pixel rounds to its nearest step or colour, in flat bands."
  variant bayer2 "Ordered dither by a 2 by 2 Bayer matrix: the coarsest checker, as on early handhelds."
  variant bayer4 "Ordered dither by a 4 by 4 Bayer matrix: the cross-hatched ramps of 8- and 16-bit art."
  variant bayer8 "Ordered dither by an 8 by 8 Bayer matrix: finer ramps with 64 levels between two colours."
  variant blueNoise "Blue noise: an even grain without a visible grid (a 64 by 64 void-and-cluster tile the engine ships, repeated), fixed per pixel so a still frame does not crawl."
  variant interleavedGradient "Interleaved gradient noise: computed per pixel without a texture; a finer diagonal structure than blueNoise up close."
}
```

<sub>`api.ogt` line 5176</sub>
