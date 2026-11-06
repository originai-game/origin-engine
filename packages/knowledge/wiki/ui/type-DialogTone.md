<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DialogTone"></a>

### `DialogTone` (enum)

A dialog's tone, shown by a mark as well as its colour.

A string: one of the variants below. The zero value is `"neutral"`.

| Variant | Doc |
|---|---|
| `"neutral"` | No mark (the default). |
| `"info"` | Information: an i. |
| `"success"` | Success: a tick. |
| `"warn"` | A warning: an exclamation. |
| `"danger"` | Danger: a cross. |

```ogt
enum DialogTone {
  doc "A dialog's tone, shown by a mark as well as its colour."
  variant neutral "No mark (the default)."
  variant info "Information: an i."
  variant success "Success: a tick."
  variant warn "A warning: an exclamation."
  variant danger "Danger: a cross."
}
```

<sub>`api.ogt` line 17826</sub>
