<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-LineMode"></a>

### `LineMode` (enum)

How a line set's points join, as three.js's three line objects.

A string: one of the variants below. The zero value is `"strip"`.

| Variant | Doc |
|---|---|
| `"strip"` | One polyline through every point, as three.js Line. |
| `"segments"` | Pairs of points, each its own segment, as three.js LineSegments. |
| `"loop"` | A polyline closed back to its first point, as three.js LineLoop. |

```ogt
enum LineMode {
  doc "How a line set's points join, as three.js's three line objects."
  variant strip "One polyline through every point, as three.js Line."
  variant segments "Pairs of points, each its own segment, as three.js LineSegments."
  variant loop "A polyline closed back to its first point, as three.js LineLoop."
}
```

<sub>Module `render` line 3747</sub>
