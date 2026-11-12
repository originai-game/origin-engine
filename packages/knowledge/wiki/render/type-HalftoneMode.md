<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-HalftoneMode"></a>

### `HalftoneMode` (enum)

How Stylise.halftone inks the frame.

See `looks §9`.

A string: one of the variants below. The zero value is `"ink"`.

| Variant | Doc |
|---|---|
| `"ink"` | One ink (halftoneColor) on paper: dots larger where the image is darker. |
| `"cmyk"` | Four-colour process print, as comics were printed: the colour split into cyan, magenta, yellow and black (grey component replacement), each a dot screen at its classic angle from halftoneAngle, the inks multiplied on white paper. |

```ogt
enum HalftoneMode {
  doc "How Stylise.halftone inks the frame."
  variant ink "One ink (halftoneColor) on paper: dots larger where the image is darker."
  variant cmyk "Four-colour process print, as comics were printed: the colour split into cyan, magenta, yellow and black (grey component replacement), each a dot screen at its classic angle from halftoneAngle, the inks multiplied on white paper."
  guide "looks §9"
}
```

<sub>Module `render` line 1587</sub>
