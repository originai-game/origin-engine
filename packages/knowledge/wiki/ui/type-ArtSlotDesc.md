<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ArtSlotDesc"></a>

### `ArtSlotDesc` (desc struct)

An art slot (docs/ui.md §13): where a generated or authored image goes, and what it shows until it is there.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The art; null shows the placeholder. |
| `kind` | `string` | `""` | — | — | — | Its kind (Appendix F: keyArt, portrait, itemIcon, cardArt, banner…): the placeholder's silhouette is the kind's. A portrait, bust, character, splash, avatar, banner or keyArt waits as head and shoulders on a wash of two hues, a body (a paperdoll's figure) as a whole standing figure with no box; any other kind as an icon of its name in the style's set, else `image`. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height; 0 is 1 (a square). |
| `fit` | `ArtFit` | `"cover"` | — | — | — | How the art fills it. |
| `focal` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | — | The point cropping keeps, fractions of the art. |
| `mask` | `ArtMask` | `"round"` | — | — | — | What it is cut to. |
| `radius` | `number \| null` | `null` | — | `cssPx` | — | The corner radius for mask round; null is the style's slot radius. |
| `loading` | `boolean` | `false` | — | — | — | The art is on its way: the placeholder's sheen runs (still under reduced motion). |
| `frame` | `ThemeColor \| null` | `null` | — | — | — | A frame around it in this ink (a rarity's, a team's); null is the style's line. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null follows the aspect. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it; empty is decoration. |

```ogt
struct ArtSlotDesc {
  doc "An art slot (docs/ui.md §13): where a generated or authored image goes, and what it shows until it is there."
  desc true
  field texture Option.Texture "The art; null shows the placeholder."
  field kind String "Its kind (Appendix F: keyArt, portrait, itemIcon, cardArt, banner…): the placeholder's silhouette is the kind's. A portrait, bust, character, splash, avatar, banner or keyArt waits as head and shoulders on a wash of two hues, a body (a paperdoll's figure) as a whole standing figure with no box; any other kind as an icon of its name in the style's set, else `image`."
  field aspect f32 "Width over height; 0 is 1 (a square)." {
    min 0
  }
  field fit ArtFit "How the art fills it."
  field focal Vector2 "The point cropping keeps, fractions of the art." {
    default 0.5 0.5
  }
  field mask ArtMask "What it is cut to."
  field radius Option.f32 "The corner radius for mask round; null is the style's slot radius." {
    unit cssPx
  }
  field loading bool "The art is on its way: the placeholder's sheen runs (still under reduced motion)."
  field frame Option.ThemeColor "A frame around it in this ink (a rarity's, a team's); null is the style's line."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height; null follows the aspect."
  field label String "What a screen reader calls it; empty is decoration."
}
```

<sub>`api.ogt` line 19025</sub>
