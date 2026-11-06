<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ImageDesc"></a>

### `ImageDesc` (desc struct)

A texture region, or a solid rectangle without a texture.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The texture; null draws a solid rectangle of the tint. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies the texture; null is white. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height (0: none): with the height left to fit, the image is as tall as its laid-out width over this (a growing column keeps a picture's proportions); with a fixed height and the width left to fit, as wide as the height times it. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it (and the probe's text): a picture that means something (a portrait, a star earned) names it in the game's words; empty is decoration. |

```ogt
struct ImageDesc {
  doc "A texture region, or a solid rectangle without a texture."
  desc true
  field texture Option.Texture "The texture; null draws a solid rectangle of the tint."
  field uv Option.Array.f32.4 "The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture."
  field tint Option.Paint "Multiplies the texture; null is white."
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height; null fits."
  field aspect f32 "Width over height (0: none): with the height left to fit, the image is as tall as its laid-out width over this (a growing column keeps a picture's proportions); with a fixed height and the width left to fit, as wide as the height times it." {
    min 0
  }
  field label String "What a screen reader calls it (and the probe's text): a picture that means something (a portrait, a star earned) names it in the game's words; empty is decoration."
}
```

<sub>`api.ogt` line 15953</sub>
