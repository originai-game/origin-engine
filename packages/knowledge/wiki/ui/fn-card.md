<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-card"></a>

### `og.ui.card`

A card over a body the game builds. Answers its response (clicked when interactive).

Its corners: a badge at the top end, a number plate at the top start, a cost disc at the bottom end; each mirrors right to left.

Parts: Card.frame[selected][<variant>], Card.media[placeholder], Card.number, Card.cost. Accessibility: a button (interactive) or group named by its eyebrow, title and subtitle, else its text; the number and the cost are read as text. Probe ids: <id>/Media, <id>/Body (with a cost and media, the disc at <id>/Media/Cost/Text; with a cost and no media, the game's body in <id>/Body/Content and the disc at <id>/Body/Cost/Text), <id>/Number/Text, <id>/Badge, <id>/Selected.

```ts
og.ui.card(id: string, desc: CardDesc, build: UiBuild): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CardDesc` | no | Variant, media, titles and state. |
| `build` | `UiBuild` | no | Builds its body. |

**Returns** `Response`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn card {
  doc "A card over a body the game builds. Answers its response (clicked when interactive)."
  doc "Its corners: a badge at the top end, a number plate at the top start, a cost disc at the bottom end; each mirrors right to left."
  doc "Parts: Card.frame[selected][<variant>], Card.media[placeholder], Card.number, Card.cost. Accessibility: a button (interactive) or group named by its eyebrow, title and subtitle, else its text; the number and the cost are read as text. Probe ids: <id>/Media, <id>/Body (with a cost and media, the disc at <id>/Media/Cost/Text; with a cost and no media, the game's body in <id>/Body/Content and the disc at <id>/Body/Cost/Text), <id>/Number/Text, <id>/Badge, <id>/Selected."
  host false
  param id String "Its id."
  param desc CardDesc "Variant, media, titles and state."
  param build UiBuild "Builds its body."
  returns Response
}
```

<sub>Module `ui` line 3036</sub>
