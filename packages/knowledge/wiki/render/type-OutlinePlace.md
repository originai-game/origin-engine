<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-OutlinePlace"></a>

### `OutlinePlace` (enum)

Where MaterialStyle's outlines lie against an object's silhouette (MaterialStyle.outlinePlace).

A string: one of the variants below. The zero value is `"inside"`.

| Variant | Doc |
|---|---|
| `"inside"` | On the object's own edge pixels, inside its silhouette: its size is unchanged. |
| `"outside"` | On the pixels round the object (a jump flood finds each pixel's nearest outlined silhouette), so the shape keeps its own pixels whole and a far character is not blacked out by its own line; round at the corners and anti-aliased at any width. |
| `"both"` | Centred on the silhouette: half the width inside, half outside. |

```ogt
enum OutlinePlace {
  doc "Where MaterialStyle's outlines lie against an object's silhouette (MaterialStyle.outlinePlace)."
  variant inside "On the object's own edge pixels, inside its silhouette: its size is unchanged."
  variant outside "On the pixels round the object (a jump flood finds each pixel's nearest outlined silhouette), so the shape keeps its own pixels whole and a far character is not blacked out by its own line; round at the corners and anti-aliased at any width."
  variant both "Centred on the silhouette: half the width inside, half outside."
}
```

<sub>Module `render` line 1424</sub>
