<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-HalftoneRegion"></a>

### `HalftoneRegion` (enum)

Which tones Stylise.halftone prints on.

A string: one of the variants below. The zero value is `"all"`.

| Variant | Doc |
|---|---|
| `"all"` | Every tone. |
| `"shade"` | The shade only: Ben-Day dots in the shadows while lit tones keep their colour (Hi-Fi Rush, Spider-Verse). |
| `"highlight"` | The highlights only. |

```ogt
enum HalftoneRegion {
  doc "Which tones Stylise.halftone prints on."
  variant all "Every tone."
  variant shade "The shade only: Ben-Day dots in the shadows while lit tones keep their colour (Hi-Fi Rush, Spider-Verse)."
  variant highlight "The highlights only."
}
```

<sub>Module `render` line 1633</sub>
