<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `shapes` module

Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, quadraticCurveTo, bezierCurveTo, splineThru, arc, absarc, ellipse, absellipse, closePath) and flattened to points by getPoints, so a three.js outline is a paste. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine.

They are TypeScript values, not world state: make them in start or a command and let the geometry made from them be what the world keeps. Points come from basic arithmetic (no Math.sin), so they are the same on every host.

A prop of these parts belongs in an asset: an `.ogt` of `type model` names each part's geometry by its og.render call (`extrudeGeometry { shape { absarc 0 0 1 0 6.283185307179586 false } depth 0.2 }`), its shapes as og.Shape's calls one a line with `hole { }` blocks, places parts as mergeGeometries does (`position`, `quaternion`, `scale`, `mirror`, or several `at { }` blocks: one mesh drawn instanced), gives each a `material` and hangs it from a named `pivot` (a node). og cook builds it into the model blob a glTF cooks to (levels of detail, raycast BVH, nodes), refuses it past project.ogt's triangle and vertex budgets, and og dev re-cooks it on an edit; og.load.gltf and a scene's `model` line take it as any model.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module shapes {
  doc "Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, quadraticCurveTo, bezierCurveTo, splineThru, arc, absarc, ellipse, absellipse, closePath) and flattened to points by getPoints, so a three.js outline is a paste. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine."
  doc "They are TypeScript values, not world state: make them in start or a command and let the geometry made from them be what the world keeps. Points come from basic arithmetic (no Math.sin), so they are the same on every host."
  doc "A prop of these parts belongs in an asset: an `.ogt` of `type model` names each part's geometry by its og.render call (`extrudeGeometry { shape { absarc 0 0 1 0 6.283185307179586 false } depth 0.2 }`), its shapes as og.Shape's calls one a line with `hole { }` blocks, places parts as mergeGeometries does (`position`, `quaternion`, `scale`, `mirror`, or several `at { }` blocks: one mesh drawn instanced), gives each a `material` and hangs it from a named `pivot` (a node). og cook builds it into the model blob a glTF cooks to (levels of detail, raycast BVH, nodes), refuses it past project.ogt's triangle and vertex budgets, and og dev re-cooks it on an edit; og.load.gltf and a scene's `model` line take it as any model."
  place root
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 7723</sub>

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

<sub>`api.ogt` line 7906</sub>

<a id="value-Shape"></a>

### `og.Shape`

three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`.

```ts
const Shape: ShapeClass
```

```ogt
value Shape ShapeClass "three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`."
```

<sub>`api.ogt` line 7907</sub>
