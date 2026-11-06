<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ArtMask"></a>

### `ArtMask` (enum)

What art is cut to.

A string: one of the variants below. The zero value is `"round"`.

| Variant | Doc |
|---|---|
| `"round"` | The slot's rectangle with its radius (the default: the style's slot radius). |
| `"circle"` | A circle (a portrait, an avatar). |
| `"squircle"` | A continuous-corner square (an app-style icon). |
| `"none"` | Square corners. |

```ogt
enum ArtMask {
  doc "What art is cut to."
  variant round "The slot's rectangle with its radius (the default: the style's slot radius)."
  variant circle "A circle (a portrait, an avatar)."
  variant squircle "A continuous-corner square (an app-style icon)."
  variant none "Square corners."
}
```

<sub>`api.ogt` line 18927</sub>
