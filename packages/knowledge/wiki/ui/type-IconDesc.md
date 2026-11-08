<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-IconDesc"></a>

### `IconDesc` (desc struct)

An icon's size, ink and name.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its size; 0 is 24. |
| `tint` | `Paint \| null` | `null` | — | — | — | Its ink; null is ink. |
| `secondary` | `string \| null` | `null` | — | — | — | A second atlas icon, overlaid in the same em square: author the two silhouettes as separate named layers. Null draws one layer. |
| `secondaryTint` | `Paint \| null` | `null` | — | — | — | The second layer's ink; null is accent. |
| `label` | `string \| null` | `null` | — | — | — | What a screen reader calls it (and the probe's text): null is the icon's name in the atlas (an English word: `star`); empty is decoration, named by nothing, so a card or button it sits on is named by its other text. A localised game names a meaningful icon from its string table. |

```ogt
struct IconDesc {
  doc "An icon's size, ink and name."
  desc true
  field size f32 "Its size; 0 is 24." {
    unit cssPx
    min 0
  }
  field tint Option.Paint "Its ink; null is ink."
  field secondary Option.String "A second atlas icon, overlaid in the same em square: author the two silhouettes as separate named layers. Null draws one layer."
  field secondaryTint Option.Paint "The second layer's ink; null is accent."
  field label Option.String "What a screen reader calls it (and the probe's text): null is the icon's name in the atlas (an English word: `star`); empty is decoration, named by nothing, so a card or button it sits on is named by its other text. A localised game names a meaningful icon from its string table."
}
```

<sub>Module `ui` line 1021</sub>
