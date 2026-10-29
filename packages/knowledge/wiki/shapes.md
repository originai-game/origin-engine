<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `shapes` module

Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, quadraticCurveTo, bezierCurveTo, splineThru, arc, absarc, ellipse, absellipse, closePath) and flattened to points by getPoints, so a three.js outline is a paste. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine.

They are TypeScript values, not world state: make them in start or a command and let the geometry made from them be what the world keeps. Points come from basic arithmetic (no Math.sin), so they are the same on every host.

A prop of these parts belongs in an asset: an `.ogt` of `type model` names each part's geometry by its og.render call (`extrudeGeometry { shape { absarc 0 0 1 0 6.283185307179586 false } depth 0.2 }`), its shapes as og.Shape's calls one a line with `hole { }` blocks, places parts as mergeGeometries does (`position`, `quaternion`, `scale`, `mirror`, or several `at { }` blocks: one mesh drawn instanced), gives each a `material` and hangs it from a named `pivot` (a node). og cook builds it into the model blob a glTF cooks to (levels of detail, raycast BVH, nodes), refuses it past project.ogt's triangle and vertex budgets, and og dev re-cooks it on an edit; og.load.gltf and a scene's `model` line take it as any model. The og-game skill §18 has a whole example.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module shapes {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 5470</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`PathList`](#type-PathList) | interface | A shape's holes, as three.js Shape.holes: an array; push a path to add a hole. |
| [`Path2`](#type-Path2) | interface | An outline of curves, as three.js Path: each call adds a curve from the current point (moveTo only moves it). Every method but getPoints returns the path, so calls chain. |
| [`Shape`](#type-Shape) | interface | An outline with holes, as three.js Shape. |
| [`Path2Class`](#type-Path2Class) | interface | og.Path2: `new og.Path2(points?)`. |
| [`ShapeClass`](#type-ShapeClass) | interface | og.Shape: `new og.Shape(points?)`. |
| [`og.Path2`](#value-Path2) | value | three.js's Path (named Path2 beside og.path's 3D paths): `new og.Path2().moveTo(0, 0).lineTo(1, 0)`. |
| [`og.Shape`](#value-Shape) | value | three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`. |

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

<sub>`api.ogt` line 5653</sub>

<a id="value-Shape"></a>

### `og.Shape`

three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`.

```ts
const Shape: ShapeClass
```

```ogt
value Shape ShapeClass "three.js's Shape: `const s = new og.Shape(); s.moveTo(0, 0).lineTo(1, 0).lineTo(0, 1); s.holes.push(hole)`."
```

<sub>`api.ogt` line 5654</sub>

## Interfaces

<a id="type-PathList"></a>

### `PathList` (interface)

A shape's holes, as three.js Shape.holes: an array; push a path to add a hole.

```ogt
interface PathList {
  doc "A shape's holes, as three.js Shape.holes: an array; push a path to add a hole."
  # members follow, each quoted below
```

<sub>`api.ogt` line 5477</sub>

<a id="member-PathList-length"></a>

#### `PathList.length` (property)

How many holes.

```ts
readonly length: number
```

```ogt
prop length u32 "How many holes." {
  host false
}
```

<sub>`api.ogt` line 5479</sub>

<a id="member-PathList-push"></a>

#### `PathList.push`

Add a hole (Array.push).

```ts
PathList.push(path: Path2): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `Path2` | no | The hole's outline, either winding. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn push {
  doc "Add a hole (Array.push)."
  host false
  param path Path2 "The hole's outline, either winding."
  returns u32
}
```

<sub>`api.ogt` line 5482</sub>

<a id="type-Path2"></a>

### `Path2` (interface)

An outline of curves, as three.js Path: each call adds a curve from the current point (moveTo only moves it). Every method but getPoints returns the path, so calls chain.

```ogt
interface Path2 {
  doc "An outline of curves, as three.js Path: each call adds a curve from the current point (moveTo only moves it). Every method but getPoints returns the path, so calls chain."
  # members follow, each quoted below
```

<sub>`api.ogt` line 5490</sub>

<a id="member-Path2-moveTo"></a>

#### `Path2.moveTo`

Move the current point without drawing, as three.js Path.moveTo.

```ts
Path2.moveTo(x: number, y: number): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | x. |
| `y` | `number` | no | y. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn moveTo {
  doc "Move the current point without drawing, as three.js Path.moveTo."
  host false
  param x f32 "x."
  param y f32 "y."
  returns This
}
```

<sub>`api.ogt` line 5492</sub>

<a id="member-Path2-lineTo"></a>

#### `Path2.lineTo`

A straight line to (x, y), as three.js Path.lineTo.

```ts
Path2.lineTo(x: number, y: number): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | x. |
| `y` | `number` | no | y. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn lineTo {
  doc "A straight line to (x, y), as three.js Path.lineTo."
  host false
  param x f32 "x."
  param y f32 "y."
  returns This
}
```

<sub>`api.ogt` line 5499</sub>

<a id="member-Path2-quadraticCurveTo"></a>

#### `Path2.quadraticCurveTo`

A quadratic Bézier curve through one control point to (x, y), as three.js Path.quadraticCurveTo.

```ts
Path2.quadraticCurveTo(cpX: number, cpY: number, x: number, y: number): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `cpX` | `number` | no | The control point's x. |
| `cpY` | `number` | no | The control point's y. |
| `x` | `number` | no | The end's x. |
| `y` | `number` | no | The end's y. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn quadraticCurveTo {
  doc "A quadratic Bézier curve through one control point to (x, y), as three.js Path.quadraticCurveTo."
  host false
  param cpX f32 "The control point's x."
  param cpY f32 "The control point's y."
  param x f32 "The end's x."
  param y f32 "The end's y."
  returns This
}
```

<sub>`api.ogt` line 5506</sub>

<a id="member-Path2-bezierCurveTo"></a>

#### `Path2.bezierCurveTo`

A cubic Bézier curve through two control points to (x, y), as three.js Path.bezierCurveTo.

```ts
Path2.bezierCurveTo(cp1X: number, cp1Y: number, cp2X: number, cp2Y: number, x: number, y: number): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `cp1X` | `number` | no | The first control point's x. |
| `cp1Y` | `number` | no | The first control point's y. |
| `cp2X` | `number` | no | The second control point's x. |
| `cp2Y` | `number` | no | The second control point's y. |
| `x` | `number` | no | The end's x. |
| `y` | `number` | no | The end's y. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn bezierCurveTo {
  doc "A cubic Bézier curve through two control points to (x, y), as three.js Path.bezierCurveTo."
  host false
  param cp1X f32 "The first control point's x."
  param cp1Y f32 "The first control point's y."
  param cp2X f32 "The second control point's x."
  param cp2Y f32 "The second control point's y."
  param x f32 "The end's x."
  param y f32 "The end's y."
  returns This
}
```

<sub>`api.ogt` line 5515</sub>

<a id="member-Path2-splineThru"></a>

#### `Path2.splineThru`

A Catmull-Rom spline from the current point through the points, as three.js Path.splineThru (a SplineCurve).

```ts
Path2.splineThru(points: readonly Vector2[]): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector2[]` | no | The points it passes through, in order. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn splineThru {
  doc "A Catmull-Rom spline from the current point through the points, as three.js Path.splineThru (a SplineCurve)."
  host false
  param points List.Vector2 "The points it passes through, in order."
  returns This
}
```

<sub>`api.ogt` line 5526</sub>

<a id="member-Path2-arc"></a>

#### `Path2.arc`

A circular arc centred at (x, y) from the current point, as three.js Path.arc.

```ts
Path2.arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, clockwise?: boolean): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The centre's x, from the current point. |
| `y` | `number` | no | The centre's y, from the current point. |
| `radius` | `number` | no | The radius. |
| `startAngle` | `number` | no | Where it starts, from +x towards +y. |
| `endAngle` | `number` | no | Where it ends. |
| `clockwise` | `boolean` | yes | Go clockwise. Default `false`. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn arc {
  doc "A circular arc centred at (x, y) from the current point, as three.js Path.arc."
  host false
  param x f32 "The centre's x, from the current point."
  param y f32 "The centre's y, from the current point."
  param radius f32 "The radius."
  param startAngle f32 "Where it starts, from +x towards +y."
  param endAngle f32 "Where it ends."
  param clockwise bool "Go clockwise." {
    default false
  }
  returns This
}
```

<sub>`api.ogt` line 5532</sub>

<a id="member-Path2-absarc"></a>

#### `Path2.absarc`

A circular arc centred at (x, y), as three.js Path.absarc: a line joins it to the current point when they differ.

```ts
Path2.absarc(x: number, y: number, radius: number, startAngle: number, endAngle: number, clockwise?: boolean): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The centre's x. |
| `y` | `number` | no | The centre's y. |
| `radius` | `number` | no | The radius. |
| `startAngle` | `number` | no | Where it starts, from +x towards +y. |
| `endAngle` | `number` | no | Where it ends. |
| `clockwise` | `boolean` | yes | Go clockwise. Default `false`. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn absarc {
  doc "A circular arc centred at (x, y), as three.js Path.absarc: a line joins it to the current point when they differ."
  host false
  param x f32 "The centre's x."
  param y f32 "The centre's y."
  param radius f32 "The radius."
  param startAngle f32 "Where it starts, from +x towards +y."
  param endAngle f32 "Where it ends."
  param clockwise bool "Go clockwise." {
    default false
  }
  returns This
}
```

<sub>`api.ogt` line 5545</sub>

<a id="member-Path2-ellipse"></a>

#### `Path2.ellipse`

An elliptic arc centred at (x, y) from the current point, as three.js Path.ellipse.

```ts
Path2.ellipse(x: number, y: number, xRadius: number, yRadius: number, startAngle: number, endAngle: number, clockwise?: boolean, rotation?: number): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The centre's x, from the current point. |
| `y` | `number` | no | The centre's y, from the current point. |
| `xRadius` | `number` | no | The radius along x. |
| `yRadius` | `number` | no | The radius along y. |
| `startAngle` | `number` | no | Where it starts. |
| `endAngle` | `number` | no | Where it ends. |
| `clockwise` | `boolean` | yes | Go clockwise. Default `false`. |
| `rotation` | `number` | yes | The ellipse's turn round its centre. Default `0`. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn ellipse {
  doc "An elliptic arc centred at (x, y) from the current point, as three.js Path.ellipse."
  host false
  param x f32 "The centre's x, from the current point."
  param y f32 "The centre's y, from the current point."
  param xRadius f32 "The radius along x."
  param yRadius f32 "The radius along y."
  param startAngle f32 "Where it starts."
  param endAngle f32 "Where it ends."
  param clockwise bool "Go clockwise." {
    default false
  }
  param rotation f32 "The ellipse's turn round its centre." {
    default 0
  }
  returns This
}
```

<sub>`api.ogt` line 5558</sub>

<a id="member-Path2-absellipse"></a>

#### `Path2.absellipse`

An elliptic arc centred at (x, y), as three.js Path.absellipse.

```ts
Path2.absellipse(x: number, y: number, xRadius: number, yRadius: number, startAngle: number, endAngle: number, clockwise?: boolean, rotation?: number): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The centre's x. |
| `y` | `number` | no | The centre's y. |
| `xRadius` | `number` | no | The radius along x. |
| `yRadius` | `number` | no | The radius along y. |
| `startAngle` | `number` | no | Where it starts. |
| `endAngle` | `number` | no | Where it ends. |
| `clockwise` | `boolean` | yes | Go clockwise. Default `false`. |
| `rotation` | `number` | yes | The ellipse's turn round its centre. Default `0`. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn absellipse {
  doc "An elliptic arc centred at (x, y), as three.js Path.absellipse."
  host false
  param x f32 "The centre's x."
  param y f32 "The centre's y."
  param xRadius f32 "The radius along x."
  param yRadius f32 "The radius along y."
  param startAngle f32 "Where it starts."
  param endAngle f32 "Where it ends."
  param clockwise bool "Go clockwise." {
    default false
  }
  param rotation f32 "The ellipse's turn round its centre." {
    default 0
  }
  returns This
}
```

<sub>`api.ogt` line 5575</sub>

<a id="member-Path2-setFromPoints"></a>

#### `Path2.setFromPoints`

Start at the first point and draw lines through the rest, as three.js Path.setFromPoints.

```ts
Path2.setFromPoints(points: readonly Vector2[]): this
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector2[]` | no | The points. |

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn setFromPoints {
  doc "Start at the first point and draw lines through the rest, as three.js Path.setFromPoints."
  host false
  param points List.Vector2 "The points."
  returns This
}
```

<sub>`api.ogt` line 5592</sub>

<a id="member-Path2-closePath"></a>

#### `Path2.closePath`

A line back to the first point when the outline does not end there, as three.js CurvePath.closePath.

```ts
Path2.closePath(): this
```

**Returns** `this`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn closePath {
  doc "A line back to the first point when the outline does not end there, as three.js CurvePath.closePath."
  host false
  returns This
}
```

<sub>`api.ogt` line 5598</sub>

<a id="member-Path2-getPoints"></a>

#### `Path2.getPoints`

The outline as points, as three.js CurvePath.getPoints: each curve divided into `divisions` (an ellipse twice as many, a spline as many per point, a line one), repeated points dropped.

```ts
Path2.getPoints(divisions?: number): readonly Vector2[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `divisions` | `number` | yes | Divisions per curve. Default `12`. |

**Returns** `readonly Vector2[]`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn getPoints {
  doc "The outline as points, as three.js CurvePath.getPoints: each curve divided into `divisions` (an ellipse twice as many, a spline as many per point, a line one), repeated points dropped."
  host false
  param divisions u32 "Divisions per curve." {
    default 12
  }
  returns List.Vector2
}
```

<sub>`api.ogt` line 5603</sub>

<a id="type-Shape"></a>

### `Shape` (interface)

An outline with holes, as three.js Shape.

Extends [`Path2`](#type-Path2): every member of it is a member here.

```ogt
interface Shape {
  doc "An outline with holes, as three.js Shape."
  extends Path2
  # members follow, each quoted below
```

<sub>`api.ogt` line 5613</sub>

<a id="member-Shape-holes"></a>

#### `Shape.holes` (property)

The holes: `shape.holes.push(hole)`.

```ts
readonly holes: PathList
```

```ogt
prop holes PathList "The holes: `shape.holes.push(hole)`." {
  host false
}
```

<sub>`api.ogt` line 5616</sub>

<a id="member-Shape-extractPoints"></a>

#### `Shape.extractPoints`

The outline's and the holes' points, as three.js Shape.extractPoints.

```ts
Shape.extractPoints(divisions?: number): ShapePoints
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `divisions` | `number` | yes | Divisions per curve. Default `12`. |

**Returns** `ShapePoints`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn extractPoints {
  doc "The outline's and the holes' points, as three.js Shape.extractPoints."
  host false
  param divisions u32 "Divisions per curve." {
    default 12
  }
  returns ShapePoints
}
```

<sub>`api.ogt` line 5619</sub>

<a id="type-Path2Class"></a>

### `Path2Class` (interface)

og.Path2: `new og.Path2(points?)`.

```ogt
interface Path2Class {
  doc "og.Path2: `new og.Path2(points?)`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 5629</sub>

<a id="member-Path2Class-new"></a>

#### `Path2Class.new`

A path, through the points when given (setFromPoints).

```ts
Path2Class.new(points?: readonly Vector2[] | null): Path2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector2[] \| null` | yes | Points to start from. |

**Returns** `Path2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn new {
  doc "A path, through the points when given (setFromPoints)."
  host false
  param points Option.List.Vector2 "Points to start from." {
    optional true
  }
  returns Path2
}
```

<sub>`api.ogt` line 5631</sub>

<a id="type-ShapeClass"></a>

### `ShapeClass` (interface)

og.Shape: `new og.Shape(points?)`.

```ogt
interface ShapeClass {
  doc "og.Shape: `new og.Shape(points?)`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 5641</sub>

<a id="member-ShapeClass-new"></a>

#### `ShapeClass.new`

A shape, through the points when given (setFromPoints).

```ts
ShapeClass.new(points?: readonly Vector2[] | null): Shape
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector2[] \| null` | yes | Points to start from. |

**Returns** `Shape`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn new {
  doc "A shape, through the points when given (setFromPoints)."
  host false
  param points Option.List.Vector2 "Points to start from." {
    optional true
  }
  returns Shape
}
```

<sub>`api.ogt` line 5643</sub>
