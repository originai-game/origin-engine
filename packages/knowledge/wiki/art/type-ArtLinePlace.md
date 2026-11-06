<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="type-ArtLinePlace"></a>

### `ArtLinePlace` (enum)

Where the style's line lies against a shape (ArtLine.place).

A string: one of the variants below. The zero value is `"outside"`.

| Variant | Doc |
|---|---|
| `"outside"` | Round the shape: the line grows the silhouette by its width. |
| `"inside"` | On the shape's own edge pixels: its size is unchanged. |
| `"both"` | Centred on the edge: half outside, half inside. |

```ogt
enum ArtLinePlace {
  doc "Where the style's line lies against a shape (ArtLine.place)."
  variant outside "Round the shape: the line grows the silhouette by its width."
  variant inside "On the shape's own edge pixels: its size is unchanged."
  variant both "Centred on the edge: half outside, half inside."
}
```

<sub>`api.ogt` line 26566</sub>
