<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="type-ArtMips"></a>

### `ArtMips` (enum)

The mip maps of the style's textures (ArtTexels.mips).

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | No constraint: each texture's own import settings. |
| `"full"` | A full mip chain: no shimmer in the distance. |
| `"none"` | The base level only: texels stay as crisp far away as near, as on early 3D hardware. |

```ogt
enum ArtMips {
  doc "The mip maps of the style's textures (ArtTexels.mips)."
  variant any "No constraint: each texture's own import settings."
  variant full "A full mip chain: no shimmer in the distance."
  variant none "The base level only: texels stay as crisp far away as near, as on early 3D hardware."
}
```

<sub>`api.ogt` line 26552</sub>
