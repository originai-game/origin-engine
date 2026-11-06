<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GraphNodeShape"></a>

### `GraphNodeShape` (enum)

A graph node's outline.

A string: one of the variants below. The zero value is `"circle"`.

| Variant | Doc |
|---|---|
| `"circle"` | A circle (the default). |
| `"square"` | A rounded square. |
| `"hex"` | A hexagon. |
| `"diamond"` | A diamond. |
| `"triangle"` | A triangle, point up. |

```ogt
enum GraphNodeShape {
  doc "A graph node's outline."
  variant circle "A circle (the default)."
  variant square "A rounded square."
  variant hex "A hexagon."
  variant diamond "A diamond."
  variant triangle "A triangle, point up."
}
```

<sub>Module `ui` line 2786</sub>
