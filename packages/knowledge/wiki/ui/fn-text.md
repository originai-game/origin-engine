<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-text"></a>

### `og.ui.text`

Text in a role (docs/ui.md §11, Foundations): the style's type step, font and weight for display, title, heading, body, label, caption or numeric (monospaced figures), in a tone, cut after maxLines with an ellipsis (the probe and a screen reader still read it whole). Display, title and heading are headings to a screen reader (a style's flourish frames them). It grows with the player's text size and wraps between CJK characters by kinsoku. Probe: the element `<id>`, kind label, its whole text.

```ts
og.ui.text(id: string, value: string, desc?: TextBlockDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text (a string-table message, og.i18n.t). |
| `desc` | `TextBlockDesc` | yes | Its role, tone and fit. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn text {
  doc "Text in a role (docs/ui.md §11, Foundations): the style's type step, font and weight for display, title, heading, body, label, caption or numeric (monospaced figures), in a tone, cut after maxLines with an ellipsis (the probe and a screen reader still read it whole). Display, title and heading are headings to a screen reader (a style's flourish frames them). It grows with the player's text size and wraps between CJK characters by kinsoku. Probe: the element `<id>`, kind label, its whole text."
  host false
  param id String "Its id."
  param value String "The text (a string-table message, og.i18n.t)."
  param desc TextBlockDesc "Its role, tone and fit."
  returns Void
}
```

<sub>`api.ogt` line 18947</sub>
