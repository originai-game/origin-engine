<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiActionHint"></a>

### `UiActionHint` (struct)

One entry of the action bar: an action the top layer accepts, its hint and glyph.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action. |
| `hint` | `string` | `""` | — | — | — | What it does here (the document's `actions` line, else its label). |
| `icon` | `string` | `""` | — | — | — | Its glyph's icon. |
| `text` | `string` | `""` | — | — | — | Its glyph's text. |
| `hold` | `boolean` | `false` | — | — | — | It is a hold. |

```ogt
struct UiActionHint {
  doc "One entry of the action bar: an action the top layer accepts, its hint and glyph."
  field action String "The action."
  field hint String "What it does here (the document's `actions` line, else its label)."
  field icon String "Its glyph's icon."
  field text String "Its glyph's text."
  field hold bool "It is a hold."
}
```

<sub>Module `ui` line 3404</sub>
