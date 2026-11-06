<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-ImageInfo"></a>

### `ImageInfo` (struct)

What a ready uncooked image holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture: og.render.sprite, og.ui.image and material maps draw it like a cooked one. |
| `width` | `number` | `0` | — | — | — | Its width in texels. |
| `height` | `number` | `0` | — | — | — | Its height in texels. |

```ogt
struct ImageInfo {
  doc "What a ready uncooked image holds."
  field texture Texture "The texture: og.render.sprite, og.ui.image and material maps draw it like a cooked one."
  field width u32 "Its width in texels."
  field height u32 "Its height in texels."
}
```

<sub>`api.ogt` line 8411</sub>
