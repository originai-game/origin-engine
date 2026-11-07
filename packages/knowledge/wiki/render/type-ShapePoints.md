<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ShapePoints"></a>

### `ShapePoints` (struct)

A shape as points, what og.Shape.extractPoints returns and what shapeGeometry, extrudeGeometry and sweepGeometry read: the outline, then each hole. A repeated closing point is dropped, as three.js drops it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `readonly Vector2[]` | `[]` | — | — | — | The outline's points, in order. |
| `holes` | `readonly (readonly Vector2[])[]` | `[]` | — | — | — | Each hole's points, in order. |

```ogt
struct ShapePoints {
  doc "A shape as points, what og.Shape.extractPoints returns and what shapeGeometry, extrudeGeometry and sweepGeometry read: the outline, then each hole. A repeated closing point is dropped, as three.js drops it."
  field shape List.Vector2 "The outline's points, in order."
  field holes List.List.Vector2 "Each hole's points, in order."
}
```

<sub>Module `render` line 3166</sub>
