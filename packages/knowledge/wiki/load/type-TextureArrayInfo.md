<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-TextureArrayInfo"></a>

### `TextureArrayInfo` (struct)

What og.load.textureArray uploaded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `array` | `TextureArray` | `0` | — | — | — | The array. |
| `width` | `number` | `0` | — | — | — | Each layer's width in texels. |
| `height` | `number` | `0` | — | — | — | Each layer's height in texels. |
| `layers` | `number` | `0` | — | — | — | Its layers, in the order the file lists them. |
| `levels` | `number` | `0` | — | — | — | Mip levels of each layer. |
| `format` | `string` | `""` | — | — | — | The encoding on the GPU: `rgba8`, or the block format the device takes for its kind (`bc7`, `bc5`, `astc4x4`, `etc2`, `eacrg11`). |
| `bytes` | `number` | `0` | — | — | — | Bytes it takes on the GPU, every level of every layer (counted in the frame's texture bytes). |

```ogt
struct TextureArrayInfo {
  doc "What og.load.textureArray uploaded."
  field array TextureArray "The array."
  field width u32 "Each layer's width in texels."
  field height u32 "Each layer's height in texels."
  field layers u32 "Its layers, in the order the file lists them."
  field levels u32 "Mip levels of each layer."
  field format String "The encoding on the GPU: `rgba8`, or the block format the device takes for its kind (`bc7`, `bc5`, `astc4x4`, `etc2`, `eacrg11`)."
  field bytes f64 "Bytes it takes on the GPU, every level of every layer (counted in the frame's texture bytes)."
}
```

<sub>Module `load` line 116</sub>
