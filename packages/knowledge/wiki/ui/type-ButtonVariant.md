<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ButtonVariant"></a>

### `ButtonVariant` (enum)

Button roles: how the style draws each.

A string: one of the variants below. The zero value is `"secondary"`.

| Variant | Doc |
|---|---|
| `"secondary"` | A neutral fill (the default). |
| `"primary"` | The primary action: one per screen. |
| `"quiet"` | Text until hovered. |
| `"danger"` | A destructive action: danger ink on the danger tint. |
| `"ghost"` | Clear inside a strong outline until hovered: a second action beside a primary that still reads as a button. |

```ogt
enum ButtonVariant {
  doc "Button roles: how the style draws each."
  variant secondary "A neutral fill (the default)."
  variant primary "The primary action: one per screen."
  variant quiet "Text until hovered."
  variant danger "A destructive action: danger ink on the danger tint."
  variant ghost "Clear inside a strong outline until hovered: a second action beside a primary that still reads as a button."
}
```

<sub>`api.ogt` line 15684</sub>
