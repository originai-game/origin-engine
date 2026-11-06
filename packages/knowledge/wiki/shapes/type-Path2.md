<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shapes`](../shapes.md) · [index](../index.md)

<a id="type-Path2"></a>

### `Path2` (interface)

An outline of curves, as three.js Path: each call adds a curve from the current point (moveTo only moves it). Every method but getPoints returns the path, so calls chain.

```ogt
interface Path2 {
  doc "An outline of curves, as three.js Path: each call adds a curve from the current point (moveTo only moves it). Every method but getPoints returns the path, so calls chain."
  # members follow, each quoted below
```

<sub>Module `shapes` line 20</sub>

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

<sub>Module `shapes` line 22</sub>

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

<sub>Module `shapes` line 29</sub>

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

<sub>Module `shapes` line 36</sub>

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

<sub>Module `shapes` line 45</sub>

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

<sub>Module `shapes` line 56</sub>

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

<sub>Module `shapes` line 62</sub>

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

<sub>Module `shapes` line 75</sub>

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

<sub>Module `shapes` line 88</sub>

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

<sub>Module `shapes` line 105</sub>

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

<sub>Module `shapes` line 122</sub>

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

<sub>Module `shapes` line 128</sub>

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

<sub>Module `shapes` line 133</sub>
