<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-ImageKind"></a>

### `ImageKind` (enum)

How an uncooked image's texels are read, as a cooked texture's `kind`.

A string: one of the variants below. The zero value is `"color"`.

| Variant | Doc |
|---|---|
| `"color"` | sRGB colour (albedo, UI art), sampled linear. |
| `"linear"` | Data stored linearly (masks, roughness). |
| `"pixel"` | sRGB colour sampled nearest (pixel art). |

```ogt
enum ImageKind {
  doc "How an uncooked image's texels are read, as a cooked texture's `kind`."
  variant color "sRGB colour (albedo, UI art), sampled linear."
  variant linear "Data stored linearly (masks, roughness)."
  variant pixel "sRGB colour sampled nearest (pixel art)."
}
```

<sub>`api.ogt` line 8342</sub>
