<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `shapes` module

Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, bezierCurveTo, arc, absarc, ellipse, closePath, ...) and flattened to points by getPoints. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine.

They are TypeScript values, not world state: the geometry made from them is what the world keeps. Points come from basic arithmetic (no Math.sin), so they are the same on every host. A `type model` asset writes the same calls as text.

See `models §1`.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module shapes {
  doc "Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, bezierCurveTo, arc, absarc, ellipse, closePath, ...) and flattened to points by getPoints. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine."
  doc "They are TypeScript values, not world state: the geometry made from them is what the world keeps. Points come from basic arithmetic (no Math.sin), so they are the same on every host. A `type model` asset writes the same calls as text."
  # declarations follow, each on its own page
```

<sub>Module `shapes` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`PathList`](shapes/type-PathList.md) | interface |  | A shape's holes, as three.js Shape.holes: an array; push a path to add a hole. |
| [`Path2`](shapes/type-Path2.md) | interface |  | An outline of curves, as three.js Path: each call adds a curve from the current point (moveTo only moves it). |
| [`Shape`](shapes/type-Shape.md) | interface |  | An outline with holes, as three.js Shape. |
| [`Path2Class`](shapes/type-Path2Class.md) | interface |  | og.Path2: `new og.Path2(points?)`. |
| [`ShapeClass`](shapes/type-ShapeClass.md) | interface |  | og.Shape: `new og.Shape(points?)`. |
| [`og.Path2`](#value-Path2) | value |  | three.js's Path (named Path2 beside og.path's 3D paths): `new og.Path2().moveTo(0, 0).lineTo(1, 0)`. |
| [`og.Shape`](#value-Shape) | value |  | three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`. |

## Values

Provided by the TypeScript runtime: no binding symbol.

<a id="value-Path2"></a>

### `og.Path2`

three.js's Path (named Path2 beside og.path's 3D paths): `new og.Path2().moveTo(0, 0).lineTo(1, 0)`.

```ts
const Path2: Path2Class
```

```ogt
value Path2 Path2Class "three.js's Path (named Path2 beside og.path's 3D paths): `new og.Path2().moveTo(0, 0).lineTo(1, 0)`."
```

<sub>Module `shapes` line 184</sub>

<a id="value-Shape"></a>

### `og.Shape`

three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`.

```ts
const Shape: ShapeClass
```

```ogt
value Shape ShapeClass "three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`."
```

<sub>Module `shapes` line 185</sub>
