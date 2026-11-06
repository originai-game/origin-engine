<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-HalftoneMode"></a>

### `HalftoneMode` (enum)

How Stylise.halftone inks the frame.

A string: one of the variants below. The zero value is `"ink"`.

| Variant | Doc |
|---|---|
| `"ink"` | One ink (halftoneColor) on paper: dots larger where the image is darker. |
| `"cmyk"` | Four-colour process print: the colour split into cyan, magenta, yellow and black (grey component replacement), each a dot screen at its classic angle (black at halftoneAngle, cyan 30 degrees before it, magenta 30 after, yellow 45 before: 15, 75, 0 and 45 at the default angle), the inks multiplied on white paper, as comics were printed. |

```ogt
enum HalftoneMode {
  doc "How Stylise.halftone inks the frame."
  variant ink "One ink (halftoneColor) on paper: dots larger where the image is darker."
  variant cmyk "Four-colour process print: the colour split into cyan, magenta, yellow and black (grey component replacement), each a dot screen at its classic angle (black at halftoneAngle, cyan 30 degrees before it, magenta 30 after, yellow 45 before: 15, 75, 0 and 45 at the default angle), the inks multiplied on white paper, as comics were printed."
}
```

<sub>`api.ogt` line 5229</sub>
