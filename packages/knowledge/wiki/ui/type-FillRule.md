<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FillRule"></a>

### `FillRule` (enum)

Which points a filled path (DrawShape.filled) covers where its contours overlap or cross.

A string: one of the variants below. The zero value is `"evenOdd"`.

| Variant | Doc |
|---|---|
| `"evenOdd"` | Where a ray from the point crosses the outline an odd number of times: every inner contour is a hole. |
| `"nonzero"` | Where the outline winds round the point at all: an inner contour is a hole only when it runs the other way (SVG's and Rive's default). |

```ogt
enum FillRule {
  doc "Which points a filled path (DrawShape.filled) covers where its contours overlap or cross."
  variant evenOdd "Where a ray from the point crosses the outline an odd number of times: every inner contour is a hole."
  variant nonzero "Where the outline winds round the point at all: an inner contour is a hole only when it runs the other way (SVG's and Rive's default)."
}
```

<sub>Module `ui` line 2773</sub>
