<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ShapePreset"></a>

### `ShapePreset` (enum)

A decorative shape.

A string: one of the variants below. The zero value is `"rect"`.

| Variant | Doc |
|---|---|
| `"rect"` | A rectangle with the radius given (the default). |
| `"capsule"` | A rectangle rounded to half its height. |
| `"ellipse"` | An ellipse (a circle when square). |
| `"polygon"` | A regular polygon of `sides`. |
| `"star"` | A star of `sides` points. |
| `"ring"` | A ring of `thickness`. |
| `"diamond"` | A square on its point. |
| `"line"` | A line across its box (vertical when taller than wide), `thickness` thick. |

```ogt
enum ShapePreset {
  doc "A decorative shape."
  variant rect "A rectangle with the radius given (the default)."
  variant capsule "A rectangle rounded to half its height."
  variant ellipse "An ellipse (a circle when square)."
  variant polygon "A regular polygon of `sides`."
  variant star "A star of `sides` points."
  variant ring "A ring of `thickness`."
  variant diamond "A square on its point."
  variant line "A line across its box (vertical when taller than wide), `thickness` thick."
}
```

<sub>Module `ui` line 3199</sub>
