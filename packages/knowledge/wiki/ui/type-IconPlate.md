<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-IconPlate"></a>

### `IconPlate` (enum)

What an icon sits on.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Nothing (the default). |
| `"circle"` | A disc of its tone's soft fill. |
| `"square"` | A rounded square in the style's control radius. |

```ogt
enum IconPlate {
  doc "What an icon sits on."
  variant none "Nothing (the default)."
  variant circle "A disc of its tone's soft fill."
  variant square "A rounded square in the style's control radius."
}
```

<sub>Module `ui` line 3215</sub>
