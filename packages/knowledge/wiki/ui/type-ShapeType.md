<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ShapeType"></a>

### `ShapeType` (enum)

An element's outline (DrawShape.kind).

A string: one of the variants below. The zero value is `"rect"`.

| Variant | Doc |
|---|---|
| `"rect"` | Its rectangle, with DrawShape.radii (or its radius) and corner. |
| `"ellipse"` | The ellipse inscribed in its rectangle (a circle when square; a capsule is a rect with radii of half its height). |
| `"polygon"` | A regular polygon of sides vertices on the circle in its rectangle, the first at the top; inner under 1 makes a star. |
| `"arc"` | A ring segment on the circle in its rectangle: start, sweep and thickness (a ring at a full turn), with caps. |
| `"path"` | A line through points (CSS px from its top left), Béziers with curves; its fills draw the line. |

```ogt
enum ShapeType {
  doc "An element's outline (DrawShape.kind)."
  variant rect "Its rectangle, with DrawShape.radii (or its radius) and corner."
  variant ellipse "The ellipse inscribed in its rectangle (a circle when square; a capsule is a rect with radii of half its height)."
  variant polygon "A regular polygon of sides vertices on the circle in its rectangle, the first at the top; inner under 1 makes a star."
  variant arc "A ring segment on the circle in its rectangle: start, sweep and thickness (a ring at a full turn), with caps."
  variant path "A line through points (CSS px from its top left), Béziers with curves; its fills draw the line."
}
```

<sub>Module `ui` line 3101</sub>
