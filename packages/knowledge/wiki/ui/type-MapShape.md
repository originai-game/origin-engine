<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MapShape"></a>

### `MapShape` (enum)

A minimap's outline.

A string: one of the variants below. The zero value is `"circle"`.

| Variant | Doc |
|---|---|
| `"circle"` | Round (the default). |
| `"square"` | Square, cornered by the style. |

```ogt
enum MapShape {
  doc "A minimap's outline."
  variant circle "Round (the default)."
  variant square "Square, cornered by the style."
}
```

<sub>Module `ui` line 6189</sub>
