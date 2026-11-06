<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CardDesc"></a>

### `CardDesc` (desc struct)

A card: media, an eyebrow, a title, a subtitle and a corner badge over a body the game builds.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `variant` | `CardVariant` | `"raised"` | — | — | — | How it stands off what it sits on. |
| `media` | `CardMedia \| null` | `null` | — | — | — | The media slot over the body; null is none. |
| `eyebrow` | `string` | `""` | — | — | — | A small line over the title (a category, a rarity); empty is none. |
| `title` | `string` | `""` | — | — | — | The title; empty is none. |
| `subtitle` | `string` | `""` | — | — | — | A line under the title; empty is none. |
| `badge` | `string` | `""` | — | — | — | A badge in the top end corner; empty is none. |
| `badgeTone` | `Tone` | `"neutral"` | — | — | — | The badge's tone. |
| `number` | `string` | `""` | — | — | — | A number on a small plate at the top start corner (its place in a hand, its slot), over the media's corner when there is media, else above the body; small, so the art stays the card's one dominant element. Empty is none. |
| `cost` | `string` | `""` | — | — | — | A numeral in a disc (a cost, a level): with media on the art's bottom end corner, opposite the number plate, so the body's text keeps the card's whole width; with none at the body's bottom end, beside it, so the game's text never runs under it. Small, so the art stays the card's one dominant element. Empty is none. |
| `interactive` | `boolean` | `false` | — | — | — | The whole card is a button: it lifts under the pointer and the focus, sinks when pressed, and answers clicked. |
| `selected` | `boolean` | `false` | — | — | — | Ringed in the focus ink (Card.frame[selected]). |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits. |
| `padding` | `number \| null` | `null` | — | `cssPx` | — | Space around the body; null is space.lg. |

```ogt
struct CardDesc {
  doc "A card: media, an eyebrow, a title, a subtitle and a corner badge over a body the game builds."
  desc true
  field variant CardVariant "How it stands off what it sits on."
  field media Option.CardMedia "The media slot over the body; null is none."
  field eyebrow String "A small line over the title (a category, a rarity); empty is none."
  field title String "The title; empty is none."
  field subtitle String "A line under the title; empty is none."
  field badge String "A badge in the top end corner; empty is none."
  field badgeTone Tone "The badge's tone."
  field number String "A number on a small plate at the top start corner (its place in a hand, its slot), over the media's corner when there is media, else above the body; small, so the art stays the card's one dominant element. Empty is none."
  field cost String "A numeral in a disc (a cost, a level): with media on the art's bottom end corner, opposite the number plate, so the body's text keeps the card's whole width; with none at the body's bottom end, beside it, so the game's text never runs under it. Small, so the art stays the card's one dominant element. Empty is none."
  field interactive bool "The whole card is a button: it lifts under the pointer and the focus, sinks when pressed, and answers clicked."
  field selected bool "Ringed in the focus ink (Card.frame[selected])."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height; null fits."
  field padding Option.f32 "Space around the body; null is space.lg." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 17729</sub>
