<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GradientType"></a>

### `GradientType` (enum)

How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right).

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | From `from` to `to`. |
| `"radial"` | Centred on `from`, `to` its x and y radii. |
| `"conic"` | Around `from`, starting at the angle to.x (radians clockwise from 3 o'clock). |
| `"diamond"` | Centred on `from`, `to` its x and y radii, by \|x\|/rx + \|y\|/ry. |

```ogt
enum GradientType {
  doc "How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right)."
  variant linear "From `from` to `to`."
  variant radial "Centred on `from`, `to` its x and y radii."
  variant conic "Around `from`, starting at the angle to.x (radians clockwise from 3 o'clock)."
  variant diamond "Centred on `from`, `to` its x and y radii, by |x|/rx + |y|/ry."
}
```

<sub>Module `ui` line 2566</sub>
