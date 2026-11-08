<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CornerShape"></a>

### `CornerShape` (enum)

How a rect's corners turn.

A string: one of the variants below. The zero value is `"round"`.

| Variant | Doc |
|---|---|
| `"round"` | Circular arcs. |
| `"squircle"` | Continuous corners: a superellipse of DrawShape.squircle. |
| `"bevel"` | A straight cut across, the radius along each edge. |
| `"notch"` | A concave quarter circle cut in (an inset notch). |

```ogt
enum CornerShape {
  doc "How a rect's corners turn."
  variant round "Circular arcs."
  variant squircle "Continuous corners: a superellipse of DrawShape.squircle."
  variant bevel "A straight cut across, the radius along each edge."
  variant notch "A concave quarter circle cut in (an inset notch)."
}
```

<sub>Module `ui` line 2952</sub>
