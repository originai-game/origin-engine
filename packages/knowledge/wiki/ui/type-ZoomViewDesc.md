<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ZoomViewDesc"></a>

### `ZoomViewDesc` (desc struct)

A zoomable picture (a map, an artwork): what it shows and its zoom limits.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The picture; null draws a solid rectangle of the tint. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The picture's region, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies the texture; null is white. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height (0: none), as ImageDesc.aspect. |
| `minZoom` | `number` | `0` | ≥ 0 | — | — | The least zoom; 0 is 1 (the whole picture). |
| `maxZoom` | `number` | `0` | ≥ 0 | — | — | The most zoom; 0 is 4. |

```ogt
struct ZoomViewDesc {
  doc "A zoomable picture (a map, an artwork): what it shows and its zoom limits."
  desc true
  field texture Option.Texture "The picture; null draws a solid rectangle of the tint."
  field uv Option.Array.f32.4 "The picture's region, [u0, v0, u1, v1] with v down; null is the whole texture."
  field tint Option.Paint "Multiplies the texture; null is white."
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height; null fits."
  field aspect f32 "Width over height (0: none), as ImageDesc.aspect." {
    min 0
  }
  field minZoom f32 "The least zoom; 0 is 1 (the whole picture)." {
    min 0
  }
  field maxZoom f32 "The most zoom; 0 is 4." {
    min 0
  }
}
```

<sub>`api.ogt` line 15967</sub>
