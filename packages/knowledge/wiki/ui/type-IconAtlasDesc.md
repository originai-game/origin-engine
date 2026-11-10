<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-IconAtlasDesc"></a>

### `IconAtlasDesc` (desc struct)

The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `font` | `Font \| null` | `null` | — | — | — | An icon set from og.load.font (an `.ogt` of `type icons` naming SVGs, such as the UI kit's iconoir set): its icons are distance-field glyphs, crisp at any size and tinted by their ink, named by the set; texture, columns, rows and names are then not used. |
| `texture` | `Texture \| null` | `null` | — | — | — | The atlas texture, from og.load.texture. |
| `columns` | `number` | `1` | ≥ 1 | — | — | Cells across. |
| `rows` | `number` | `1` | ≥ 1 | — | — | Cells down. |
| `names` | `readonly string[]` | `[]` | — | — | — | Each cell's name, in reading order: a document's `<og-icon>` draws a cell by its name. |

```ogt
struct IconAtlasDesc {
  doc "The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font)."
  desc true
  field font Option.Font "An icon set from og.load.font (an `.ogt` of `type icons` naming SVGs, such as the UI kit's iconoir set): its icons are distance-field glyphs, crisp at any size and tinted by their ink, named by the set; texture, columns, rows and names are then not used."
  field texture Option.Texture "The atlas texture, from og.load.texture."
  field columns u32 "Cells across." {
    default 1
    min 1
  }
  field rows u32 "Cells down." {
    default 1
    min 1
  }
  field names List.String "Each cell's name, in reading order: a document's `<og-icon>` draws a cell by its name."
}
```

<sub>Module `ui` line 407</sub>
