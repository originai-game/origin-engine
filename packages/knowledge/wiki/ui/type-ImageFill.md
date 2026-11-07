<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ImageFill"></a>

### `ImageFill` (desc struct)

A texture region with a fit.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture (og.load.texture). |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The region's size in image pixels (none, nineSlice and pattern read it); zero is the texture's. |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies it; null is white. |
| `fit` | `ImageFit` | `"fill"` | — | — | — | How it fills the box. |
| `focal` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | `ratio` | — | Where cover crops and contain or none place it, fractions of the free space. |
| `slice` | `[number, number, number, number]` | `[0, 0, 0, 0]` | — | — | — | A nine-slice's border in image pixels: left, top, right, bottom. |
| `border` | `[number, number, number, number] \| null` | `null` | — | — | — | A nine-slice's border as drawn, CSS px; null is slice. |
| `edges` | `SliceEdges` | `"stretch"` | — | — | — | A nine-slice's edges and centre. |
| `hollow` | `boolean` | `false` | — | — | — | A nine-slice leaves its centre empty. |
| `tile` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | A pattern's tile, CSS px; zero is size. |
| `patternRotation` | `number` | `0` | — | `rad` | — | A pattern's turn. |
| `offset` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | A pattern's offset. |
| `scroll` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | A pattern's scroll, CSS px a second of the UI clock. |

```ogt
struct ImageFill {
  doc "A texture region with a fit."
  desc true
  field texture Texture "The texture (og.load.texture)."
  field uv Option.Array.f32.4 "The region, [u0, v0, u1, v1] with v down; null is the whole texture."
  field size Vector2 "The region's size in image pixels (none, nineSlice and pattern read it); zero is the texture's."
  field tint Option.Paint "Multiplies it; null is white."
  field fit ImageFit "How it fills the box."
  field focal Vector2 "Where cover crops and contain or none place it, fractions of the free space." {
    default 0.5 0.5
    unit ratio
  }
  field slice Array.f32.4 "A nine-slice's border in image pixels: left, top, right, bottom."
  field border Option.Array.f32.4 "A nine-slice's border as drawn, CSS px; null is slice."
  field edges SliceEdges "A nine-slice's edges and centre."
  field hollow bool "A nine-slice leaves its centre empty."
  field tile Vector2 "A pattern's tile, CSS px; zero is size." {
    unit cssPx
  }
  field patternRotation f32 "A pattern's turn." {
    unit rad
  }
  field offset Vector2 "A pattern's offset." {
    unit cssPx
  }
  field scroll Vector2 "A pattern's scroll, CSS px a second of the UI clock." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 7197</sub>
