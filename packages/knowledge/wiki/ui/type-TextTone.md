<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextTone"></a>

### `TextTone` (enum)

A text's ink by meaning.

A string: one of the variants below. The zero value is `"normal"`.

| Variant | Doc |
|---|---|
| `"normal"` | The style's ink (the default). |
| `"muted"` | Secondary: inkMuted (4.5:1 on its panel in every built-in style). |
| `"faint"` | Tertiary, never for what must be read: inkFaint. |
| `"primary"` | The style's primary as ink (a link, a highlight). |
| `"success"` | Success ink. |
| `"warn"` | Warning ink. |
| `"danger"` | Danger ink. |
| `"info"` | Information ink. |
| `"inverse"` | inkInverse, for text on an ink-coloured fill. |

```ogt
enum TextTone {
  doc "A text's ink by meaning."
  variant normal "The style's ink (the default)."
  variant muted "Secondary: inkMuted (4.5:1 on its panel in every built-in style)."
  variant faint "Tertiary, never for what must be read: inkFaint."
  variant primary "The style's primary as ink (a link, a highlight)."
  variant success "Success ink."
  variant warn "Warning ink."
  variant danger "Danger ink."
  variant info "Information ink."
  variant inverse "inkInverse, for text on an ink-coloured fill."
}
```

<sub>Module `ui` line 3037</sub>
