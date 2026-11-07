<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Fill"></a>

### `Fill` (union)

One layer of an element's fill, or a stroke's paint.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "solid"` | [`SolidFill`](../ui/type-SolidFill.md#type-SolidFill) | A colour. |
| `kind: "gradient"` | [`Gradient`](../ui/type-Gradient.md#type-Gradient) | A gradient. |
| `kind: "image"` | [`ImageFill`](../ui/type-ImageFill.md#type-ImageFill) | An image with a fit, a nine-slice or a pattern. |
| `kind: "vector"` | [`VectorFill`](../ui/type-VectorFill.md#type-VectorFill) | A cooked vector (as a stroke's paint, its tint). |

```ogt
union Fill {
  doc "One layer of an element's fill, or a stroke's paint."
  tag kind
  case solid SolidFill "A colour."
  case gradient Gradient "A gradient."
  case image ImageFill "An image with a fit, a nine-slice or a pattern."
  case vector VectorFill "A cooked vector (as a stroke's paint, its tint)."
}
```

<sub>Module `ui` line 7234</sub>
