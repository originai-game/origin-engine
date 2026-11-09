<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ColorVision"></a>

### `ColorVision` (enum)

A colour vision deficiency (UiDesc.colorVision; CSS @media (-og-color-vision: <name>)).

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Typical colour vision: no filter. |
| `"protanopia"` | No red-sensitive cones: reds dark, red and green confused. |
| `"deuteranopia"` | No green-sensitive cones: red and green confused. |
| `"tritanopia"` | No blue-sensitive cones: blue and green, yellow and violet confused. |

```ogt
enum ColorVision {
  doc "A colour vision deficiency (UiDesc.colorVision; CSS @media (-og-color-vision: <name>))."
  variant none "Typical colour vision: no filter."
  variant protanopia "No red-sensitive cones: reds dark, red and green confused."
  variant deuteranopia "No green-sensitive cones: red and green confused."
  variant tritanopia "No blue-sensitive cones: blue and green, yellow and violet confused."
}
```

<sub>Module `ui` line 359</sub>
