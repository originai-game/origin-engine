<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Box3Math"></a>

### `Box3Math` (interface)

Pure functions over Box3, as three.js Box3's methods, each returning a new value. The value `box3`.

```ogt
interface Box3Math {
  doc "Pure functions over Box3, as three.js Box3's methods, each returning a new value. The value `box3`."
  # members follow, each quoted below
```

<sub>Module `math` line 303</sub>

<a id="member-Box3Math-empty"></a>

#### `Box3Math.empty`

The empty box (min +Infinity, max -Infinity), as new three.js Box3(): expandByPoint and union grow it.

```ts
Box3Math.empty(): Box3
```

**Returns** `Box3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn empty {
  doc "The empty box (min +Infinity, max -Infinity), as new three.js Box3(): expandByPoint and union grow it."
  host false
  returns Box3
}
```

<sub>Module `math` line 305</sub>

<a id="member-Box3Math-fromPoints"></a>

#### `Box3Math.fromPoints`

The smallest box holding every point, as three.js Box3.setFromPoints; no points is the empty box.

```ts
Box3Math.fromPoints(points: readonly Vector3[]): Box3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector3[]` | no | The points. |

**Returns** `Box3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromPoints {
  doc "The smallest box holding every point, as three.js Box3.setFromPoints; no points is the empty box."
  host false
  param points List.Vector3 "The points."
  returns Box3
}
```

<sub>Module `math` line 310</sub>

<a id="member-Box3Math-isEmpty"></a>

#### `Box3Math.isEmpty`

Whether the box holds no point (min above max on some axis), as three.js Box3.isEmpty.

```ts
Box3Math.isEmpty(box: Box3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn isEmpty {
  doc "Whether the box holds no point (min above max on some axis), as three.js Box3.isEmpty."
  host false
  param box Box3 "The box."
  returns bool
}
```

<sub>Module `math` line 316</sub>

<a id="member-Box3Math-containsPoint"></a>

#### `Box3Math.containsPoint`

Whether p is inside or on the box, as three.js Box3.containsPoint.

```ts
Box3Math.containsPoint(box: Box3, p: Vector3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |
| `p` | `Vector3` | no | The point. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn containsPoint {
  doc "Whether p is inside or on the box, as three.js Box3.containsPoint."
  host false
  param box Box3 "The box."
  param p Vector3 "The point."
  returns bool
}
```

<sub>Module `math` line 322</sub>

<a id="member-Box3Math-containsBox"></a>

#### `Box3Math.containsBox`

Whether other lies wholly inside box, as three.js Box3.containsBox.

```ts
Box3Math.containsBox(box: Box3, other: Box3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The outer box. |
| `other` | `Box3` | no | The inner box. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn containsBox {
  doc "Whether other lies wholly inside box, as three.js Box3.containsBox."
  host false
  param box Box3 "The outer box."
  param other Box3 "The inner box."
  returns bool
}
```

<sub>Module `math` line 329</sub>

<a id="member-Box3Math-intersectsBox"></a>

#### `Box3Math.intersectsBox`

Whether two boxes overlap or touch, as three.js Box3.intersectsBox.

```ts
Box3Math.intersectsBox(a: Box3, b: Box3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Box3` | no | The first box. |
| `b` | `Box3` | no | The second box. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsBox {
  doc "Whether two boxes overlap or touch, as three.js Box3.intersectsBox."
  host false
  param a Box3 "The first box."
  param b Box3 "The second box."
  returns bool
}
```

<sub>Module `math` line 336</sub>

<a id="member-Box3Math-intersectsSphere"></a>

#### `Box3Math.intersectsSphere`

Whether the box and the sphere overlap, as three.js Box3.intersectsSphere.

```ts
Box3Math.intersectsSphere(box: Box3, sphere: Sphere): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |
| `sphere` | `Sphere` | no | The sphere. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsSphere {
  doc "Whether the box and the sphere overlap, as three.js Box3.intersectsSphere."
  host false
  param box Box3 "The box."
  param sphere Sphere "The sphere."
  returns bool
}
```

<sub>Module `math` line 343</sub>

<a id="member-Box3Math-expandByPoint"></a>

#### `Box3Math.expandByPoint`

The box grown to hold p, as three.js Box3.expandByPoint.

```ts
Box3Math.expandByPoint(box: Box3, p: Vector3): Box3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |
| `p` | `Vector3` | no | The point. |

**Returns** `Box3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn expandByPoint {
  doc "The box grown to hold p, as three.js Box3.expandByPoint."
  host false
  param box Box3 "The box."
  param p Vector3 "The point."
  returns Box3
}
```

<sub>Module `math` line 350</sub>

<a id="member-Box3Math-union"></a>

#### `Box3Math.union`

The smallest box holding both, as three.js Box3.union.

```ts
Box3Math.union(a: Box3, b: Box3): Box3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Box3` | no | The first box. |
| `b` | `Box3` | no | The second box. |

**Returns** `Box3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn union {
  doc "The smallest box holding both, as three.js Box3.union."
  host false
  param a Box3 "The first box."
  param b Box3 "The second box."
  returns Box3
}
```

<sub>Module `math` line 357</sub>

<a id="member-Box3Math-getCenter"></a>

#### `Box3Math.getCenter`

The centre, as three.js Box3.getCenter; the empty box's is the origin.

```ts
Box3Math.getCenter(box: Box3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn getCenter {
  doc "The centre, as three.js Box3.getCenter; the empty box's is the origin."
  host false
  param box Box3 "The box."
  returns Vector3
}
```

<sub>Module `math` line 364</sub>

<a id="member-Box3Math-getSize"></a>

#### `Box3Math.getSize`

The size along each axis, as three.js Box3.getSize; the empty box's is zero.

```ts
Box3Math.getSize(box: Box3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn getSize {
  doc "The size along each axis, as three.js Box3.getSize; the empty box's is zero."
  host false
  param box Box3 "The box."
  returns Vector3
}
```

<sub>Module `math` line 370</sub>

<a id="member-Box3Math-applyMatrix4"></a>

#### `Box3Math.applyMatrix4`

The box holding the eight corners transformed by m, as three.js Box3.applyMatrix4.

```ts
Box3Math.applyMatrix4(box: Box3, m: Matrix4): Box3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `box` | `Box3` | no | The box. |
| `m` | `Matrix4` | no | The transform. |

**Returns** `Box3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn applyMatrix4 {
  doc "The box holding the eight corners transformed by m, as three.js Box3.applyMatrix4."
  host false
  param box Box3 "The box."
  param m Matrix4 "The transform."
  returns Box3
}
```

<sub>Module `math` line 376</sub>
