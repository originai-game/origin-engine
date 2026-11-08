<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Contrast"></a>

### `Contrast` (enum)

The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour).

A string: one of the variants below. The zero value is `"standard"`.

| Variant | Doc |
|---|---|
| `"standard"` | The style's palette as it is. |
| `"high"` | Stronger inks, fills and edges. |

```ogt
enum Contrast {
  doc "The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour)."
  variant standard "The style's palette as it is."
  variant high "Stronger inks, fills and edges."
}
```

<sub>Module `ui` line 817</sub>
