<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CardMedia"></a>

### `CardMedia` (desc struct)

A card's media slot: art the game fills, or a placeholder drawn from the style when it has none.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The art (og.load.texture), covering the slot; 0 draws the placeholder. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height; 0 is 16:9. |
| `focal` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | — | The art's point kept in view when cover crops it, fractions of the free space. |
| `icon` | `string` | `""` | — | — | — | The placeholder's icon (og.ui.icon's name); empty is `image`. |
| `label` | `string` | `""` | — | — | — | What the art shows, for a screen reader; empty is decorative. |

```ogt
struct CardMedia {
  doc "A card's media slot: art the game fills, or a placeholder drawn from the style when it has none."
  desc true
  field texture Texture "The art (og.load.texture), covering the slot; 0 draws the placeholder."
  field aspect f32 "Width over height; 0 is 16:9." {
    min 0
  }
  field focal Vector2 "The art's point kept in view when cover crops it, fractions of the free space." {
    default 0.5 0.5
  }
  field icon String "The placeholder's icon (og.ui.icon's name); empty is `image`."
  field label String "What the art shows, for a screen reader; empty is decorative."
}
```

<sub>Module `ui` line 1850</sub>
