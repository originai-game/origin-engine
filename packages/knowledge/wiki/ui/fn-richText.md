<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-richText"></a>

### `og.ui.richText`

A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the style), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning.

```ts
og.ui.richText(id: string, markup: string, desc?: RichTextDesc): RichTextResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `markup` | `string` | no | The markup. |
| `desc` | `RichTextDesc` | yes | Defaults for its spans: type step, ink, alignment, wrapping, width and font. |

**Returns** `RichTextResult`.

Binding symbol `og_ui_rich_text` · publish builds: kept

```ogt
fn richText {
  doc "A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the style), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning."
  param id String "Its id."
  param markup String "The markup."
  param desc RichTextDesc "Defaults for its spans: type step, ink, alignment, wrapping, width and font."
  returns RichTextResult
}
```

<sub>`api.ogt` line 16338</sub>
