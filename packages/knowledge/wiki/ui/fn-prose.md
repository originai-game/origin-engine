<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-prose"></a>

### `og.ui.prose`

A passage of rich text (og.ui.richText's markup: `[b]`, `[color=…]`, `[icon=…]`, `[link=…]`, `[wave]`…), split into paragraphs at blank lines, at a readable measure, each a text element `<id>/P#i` whose links are focusable children (`<id>/P#i/<link>`). Answers the link clicked this tick (keys and pads press links too). Its region reads as one group to a screen reader.

```ts
og.ui.prose(id: string, markup: string, desc?: ProseDesc): RichTextResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `markup` | `string` | no | The markup; a blank line starts a paragraph. |
| `desc` | `ProseDesc` | yes | Its role, tone and measure. |

**Returns** `RichTextResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn prose {
  doc "A passage of rich text (og.ui.richText's markup: `[b]`, `[color=…]`, `[icon=…]`, `[link=…]`, `[wave]`…), split into paragraphs at blank lines, at a readable measure, each a text element `<id>/P#i` whose links are focusable children (`<id>/P#i/<link>`). Answers the link clicked this tick (keys and pads press links too). Its region reads as one group to a screen reader."
  host false
  param id String "Its id."
  param markup String "The markup; a blank line starts a paragraph."
  param desc ProseDesc "Its role, tone and measure."
  returns RichTextResult
}
```

<sub>Module `ui` line 3111</sub>
