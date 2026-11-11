<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="type-ArtFilter"></a>

### `ArtFilter` (enum)

How the style's textures are sampled (ArtTexels.filter).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: each texture's own import settings. |
| `"nearest"` | Nearest: texels stay square and crisp up close, as low-resolution 3D and pixel art. |
| `"linear"` | Linear: texels blend smoothly. |

```ogt
enum ArtFilter {
  doc "How the style's textures are sampled (ArtTexels.filter)."
  variant any "No constraint: each texture's own import settings."
  variant nearest "Nearest: texels stay square and crisp up close, as low-resolution 3D and pixel art."
  variant linear "Linear: texels blend smoothly."
}
```

<sub>Module `art` line 21</sub>
