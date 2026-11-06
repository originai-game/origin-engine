<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Tone"></a>

### `Tone` (enum)

A status tone of badges, toasts and map markers.

A string: one of the variants below. The zero value is `"neutral"`.

| Variant | Doc |
|---|---|
| `"neutral"` | Neutral: control fill and muted ink. |
| `"success"` | Success. |
| `"warn"` | Warning (orange). |
| `"danger"` | Danger. |
| `"info"` | Information. |
| `"primary"` | The game's identity: the primary's soft wash and highlight ink. |
| `"inverse"` | Neutral emphasis: ink fill, inverse ink. |

```ogt
enum Tone {
  doc "A status tone of badges, toasts and map markers."
  variant neutral "Neutral: control fill and muted ink."
  variant success "Success."
  variant warn "Warning (orange)."
  variant danger "Danger."
  variant info "Information."
  variant primary "The game's identity: the primary's soft wash and highlight ink."
  variant inverse "Neutral emphasis: ink fill, inverse ink."
}
```

<sub>`api.ogt` line 16458</sub>
