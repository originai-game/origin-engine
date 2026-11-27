<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="record-ArtTexels"></a>

### `og.ArtTexels` (resource)

The style's texel density on 3D surfaces: one density for the whole world, so no model is sharper or blurrier than its neighbours, the largest texture, and how textures are filtered and mipped. Every field is no constraint at its zero value.

Token `og.ArtTexels`, registered under the schema name `og.ArtTexels`; its value type is `ArtTexels`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `density` | `number` | `0` | 0 … 16384 | — | — | Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint. |
| `maxSize` | `number` | `0` | 0 … 16384 | — | — | The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit. |
| `filter` | `ArtFilter` | `"any"` | — | — | — | How the style's textures are sampled: any (no constraint), nearest or linear. |
| `mips` | `ArtMips` | `"any"` | — | — | — | Their mip maps: any (no constraint), full or none. |

```ogt
resource ArtTexels {
  doc "The style's texel density on 3D surfaces: one density for the whole world, so no model is sharper or blurrier than its neighbours, the largest texture, and how textures are filtered and mipped. Every field is no constraint at its zero value."
  field density f32 "Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint." {
    min 0
    max 16384
  }
  field maxSize u32 "The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit." {
    min 0
    max 16384
  }
  field filter ArtFilter "How the style's textures are sampled: any (no constraint), nearest or linear."
  field mips ArtMips "Their mip maps: any (no constraint), full or none."
}
```

<sub>Module `art` line 141</sub>
