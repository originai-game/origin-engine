<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `math` module

Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-core`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module math {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 37</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Vector2`](#type-Vector2) | struct | A 2D vector, as three.js Vector2. |
| [`Vector3`](#type-Vector3) | struct | A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js. |
| [`Quaternion`](#type-Quaternion) | struct | A unit quaternion rotation, as three.js Quaternion. The zero value is the identity. |
| [`Matrix4`](#type-Matrix4) | struct | A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity. |
| [`Color`](#type-Color) | struct | A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR. |
| [`Ray`](#type-Ray) | struct | A ray, as three.js Ray. |
| [`Box3`](#type-Box3) | struct | An axis-aligned box, as three.js Box3. A box whose min exceeds its max on an axis is empty (box3.empty() makes one: min +Infinity, max -Infinity). |
| [`Sphere`](#type-Sphere) | struct | A sphere, as three.js Sphere. A negative radius is empty. |
| [`Plane`](#type-Plane) | struct | A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points. |
| [`Frustum`](#type-Frustum) | struct | A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). frustum.fromCamera and frustum.fromProjectionMatrix make one. |
| [`Vector3Math`](#type-Vector3Math) | interface | Pure functions over Vector3, as three.js Vector3's methods of the same names: each returns a new value and changes none of its arguments. The value `vec3`: `vec3.add(a, b)`. |
| [`Vector2Math`](#type-Vector2Math) | interface | Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. The value `vec2`: `vec2.add(a, b)`. The same bits on every host. |
| [`QuaternionMath`](#type-QuaternionMath) | interface | Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. The value `quat`: `quat.multiply(a, b)`. The same bits on every host (trigonometry from + - * / and sqrt). |
| [`Matrix4Math`](#type-Matrix4Math) | interface | Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. The value `mat4`: `mat4.multiply(a, b)`. |
| [`Box3Math`](#type-Box3Math) | interface | Pure functions over Box3, as three.js Box3's methods, each returning a new value. The value `box3`. |
| [`SphereMath`](#type-SphereMath) | interface | Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. The value `sphere`. |
| [`PlaneMath`](#type-PlaneMath) | interface | Pure functions over Plane, as three.js Plane's methods, each returning a new value. The value `plane`. |
| [`FrustumMath`](#type-FrustumMath) | interface | Pure functions over Frustum, as three.js Frustum's methods. The value `frustum`: `frustum.intersectsBox(frustum.fromCamera(camera, 16 / 9), box)` culls as the renderer would. |
| [`RayMath`](#type-RayMath) | interface | Pure functions over Ray, as three.js Ray's methods. The value `ray`. Each intersection answers the nearest point at or after the origin, or null. |
| [`ColorMath`](#type-ColorMath) | interface | Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. The value `color`: `color.fromHex(0xff8800)`. The same bits on every host (no Math.pow). |
| [`vec3`](#value-vec3) | value | Vector3's pure functions: `vec3.add(a, b)`, `vec3.normalize(v)`, `vec3.applyQuaternion(v, q)`. |
| [`vec2`](#value-vec2) | value | Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`. |
| [`quat`](#value-quat) | value | Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`. |
| [`mat4`](#value-mat4) | value | Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`. |
| [`box3`](#value-box3) | value | Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`. |
| [`sphere`](#value-sphere) | value | Sphere's pure functions: `sphere.containsPoint(s, p)`. |
| [`plane`](#value-plane) | value | Plane's pure functions: `plane.distanceToPoint(pl, p)`. |
| [`frustum`](#value-frustum) | value | Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`. |
| [`ray`](#value-ray) | value | Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`. |
| [`color`](#value-color) | value | Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`. |

## Values

Provided by the TypeScript runtime: no binding symbol.

<a id="value-vec3"></a>

### `vec3`

Vector3's pure functions: `vec3.add(a, b)`, `vec3.normalize(v)`, `vec3.applyQuaternion(v, q)`.

```ts
const vec3: Vector3Math
```

```ogt
value vec3 Vector3Math "Vector3's pure functions: `vec3.add(a, b)`, `vec3.normalize(v)`, `vec3.applyQuaternion(v, q)`."
```

<sub>`api.ogt` line 647</sub>

<a id="value-vec2"></a>

### `vec2`

Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`.

```ts
const vec2: Vector2Math
```

```ogt
value vec2 Vector2Math "Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`."
```

<sub>`api.ogt` line 648</sub>

<a id="value-quat"></a>

### `quat`

Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`.

```ts
const quat: QuaternionMath
```

```ogt
value quat QuaternionMath "Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`."
```

<sub>`api.ogt` line 649</sub>

<a id="value-mat4"></a>

### `mat4`

Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`.

```ts
const mat4: Matrix4Math
```

```ogt
value mat4 Matrix4Math "Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`."
```

<sub>`api.ogt` line 650</sub>

<a id="value-box3"></a>

### `box3`

Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`.

```ts
const box3: Box3Math
```

```ogt
value box3 Box3Math "Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`."
```

<sub>`api.ogt` line 651</sub>

<a id="value-sphere"></a>

### `sphere`

Sphere's pure functions: `sphere.containsPoint(s, p)`.

```ts
const sphere: SphereMath
```

```ogt
value sphere SphereMath "Sphere's pure functions: `sphere.containsPoint(s, p)`."
```

<sub>`api.ogt` line 652</sub>

<a id="value-plane"></a>

### `plane`

Plane's pure functions: `plane.distanceToPoint(pl, p)`.

```ts
const plane: PlaneMath
```

```ogt
value plane PlaneMath "Plane's pure functions: `plane.distanceToPoint(pl, p)`."
```

<sub>`api.ogt` line 653</sub>

<a id="value-frustum"></a>

### `frustum`

Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`.

```ts
const frustum: FrustumMath
```

```ogt
value frustum FrustumMath "Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`."
```

<sub>`api.ogt` line 654</sub>

<a id="value-ray"></a>

### `ray`

Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`.

```ts
const ray: RayMath
```

```ogt
value ray RayMath "Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`."
```

<sub>`api.ogt` line 655</sub>

<a id="value-color"></a>

### `color`

Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`.

```ts
const color: ColorMath
```

```ogt
value color ColorMath "Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`."
```

<sub>`api.ogt` line 656</sub>

## Interfaces

<a id="type-Vector3Math"></a>

### `Vector3Math` (interface)

Pure functions over Vector3, as three.js Vector3's methods of the same names: each returns a new value and changes none of its arguments. The value `vec3`: `vec3.add(a, b)`.

No member reads Math.sin, Math.pow or the clock: results are the same bits on every host, so a system may store them in world state.

```ogt
interface Vector3Math {
  doc "Pure functions over Vector3, as three.js Vector3's methods of the same names: each returns a new value and changes none of its arguments. The value `vec3`: `vec3.add(a, b)`."
  doc "No member reads Math.sin, Math.pow or the clock: results are the same bits on every host, so a system may store them in world state."
  # members follow, each quoted below
```

<sub>`api.ogt` line 112</sub>

<a id="member-Vector3Math-add"></a>

#### `Vector3Math.add`

a + b, as three.js Vector3.addVectors.

```ts
Vector3Math.add(a: Vector3, b: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The first vector. |
| `b` | `Vector3` | no | The second vector. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn add {
  doc "a + b, as three.js Vector3.addVectors."
  host false
  param a Vector3 "The first vector."
  param b Vector3 "The second vector."
  returns Vector3
}
```

<sub>`api.ogt` line 115</sub>

<a id="member-Vector3Math-sub"></a>

#### `Vector3Math.sub`

a - b, as three.js Vector3.subVectors.

```ts
Vector3Math.sub(a: Vector3, b: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The vector subtracted from. |
| `b` | `Vector3` | no | The vector subtracted. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn sub {
  doc "a - b, as three.js Vector3.subVectors."
  host false
  param a Vector3 "The vector subtracted from."
  param b Vector3 "The vector subtracted."
  returns Vector3
}
```

<sub>`api.ogt` line 122</sub>

<a id="member-Vector3Math-scale"></a>

#### `Vector3Math.scale`

v * s, as three.js Vector3.multiplyScalar.

```ts
Vector3Math.scale(v: Vector3, s: number): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector3` | no | The vector. |
| `s` | `number` | no | The factor. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn scale {
  doc "v * s, as three.js Vector3.multiplyScalar."
  host false
  param v Vector3 "The vector."
  param s f32 "The factor."
  returns Vector3
}
```

<sub>`api.ogt` line 129</sub>

<a id="member-Vector3Math-dot"></a>

#### `Vector3Math.dot`

The dot product, as three.js Vector3.dot.

```ts
Vector3Math.dot(a: Vector3, b: Vector3): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The first vector. |
| `b` | `Vector3` | no | The second vector. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dot {
  doc "The dot product, as three.js Vector3.dot."
  host false
  param a Vector3 "The first vector."
  param b Vector3 "The second vector."
  returns f32
}
```

<sub>`api.ogt` line 136</sub>

<a id="member-Vector3Math-cross"></a>

#### `Vector3Math.cross`

The cross product a x b (right-handed), as three.js Vector3.crossVectors.

```ts
Vector3Math.cross(a: Vector3, b: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The first vector. |
| `b` | `Vector3` | no | The second vector. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn cross {
  doc "The cross product a x b (right-handed), as three.js Vector3.crossVectors."
  host false
  param a Vector3 "The first vector."
  param b Vector3 "The second vector."
  returns Vector3
}
```

<sub>`api.ogt` line 143</sub>

<a id="member-Vector3Math-length"></a>

#### `Vector3Math.length`

The Euclidean length, as three.js Vector3.length.

```ts
Vector3Math.length(v: Vector3): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector3` | no | The vector. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn length {
  doc "The Euclidean length, as three.js Vector3.length."
  host false
  param v Vector3 "The vector."
  returns f32
}
```

<sub>`api.ogt` line 150</sub>

<a id="member-Vector3Math-normalize"></a>

#### `Vector3Math.normalize`

The unit vector along v, as three.js Vector3.normalize; the zero vector stays zero.

```ts
Vector3Math.normalize(v: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector3` | no | The vector. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn normalize {
  doc "The unit vector along v, as three.js Vector3.normalize; the zero vector stays zero."
  host false
  param v Vector3 "The vector."
  returns Vector3
}
```

<sub>`api.ogt` line 156</sub>

<a id="member-Vector3Math-lerp"></a>

#### `Vector3Math.lerp`

a + (b - a) * t, as three.js Vector3.lerpVectors.

```ts
Vector3Math.lerp(a: Vector3, b: Vector3, t: number): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The value at t = 0. |
| `b` | `Vector3` | no | The value at t = 1. |
| `t` | `number` | no | The fraction (not clamped). |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn lerp {
  doc "a + (b - a) * t, as three.js Vector3.lerpVectors."
  host false
  param a Vector3 "The value at t = 0."
  param b Vector3 "The value at t = 1."
  param t f32 "The fraction (not clamped)."
  returns Vector3
}
```

<sub>`api.ogt` line 162</sub>

<a id="member-Vector3Math-distance"></a>

#### `Vector3Math.distance`

The distance between two points, as three.js Vector3.distanceTo.

```ts
Vector3Math.distance(a: Vector3, b: Vector3): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The first point. |
| `b` | `Vector3` | no | The second point. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn distance {
  doc "The distance between two points, as three.js Vector3.distanceTo."
  host false
  param a Vector3 "The first point."
  param b Vector3 "The second point."
  returns f32
}
```

<sub>`api.ogt` line 170</sub>

<a id="member-Vector3Math-applyQuaternion"></a>

#### `Vector3Math.applyQuaternion`

v rotated by q, as three.js Vector3.applyQuaternion.

```ts
Vector3Math.applyQuaternion(v: Vector3, q: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector3` | no | The vector. |
| `q` | `Quaternion` | no | The unit quaternion. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn applyQuaternion {
  doc "v rotated by q, as three.js Vector3.applyQuaternion."
  host false
  param v Vector3 "The vector."
  param q Quaternion "The unit quaternion."
  returns Vector3
}
```

<sub>`api.ogt` line 177</sub>

<a id="type-Vector2Math"></a>

### `Vector2Math` (interface)

Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. The value `vec2`: `vec2.add(a, b)`. The same bits on every host.

```ogt
interface Vector2Math {
  doc "Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. The value `vec2`: `vec2.add(a, b)`. The same bits on every host."
  # members follow, each quoted below
```

<sub>`api.ogt` line 186</sub>

<a id="member-Vector2Math-add"></a>

#### `Vector2Math.add`

a + b, as three.js Vector2.addVectors.

```ts
Vector2Math.add(a: Vector2, b: Vector2): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector2` | no | The first vector. |
| `b` | `Vector2` | no | The second vector. |

**Returns** `Vector2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn add {
  doc "a + b, as three.js Vector2.addVectors."
  host false
  param a Vector2 "The first vector."
  param b Vector2 "The second vector."
  returns Vector2
}
```

<sub>`api.ogt` line 188</sub>

<a id="member-Vector2Math-sub"></a>

#### `Vector2Math.sub`

a - b, as three.js Vector2.subVectors.

```ts
Vector2Math.sub(a: Vector2, b: Vector2): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector2` | no | The vector subtracted from. |
| `b` | `Vector2` | no | The vector subtracted. |

**Returns** `Vector2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn sub {
  doc "a - b, as three.js Vector2.subVectors."
  host false
  param a Vector2 "The vector subtracted from."
  param b Vector2 "The vector subtracted."
  returns Vector2
}
```

<sub>`api.ogt` line 195</sub>

<a id="member-Vector2Math-scale"></a>

#### `Vector2Math.scale`

v * s, as three.js Vector2.multiplyScalar.

```ts
Vector2Math.scale(v: Vector2, s: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector2` | no | The vector. |
| `s` | `number` | no | The factor. |

**Returns** `Vector2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn scale {
  doc "v * s, as three.js Vector2.multiplyScalar."
  host false
  param v Vector2 "The vector."
  param s f32 "The factor."
  returns Vector2
}
```

<sub>`api.ogt` line 202</sub>

<a id="member-Vector2Math-dot"></a>

#### `Vector2Math.dot`

The dot product, as three.js Vector2.dot.

```ts
Vector2Math.dot(a: Vector2, b: Vector2): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector2` | no | The first vector. |
| `b` | `Vector2` | no | The second vector. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dot {
  doc "The dot product, as three.js Vector2.dot."
  host false
  param a Vector2 "The first vector."
  param b Vector2 "The second vector."
  returns f32
}
```

<sub>`api.ogt` line 209</sub>

<a id="member-Vector2Math-cross"></a>

#### `Vector2Math.cross`

The 2D cross product a.x * b.y - a.y * b.x (the z of the 3D cross product), as three.js Vector2.cross: positive when b is counter-clockwise of a.

```ts
Vector2Math.cross(a: Vector2, b: Vector2): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector2` | no | The first vector. |
| `b` | `Vector2` | no | The second vector. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn cross {
  doc "The 2D cross product a.x * b.y - a.y * b.x (the z of the 3D cross product), as three.js Vector2.cross: positive when b is counter-clockwise of a."
  host false
  param a Vector2 "The first vector."
  param b Vector2 "The second vector."
  returns f32
}
```

<sub>`api.ogt` line 216</sub>

<a id="member-Vector2Math-length"></a>

#### `Vector2Math.length`

The Euclidean length, as three.js Vector2.length.

```ts
Vector2Math.length(v: Vector2): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector2` | no | The vector. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn length {
  doc "The Euclidean length, as three.js Vector2.length."
  host false
  param v Vector2 "The vector."
  returns f32
}
```

<sub>`api.ogt` line 223</sub>

<a id="member-Vector2Math-normalize"></a>

#### `Vector2Math.normalize`

The unit vector along v, as three.js Vector2.normalize; the zero vector stays zero.

```ts
Vector2Math.normalize(v: Vector2): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector2` | no | The vector. |

**Returns** `Vector2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn normalize {
  doc "The unit vector along v, as three.js Vector2.normalize; the zero vector stays zero."
  host false
  param v Vector2 "The vector."
  returns Vector2
}
```

<sub>`api.ogt` line 229</sub>

<a id="member-Vector2Math-lerp"></a>

#### `Vector2Math.lerp`

a + (b - a) * t, as three.js Vector2.lerpVectors.

```ts
Vector2Math.lerp(a: Vector2, b: Vector2, t: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector2` | no | The value at t = 0. |
| `b` | `Vector2` | no | The value at t = 1. |
| `t` | `number` | no | The fraction (not clamped). |

**Returns** `Vector2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn lerp {
  doc "a + (b - a) * t, as three.js Vector2.lerpVectors."
  host false
  param a Vector2 "The value at t = 0."
  param b Vector2 "The value at t = 1."
  param t f32 "The fraction (not clamped)."
  returns Vector2
}
```

<sub>`api.ogt` line 235</sub>

<a id="member-Vector2Math-distance"></a>

#### `Vector2Math.distance`

The distance between two points, as three.js Vector2.distanceTo.

```ts
Vector2Math.distance(a: Vector2, b: Vector2): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector2` | no | The first point. |
| `b` | `Vector2` | no | The second point. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn distance {
  doc "The distance between two points, as three.js Vector2.distanceTo."
  host false
  param a Vector2 "The first point."
  param b Vector2 "The second point."
  returns f32
}
```

<sub>`api.ogt` line 243</sub>

<a id="member-Vector2Math-rotateAround"></a>

#### `Vector2Math.rotateAround`

v rotated counter-clockwise by angle about center, as three.js Vector2.rotateAround (the 2D form of applyQuaternion).

```ts
Vector2Math.rotateAround(v: Vector2, center: Vector2, angle: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `v` | `Vector2` | no | The point. |
| `center` | `Vector2` | no | The pivot. |
| `angle` | `number` | no | The angle in radians, counter-clockwise. |

**Returns** `Vector2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn rotateAround {
  doc "v rotated counter-clockwise by angle about center, as three.js Vector2.rotateAround (the 2D form of applyQuaternion)."
  host false
  param v Vector2 "The point."
  param center Vector2 "The pivot."
  param angle f32 "The angle in radians, counter-clockwise."
  returns Vector2
}
```

<sub>`api.ogt` line 250</sub>

<a id="type-QuaternionMath"></a>

### `QuaternionMath` (interface)

Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. The value `quat`: `quat.multiply(a, b)`. The same bits on every host (trigonometry from + - * / and sqrt).

```ogt
interface QuaternionMath {
  doc "Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. The value `quat`: `quat.multiply(a, b)`. The same bits on every host (trigonometry from + - * / and sqrt)."
  # members follow, each quoted below
```

<sub>`api.ogt` line 260</sub>

<a id="member-QuaternionMath-multiply"></a>

#### `QuaternionMath.multiply`

a * b: b's rotation, then a's, as three.js Quaternion.multiplyQuaternions.

```ts
QuaternionMath.multiply(a: Quaternion, b: Quaternion): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Quaternion` | no | The rotation applied second. |
| `b` | `Quaternion` | no | The rotation applied first. |

**Returns** `Quaternion`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn multiply {
  doc "a * b: b's rotation, then a's, as three.js Quaternion.multiplyQuaternions."
  host false
  param a Quaternion "The rotation applied second."
  param b Quaternion "The rotation applied first."
  returns Quaternion
}
```

<sub>`api.ogt` line 262</sub>

<a id="member-QuaternionMath-slerp"></a>

#### `QuaternionMath.slerp`

The spherical interpolation from a to b by t along the shorter arc, as three.js Quaternion.slerpQuaternions.

```ts
QuaternionMath.slerp(a: Quaternion, b: Quaternion, t: number): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Quaternion` | no | The value at t = 0. |
| `b` | `Quaternion` | no | The value at t = 1. |
| `t` | `number` | no | The fraction, 0 to 1. |

**Returns** `Quaternion`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn slerp {
  doc "The spherical interpolation from a to b by t along the shorter arc, as three.js Quaternion.slerpQuaternions."
  host false
  param a Quaternion "The value at t = 0."
  param b Quaternion "The value at t = 1."
  param t f32 "The fraction, 0 to 1."
  returns Quaternion
}
```

<sub>`api.ogt` line 269</sub>

<a id="member-QuaternionMath-fromEuler"></a>

#### `QuaternionMath.fromEuler`

The rotation of Euler angles, as three.js Quaternion.setFromEuler: the x, y and z of `angles` in radians, applied in `order` (three.js Euler.order, intrinsic: "XYZ" turns about x, then the turned y, then the turned z).

```ts
QuaternionMath.fromEuler(angles: Vector3, order?: string): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `angles` | `Vector3` | no | The angles about x, y and z, in radians. |
| `order` | `string` | yes | "XYZ", "YXZ", "ZXY", "ZYX", "YZX" or "XZY". Default `"XYZ"`. |

**Returns** `Quaternion`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromEuler {
  doc "The rotation of Euler angles, as three.js Quaternion.setFromEuler: the x, y and z of `angles` in radians, applied in `order` (three.js Euler.order, intrinsic: \"XYZ\" turns about x, then the turned y, then the turned z)."
  host false
  param angles Vector3 "The angles about x, y and z, in radians."
  param order String "\"XYZ\", \"YXZ\", \"ZXY\", \"ZYX\", \"YZX\" or \"XZY\"." {
    default "XYZ"
  }
  returns Quaternion
}
```

<sub>`api.ogt` line 277</sub>

<a id="member-QuaternionMath-invert"></a>

#### `QuaternionMath.invert`

The inverse rotation (the conjugate of a unit quaternion), as three.js Quaternion.invert.

```ts
QuaternionMath.invert(q: Quaternion): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `q` | `Quaternion` | no | The rotation. |

**Returns** `Quaternion`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn invert {
  doc "The inverse rotation (the conjugate of a unit quaternion), as three.js Quaternion.invert."
  host false
  param q Quaternion "The rotation."
  returns Quaternion
}
```

<sub>`api.ogt` line 286</sub>

<a id="member-QuaternionMath-rotate"></a>

#### `QuaternionMath.rotate`

The vector v turned by q: vec3.applyQuaternion(v, q), from the rotation's side.

```ts
QuaternionMath.rotate(q: Quaternion, v: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `q` | `Quaternion` | no | The rotation. |
| `v` | `Vector3` | no | The vector. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn rotate {
  doc "The vector v turned by q: vec3.applyQuaternion(v, q), from the rotation's side."
  host false
  param q Quaternion "The rotation."
  param v Vector3 "The vector."
  returns Vector3
}
```

<sub>`api.ogt` line 292</sub>

<a id="type-Matrix4Math"></a>

### `Matrix4Math` (interface)

Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. The value `mat4`: `mat4.multiply(a, b)`.

```ogt
interface Matrix4Math {
  doc "Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. The value `mat4`: `mat4.multiply(a, b)`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 301</sub>

<a id="member-Matrix4Math-multiply"></a>

#### `Matrix4Math.multiply`

a * b: b's transform, then a's, as three.js Matrix4.multiplyMatrices.

```ts
Matrix4Math.multiply(a: Matrix4, b: Matrix4): Matrix4
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Matrix4` | no | The transform applied second. |
| `b` | `Matrix4` | no | The transform applied first. |

**Returns** `Matrix4`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn multiply {
  doc "a * b: b's transform, then a's, as three.js Matrix4.multiplyMatrices."
  host false
  param a Matrix4 "The transform applied second."
  param b Matrix4 "The transform applied first."
  returns Matrix4
}
```

<sub>`api.ogt` line 303</sub>

<a id="member-Matrix4Math-invert"></a>

#### `Matrix4Math.invert`

The inverse, as three.js Matrix4.invert; a singular matrix inverts to all zeros, as three.js.

```ts
Matrix4Math.invert(m: Matrix4): Matrix4
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `m` | `Matrix4` | no | The matrix. |

**Returns** `Matrix4`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn invert {
  doc "The inverse, as three.js Matrix4.invert; a singular matrix inverts to all zeros, as three.js."
  host false
  param m Matrix4 "The matrix."
  returns Matrix4
}
```

<sub>`api.ogt` line 310</sub>

<a id="member-Matrix4Math-transformPoint"></a>

#### `Matrix4Math.transformPoint`

The point p transformed by m with the perspective divide, as three.js Vector3.applyMatrix4.

```ts
Matrix4Math.transformPoint(m: Matrix4, p: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `m` | `Matrix4` | no | The transform. |
| `p` | `Vector3` | no | The point. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn transformPoint {
  doc "The point p transformed by m with the perspective divide, as three.js Vector3.applyMatrix4."
  host false
  param m Matrix4 "The transform."
  param p Vector3 "The point."
  returns Vector3
}
```

<sub>`api.ogt` line 316</sub>

<a id="member-Matrix4Math-transformDirection"></a>

#### `Matrix4Math.transformDirection`

The direction d turned by m's upper 3x3 and normalised, as three.js Vector3.transformDirection (translation ignored).

```ts
Matrix4Math.transformDirection(m: Matrix4, d: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `m` | `Matrix4` | no | The transform. |
| `d` | `Vector3` | no | The direction. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn transformDirection {
  doc "The direction d turned by m's upper 3x3 and normalised, as three.js Vector3.transformDirection (translation ignored)."
  host false
  param m Matrix4 "The transform."
  param d Vector3 "The direction."
  returns Vector3
}
```

<sub>`api.ogt` line 323</sub>

<a id="member-Matrix4Math-compose"></a>

#### `Matrix4Math.compose`

The transform of a position, a rotation and a scale, as three.js Matrix4.compose: what og.Transform's fields make.

```ts
Matrix4Math.compose(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `position` | `Vector3` | no | The translation. |
| `quaternion` | `Quaternion` | no | The rotation. |
| `scale` | `Vector3` | no | The scale. |

**Returns** `Matrix4`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn compose {
  doc "The transform of a position, a rotation and a scale, as three.js Matrix4.compose: what og.Transform's fields make."
  host false
  param position Vector3 "The translation."
  param quaternion Quaternion "The rotation."
  param scale Vector3 "The scale."
  returns Matrix4
}
```

<sub>`api.ogt` line 330</sub>

<a id="type-Box3Math"></a>

### `Box3Math` (interface)

Pure functions over Box3, as three.js Box3's methods, each returning a new value. The value `box3`.

```ogt
interface Box3Math {
  doc "Pure functions over Box3, as three.js Box3's methods, each returning a new value. The value `box3`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 340</sub>

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

<sub>`api.ogt` line 342</sub>

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

<sub>`api.ogt` line 347</sub>

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

<sub>`api.ogt` line 353</sub>

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

<sub>`api.ogt` line 359</sub>

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

<sub>`api.ogt` line 366</sub>

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

<sub>`api.ogt` line 373</sub>

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

<sub>`api.ogt` line 380</sub>

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

<sub>`api.ogt` line 387</sub>

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

<sub>`api.ogt` line 394</sub>

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

<sub>`api.ogt` line 401</sub>

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

<sub>`api.ogt` line 407</sub>

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

<sub>`api.ogt` line 413</sub>

<a id="type-SphereMath"></a>

### `SphereMath` (interface)

Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. The value `sphere`.

```ogt
interface SphereMath {
  doc "Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. The value `sphere`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 422</sub>

<a id="member-SphereMath-fromPoints"></a>

#### `SphereMath.fromPoints`

A sphere around the points, as three.js Sphere.setFromPoints: centred on their bounding box's centre, with the radius of the farthest point.

```ts
SphereMath.fromPoints(points: readonly Vector3[]): Sphere
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector3[]` | no | The points. |

**Returns** `Sphere`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromPoints {
  doc "A sphere around the points, as three.js Sphere.setFromPoints: centred on their bounding box's centre, with the radius of the farthest point."
  host false
  param points List.Vector3 "The points."
  returns Sphere
}
```

<sub>`api.ogt` line 424</sub>

<a id="member-SphereMath-containsPoint"></a>

#### `SphereMath.containsPoint`

Whether p is inside or on the sphere, as three.js Sphere.containsPoint.

```ts
SphereMath.containsPoint(sphere: Sphere, p: Vector3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `sphere` | `Sphere` | no | The sphere. |
| `p` | `Vector3` | no | The point. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn containsPoint {
  doc "Whether p is inside or on the sphere, as three.js Sphere.containsPoint."
  host false
  param sphere Sphere "The sphere."
  param p Vector3 "The point."
  returns bool
}
```

<sub>`api.ogt` line 430</sub>

<a id="member-SphereMath-distanceToPoint"></a>

#### `SphereMath.distanceToPoint`

The distance from the surface to p, negative inside, as three.js Sphere.distanceToPoint.

```ts
SphereMath.distanceToPoint(sphere: Sphere, p: Vector3): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `sphere` | `Sphere` | no | The sphere. |
| `p` | `Vector3` | no | The point. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn distanceToPoint {
  doc "The distance from the surface to p, negative inside, as three.js Sphere.distanceToPoint."
  host false
  param sphere Sphere "The sphere."
  param p Vector3 "The point."
  returns f32
}
```

<sub>`api.ogt` line 437</sub>

<a id="member-SphereMath-intersectsSphere"></a>

#### `SphereMath.intersectsSphere`

Whether two spheres overlap or touch, as three.js Sphere.intersectsSphere.

```ts
SphereMath.intersectsSphere(a: Sphere, b: Sphere): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Sphere` | no | The first sphere. |
| `b` | `Sphere` | no | The second sphere. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsSphere {
  doc "Whether two spheres overlap or touch, as three.js Sphere.intersectsSphere."
  host false
  param a Sphere "The first sphere."
  param b Sphere "The second sphere."
  returns bool
}
```

<sub>`api.ogt` line 444</sub>

<a id="member-SphereMath-intersectsBox"></a>

#### `SphereMath.intersectsBox`

Whether the sphere and the box overlap, as three.js Sphere.intersectsBox.

```ts
SphereMath.intersectsBox(sphere: Sphere, box: Box3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `sphere` | `Sphere` | no | The sphere. |
| `box` | `Box3` | no | The box. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsBox {
  doc "Whether the sphere and the box overlap, as three.js Sphere.intersectsBox."
  host false
  param sphere Sphere "The sphere."
  param box Box3 "The box."
  returns bool
}
```

<sub>`api.ogt` line 451</sub>

<a id="member-SphereMath-getBoundingBox"></a>

#### `SphereMath.getBoundingBox`

The box around the sphere, as three.js Sphere.getBoundingBox.

```ts
SphereMath.getBoundingBox(sphere: Sphere): Box3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `sphere` | `Sphere` | no | The sphere. |

**Returns** `Box3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn getBoundingBox {
  doc "The box around the sphere, as three.js Sphere.getBoundingBox."
  host false
  param sphere Sphere "The sphere."
  returns Box3
}
```

<sub>`api.ogt` line 458</sub>

<a id="member-SphereMath-applyMatrix4"></a>

#### `SphereMath.applyMatrix4`

The sphere transformed by m (its radius by m's largest axis scale), as three.js Sphere.applyMatrix4.

```ts
SphereMath.applyMatrix4(sphere: Sphere, m: Matrix4): Sphere
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `sphere` | `Sphere` | no | The sphere. |
| `m` | `Matrix4` | no | The transform. |

**Returns** `Sphere`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn applyMatrix4 {
  doc "The sphere transformed by m (its radius by m's largest axis scale), as three.js Sphere.applyMatrix4."
  host false
  param sphere Sphere "The sphere."
  param m Matrix4 "The transform."
  returns Sphere
}
```

<sub>`api.ogt` line 464</sub>

<a id="type-PlaneMath"></a>

### `PlaneMath` (interface)

Pure functions over Plane, as three.js Plane's methods, each returning a new value. The value `plane`.

```ogt
interface PlaneMath {
  doc "Pure functions over Plane, as three.js Plane's methods, each returning a new value. The value `plane`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 473</sub>

<a id="member-PlaneMath-fromNormalAndCoplanarPoint"></a>

#### `PlaneMath.fromNormalAndCoplanarPoint`

The plane through p with the normal n (normalised), as three.js Plane.setFromNormalAndCoplanarPoint.

```ts
PlaneMath.fromNormalAndCoplanarPoint(n: Vector3, p: Vector3): Plane
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `n` | `Vector3` | no | The normal. |
| `p` | `Vector3` | no | A point on the plane. |

**Returns** `Plane`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromNormalAndCoplanarPoint {
  doc "The plane through p with the normal n (normalised), as three.js Plane.setFromNormalAndCoplanarPoint."
  host false
  param n Vector3 "The normal."
  param p Vector3 "A point on the plane."
  returns Plane
}
```

<sub>`api.ogt` line 475</sub>

<a id="member-PlaneMath-fromCoplanarPoints"></a>

#### `PlaneMath.fromCoplanarPoints`

The plane through a, b and c, its normal (b - a) x (c - a) normalised (counter-clockwise points face you), as three.js Plane.setFromCoplanarPoints.

```ts
PlaneMath.fromCoplanarPoints(a: Vector3, b: Vector3, c: Vector3): Plane
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Vector3` | no | The first point. |
| `b` | `Vector3` | no | The second point. |
| `c` | `Vector3` | no | The third point. |

**Returns** `Plane`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromCoplanarPoints {
  doc "The plane through a, b and c, its normal (b - a) x (c - a) normalised (counter-clockwise points face you), as three.js Plane.setFromCoplanarPoints."
  host false
  param a Vector3 "The first point."
  param b Vector3 "The second point."
  param c Vector3 "The third point."
  returns Plane
}
```

<sub>`api.ogt` line 482</sub>

<a id="member-PlaneMath-normalize"></a>

#### `PlaneMath.normalize`

The same plane with a unit normal, as three.js Plane.normalize.

```ts
PlaneMath.normalize(plane: Plane): Plane
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `plane` | `Plane` | no | The plane. |

**Returns** `Plane`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn normalize {
  doc "The same plane with a unit normal, as three.js Plane.normalize."
  host false
  param plane Plane "The plane."
  returns Plane
}
```

<sub>`api.ogt` line 490</sub>

<a id="member-PlaneMath-distanceToPoint"></a>

#### `PlaneMath.distanceToPoint`

The signed distance from the plane to p, positive on the normal's side, as three.js Plane.distanceToPoint.

```ts
PlaneMath.distanceToPoint(plane: Plane, p: Vector3): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `plane` | `Plane` | no | The plane. |
| `p` | `Vector3` | no | The point. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn distanceToPoint {
  doc "The signed distance from the plane to p, positive on the normal's side, as three.js Plane.distanceToPoint."
  host false
  param plane Plane "The plane."
  param p Vector3 "The point."
  returns f32
}
```

<sub>`api.ogt` line 496</sub>

<a id="member-PlaneMath-projectPoint"></a>

#### `PlaneMath.projectPoint`

The point of the plane nearest p, as three.js Plane.projectPoint.

```ts
PlaneMath.projectPoint(plane: Plane, p: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `plane` | `Plane` | no | The plane. |
| `p` | `Vector3` | no | The point. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn projectPoint {
  doc "The point of the plane nearest p, as three.js Plane.projectPoint."
  host false
  param plane Plane "The plane."
  param p Vector3 "The point."
  returns Vector3
}
```

<sub>`api.ogt` line 503</sub>

<a id="member-PlaneMath-intersectsBox"></a>

#### `PlaneMath.intersectsBox`

Whether the plane passes through the box, as three.js Plane.intersectsBox.

```ts
PlaneMath.intersectsBox(plane: Plane, box: Box3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `plane` | `Plane` | no | The plane. |
| `box` | `Box3` | no | The box. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsBox {
  doc "Whether the plane passes through the box, as three.js Plane.intersectsBox."
  host false
  param plane Plane "The plane."
  param box Box3 "The box."
  returns bool
}
```

<sub>`api.ogt` line 510</sub>

<a id="member-PlaneMath-intersectsSphere"></a>

#### `PlaneMath.intersectsSphere`

Whether the plane passes through the sphere, as three.js Plane.intersectsSphere.

```ts
PlaneMath.intersectsSphere(plane: Plane, sphere: Sphere): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `plane` | `Plane` | no | The plane. |
| `sphere` | `Sphere` | no | The sphere. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsSphere {
  doc "Whether the plane passes through the sphere, as three.js Plane.intersectsSphere."
  host false
  param plane Plane "The plane."
  param sphere Sphere "The sphere."
  returns bool
}
```

<sub>`api.ogt` line 517</sub>

<a id="type-FrustumMath"></a>

### `FrustumMath` (interface)

Pure functions over Frustum, as three.js Frustum's methods. The value `frustum`: `frustum.intersectsBox(frustum.fromCamera(camera, 16 / 9), box)` culls as the renderer would.

```ogt
interface FrustumMath {
  doc "Pure functions over Frustum, as three.js Frustum's methods. The value `frustum`: `frustum.intersectsBox(frustum.fromCamera(camera, 16 / 9), box)` culls as the renderer would."
  # members follow, each quoted below
```

<sub>`api.ogt` line 526</sub>

<a id="member-FrustumMath-fromProjectionMatrix"></a>

#### `FrustumMath.fromProjectionMatrix`

The frustum of a view-projection matrix (projection * inverse camera world), as three.js Frustum.setFromProjectionMatrix (WebGL clip space, z from -1 to 1).

```ts
FrustumMath.fromProjectionMatrix(m: Matrix4): Frustum
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `m` | `Matrix4` | no | The view-projection matrix. |

**Returns** `Frustum`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromProjectionMatrix {
  doc "The frustum of a view-projection matrix (projection * inverse camera world), as three.js Frustum.setFromProjectionMatrix (WebGL clip space, z from -1 to 1)."
  host false
  param m Matrix4 "The view-projection matrix."
  returns Frustum
}
```

<sub>`api.ogt` line 528</sub>

<a id="member-FrustumMath-fromCamera"></a>

#### `FrustumMath.fromCamera`

The frustum og.Camera sees at an aspect ratio (width over height): its perspective projection (fov, near, far, zoom) or orthographic one (height, zoom), from its position and quaternion; shake, punch and kick, which are visual only, are left out.

```ts
FrustumMath.fromCamera(camera: Camera, aspect: number): Frustum
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `camera` | `Camera` | no | The og.Camera resource's value (world.resource(og.Camera)). |
| `aspect` | `number` | no | The view's width over its height. |

**Returns** `Frustum`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromCamera {
  doc "The frustum og.Camera sees at an aspect ratio (width over height): its perspective projection (fov, near, far, zoom) or orthographic one (height, zoom), from its position and quaternion; shake, punch and kick, which are visual only, are left out."
  host false
  param camera Camera "The og.Camera resource's value (world.resource(og.Camera))."
  param aspect f32 "The view's width over its height."
  returns Frustum
}
```

<sub>`api.ogt` line 534</sub>

<a id="member-FrustumMath-containsPoint"></a>

#### `FrustumMath.containsPoint`

Whether p is inside the frustum, as three.js Frustum.containsPoint.

```ts
FrustumMath.containsPoint(frustum: Frustum, p: Vector3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `frustum` | `Frustum` | no | The frustum. |
| `p` | `Vector3` | no | The point. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn containsPoint {
  doc "Whether p is inside the frustum, as three.js Frustum.containsPoint."
  host false
  param frustum Frustum "The frustum."
  param p Vector3 "The point."
  returns bool
}
```

<sub>`api.ogt` line 541</sub>

<a id="member-FrustumMath-intersectsBox"></a>

#### `FrustumMath.intersectsBox`

Whether the box is at least partly inside, as three.js Frustum.intersectsBox (conservative near the corners, as three.js).

```ts
FrustumMath.intersectsBox(frustum: Frustum, box: Box3): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `frustum` | `Frustum` | no | The frustum. |
| `box` | `Box3` | no | The box. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsBox {
  doc "Whether the box is at least partly inside, as three.js Frustum.intersectsBox (conservative near the corners, as three.js)."
  host false
  param frustum Frustum "The frustum."
  param box Box3 "The box."
  returns bool
}
```

<sub>`api.ogt` line 548</sub>

<a id="member-FrustumMath-intersectsSphere"></a>

#### `FrustumMath.intersectsSphere`

Whether the sphere is at least partly inside, as three.js Frustum.intersectsSphere.

```ts
FrustumMath.intersectsSphere(frustum: Frustum, sphere: Sphere): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `frustum` | `Frustum` | no | The frustum. |
| `sphere` | `Sphere` | no | The sphere. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectsSphere {
  doc "Whether the sphere is at least partly inside, as three.js Frustum.intersectsSphere."
  host false
  param frustum Frustum "The frustum."
  param sphere Sphere "The sphere."
  returns bool
}
```

<sub>`api.ogt` line 555</sub>

<a id="type-RayMath"></a>

### `RayMath` (interface)

Pure functions over Ray, as three.js Ray's methods. The value `ray`. Each intersection answers the nearest point at or after the origin, or null.

```ogt
interface RayMath {
  doc "Pure functions over Ray, as three.js Ray's methods. The value `ray`. Each intersection answers the nearest point at or after the origin, or null."
  # members follow, each quoted below
```

<sub>`api.ogt` line 564</sub>

<a id="member-RayMath-at"></a>

#### `RayMath.at`

The point t along the ray, as three.js Ray.at.

```ts
RayMath.at(ray: Ray, t: number): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `ray` | `Ray` | no | The ray. |
| `t` | `number` | no | The distance along the direction, in metres. |

**Returns** `Vector3`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn at {
  doc "The point t along the ray, as three.js Ray.at."
  host false
  param ray Ray "The ray."
  param t f32 "The distance along the direction, in metres."
  returns Vector3
}
```

<sub>`api.ogt` line 566</sub>

<a id="member-RayMath-distanceToPoint"></a>

#### `RayMath.distanceToPoint`

The distance from p to the nearest point of the ray, as three.js Ray.distanceToPoint.

```ts
RayMath.distanceToPoint(ray: Ray, p: Vector3): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `ray` | `Ray` | no | The ray. |
| `p` | `Vector3` | no | The point. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn distanceToPoint {
  doc "The distance from p to the nearest point of the ray, as three.js Ray.distanceToPoint."
  host false
  param ray Ray "The ray."
  param p Vector3 "The point."
  returns f32
}
```

<sub>`api.ogt` line 573</sub>

<a id="member-RayMath-intersectBox"></a>

#### `RayMath.intersectBox`

Where the ray enters the box (where it leaves, when it starts inside), as three.js Ray.intersectBox.

```ts
RayMath.intersectBox(ray: Ray, box: Box3): Vector3 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `ray` | `Ray` | no | The ray. |
| `box` | `Box3` | no | The box. |

**Returns** `Vector3 | null`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectBox {
  doc "Where the ray enters the box (where it leaves, when it starts inside), as three.js Ray.intersectBox."
  host false
  param ray Ray "The ray."
  param box Box3 "The box."
  returns Option.Vector3
}
```

<sub>`api.ogt` line 580</sub>

<a id="member-RayMath-intersectSphere"></a>

#### `RayMath.intersectSphere`

Where the ray meets the sphere (where it leaves, when it starts inside), as three.js Ray.intersectSphere.

```ts
RayMath.intersectSphere(ray: Ray, sphere: Sphere): Vector3 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `ray` | `Ray` | no | The ray. |
| `sphere` | `Sphere` | no | The sphere. |

**Returns** `Vector3 | null`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectSphere {
  doc "Where the ray meets the sphere (where it leaves, when it starts inside), as three.js Ray.intersectSphere."
  host false
  param ray Ray "The ray."
  param sphere Sphere "The sphere."
  returns Option.Vector3
}
```

<sub>`api.ogt` line 587</sub>

<a id="member-RayMath-intersectPlane"></a>

#### `RayMath.intersectPlane`

Where the ray meets the plane (its origin when it lies in the plane), as three.js Ray.intersectPlane; null when parallel or pointing away.

```ts
RayMath.intersectPlane(ray: Ray, plane: Plane): Vector3 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `ray` | `Ray` | no | The ray. |
| `plane` | `Plane` | no | The plane. |

**Returns** `Vector3 | null`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectPlane {
  doc "Where the ray meets the plane (its origin when it lies in the plane), as three.js Ray.intersectPlane; null when parallel or pointing away."
  host false
  param ray Ray "The ray."
  param plane Plane "The plane."
  returns Option.Vector3
}
```

<sub>`api.ogt` line 594</sub>

<a id="member-RayMath-intersectTriangle"></a>

#### `RayMath.intersectTriangle`

Where the ray meets the triangle a, b, c, as three.js Ray.intersectTriangle; with backfaceCulling a triangle facing away (clockwise from the origin) is missed.

```ts
RayMath.intersectTriangle(ray: Ray, a: Vector3, b: Vector3, c: Vector3, backfaceCulling?: boolean): Vector3 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `ray` | `Ray` | no | The ray. |
| `a` | `Vector3` | no | The first corner. |
| `b` | `Vector3` | no | The second corner. |
| `c` | `Vector3` | no | The third corner. |
| `backfaceCulling` | `boolean` | yes | Miss triangles facing away. Default `false`. |

**Returns** `Vector3 | null`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn intersectTriangle {
  doc "Where the ray meets the triangle a, b, c, as three.js Ray.intersectTriangle; with backfaceCulling a triangle facing away (clockwise from the origin) is missed."
  host false
  param ray Ray "The ray."
  param a Vector3 "The first corner."
  param b Vector3 "The second corner."
  param c Vector3 "The third corner."
  param backfaceCulling bool "Miss triangles facing away." {
    default false
  }
  returns Option.Vector3
}
```

<sub>`api.ogt` line 601</sub>

<a id="type-ColorMath"></a>

### `ColorMath` (interface)

Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. The value `color`: `color.fromHex(0xff8800)`. The same bits on every host (no Math.pow).

```ogt
interface ColorMath {
  doc "Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. The value `color`: `color.fromHex(0xff8800)`. The same bits on every host (no Math.pow)."
  # members follow, each quoted below
```

<sub>`api.ogt` line 615</sub>

<a id="member-ColorMath-fromHex"></a>

#### `ColorMath.fromHex`

The linear colour of an sRGB hex number, as new three.js Color(0xrrggbb).

```ts
ColorMath.fromHex(hex: number): Color
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `hex` | `number` | no | 0xrrggbb, sRGB. |

**Returns** `Color`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromHex {
  doc "The linear colour of an sRGB hex number, as new three.js Color(0xrrggbb)."
  host false
  param hex u32 "0xrrggbb, sRGB."
  returns Color
}
```

<sub>`api.ogt` line 617</sub>

<a id="member-ColorMath-toHex"></a>

#### `ColorMath.toHex`

The sRGB hex number of a linear colour, each channel clamped to 0..1 and rounded, as three.js Color.getHex.

```ts
ColorMath.toHex(c: Color): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `c` | `Color` | no | The colour. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn toHex {
  doc "The sRGB hex number of a linear colour, each channel clamped to 0..1 and rounded, as three.js Color.getHex."
  host false
  param c Color "The colour."
  returns u32
}
```

<sub>`api.ogt` line 623</sub>

<a id="member-ColorMath-lerp"></a>

#### `ColorMath.lerp`

a + (b - a) * t in linear RGB, as three.js Color.lerpColors.

```ts
ColorMath.lerp(a: Color, b: Color, t: number): Color
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `a` | `Color` | no | The value at t = 0. |
| `b` | `Color` | no | The value at t = 1. |
| `t` | `number` | no | The fraction. |

**Returns** `Color`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn lerp {
  doc "a + (b - a) * t in linear RGB, as three.js Color.lerpColors."
  host false
  param a Color "The value at t = 0."
  param b Color "The value at t = 1."
  param t f32 "The fraction."
  returns Color
}
```

<sub>`api.ogt` line 629</sub>

<a id="member-ColorMath-fromHsl"></a>

#### `ColorMath.fromHsl`

The linear colour of sRGB hue, saturation and lightness, as three.js Color.setHSL (hue wraps, saturation and lightness clamp to 0..1).

```ts
ColorMath.fromHsl(h: number, s: number, l: number): Color
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `h` | `number` | no | The hue, 0 to 1 around the wheel (0 red, 1/3 green, 2/3 blue). |
| `s` | `number` | no | The saturation, 0 to 1. |
| `l` | `number` | no | The lightness, 0 to 1. |

**Returns** `Color`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn fromHsl {
  doc "The linear colour of sRGB hue, saturation and lightness, as three.js Color.setHSL (hue wraps, saturation and lightness clamp to 0..1)."
  host false
  param h f32 "The hue, 0 to 1 around the wheel (0 red, 1/3 green, 2/3 blue)."
  param s f32 "The saturation, 0 to 1."
  param l f32 "The lightness, 0 to 1."
  returns Color
}
```

<sub>`api.ogt` line 637</sub>

## Types

<a id="type-Vector2"></a>

### `Vector2` (struct)

A 2D vector, as three.js Vector2.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |

```ogt
struct Vector2 {
  doc "A 2D vector, as three.js Vector2."
  field x f32 "The x component."
  field y f32 "The y component."
}
```

<sub>`api.ogt` line 42</sub>

<a id="type-Vector3"></a>

### `Vector3` (struct)

A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |
| `z` | `number` | `0` | — | — | — | The z component. |

```ogt
struct Vector3 {
  doc "A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js."
  field x f32 "The x component."
  field y f32 "The y component."
  field z f32 "The z component."
}
```

<sub>`api.ogt` line 48</sub>

<a id="type-Quaternion"></a>

### `Quaternion` (struct)

A unit quaternion rotation, as three.js Quaternion. The zero value is the identity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The x component. |
| `y` | `number` | `0` | — | — | — | The y component. |
| `z` | `number` | `0` | — | — | — | The z component. |
| `w` | `number` | `1` | — | — | — | The w (scalar) component. |

```ogt
struct Quaternion {
  doc "A unit quaternion rotation, as three.js Quaternion. The zero value is the identity."
  field x f32 "The x component."
  field y f32 "The y component."
  field z f32 "The z component."
  field w f32 "The w (scalar) component." {
    default 1
  }
}
```

<sub>`api.ogt` line 55</sub>

<a id="type-Matrix4"></a>

### `Matrix4` (struct)

A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `elements` | `[number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number]` | `[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]` | — | — | — | The elements, column-major, as three.js Matrix4.elements. |

```ogt
struct Matrix4 {
  doc "A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity."
  field elements Array.f32.16 "The elements, column-major, as three.js Matrix4.elements." {
    default 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1
  }
}
```

<sub>`api.ogt` line 65</sub>

<a id="type-Color"></a>

### `Color` (struct)

A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `r` | `number` | `0` | — | — | — | The red component. |
| `g` | `number` | `0` | — | — | — | The green component. |
| `b` | `number` | `0` | — | — | — | The blue component. |

```ogt
struct Color {
  doc "A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR."
  field r f32 "The red component."
  field g f32 "The green component."
  field b f32 "The blue component."
}
```

<sub>`api.ogt` line 72</sub>

<a id="type-Ray"></a>

### `Ray` (struct)

A ray, as three.js Ray.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the ray starts. |
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction the ray points in. |

```ogt
struct Ray {
  doc "A ray, as three.js Ray."
  field origin Vector3 "Where the ray starts."
  field direction Vector3 "The unit direction the ray points in."
}
```

<sub>`api.ogt` line 79</sub>

<a id="type-Box3"></a>

### `Box3` (struct)

An axis-aligned box, as three.js Box3. A box whose min exceeds its max on an axis is empty (box3.empty() makes one: min +Infinity, max -Infinity).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The lower corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The upper corner. |

```ogt
struct Box3 {
  doc "An axis-aligned box, as three.js Box3. A box whose min exceeds its max on an axis is empty (box3.empty() makes one: min +Infinity, max -Infinity)."
  field min Vector3 "The lower corner."
  field max Vector3 "The upper corner."
}
```

<sub>`api.ogt` line 85</sub>

<a id="type-Sphere"></a>

### `Sphere` (struct)

A sphere, as three.js Sphere. A negative radius is empty.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The centre. |
| `radius` | `number` | `0` | — | `m` | — | The radius. |

```ogt
struct Sphere {
  doc "A sphere, as three.js Sphere. A negative radius is empty."
  field center Vector3 "The centre."
  field radius f32 "The radius." {
    unit m
  }
}
```

<sub>`api.ogt` line 91</sub>

<a id="type-Plane"></a>

### `Plane` (struct)

A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit normal. |
| `constant` | `number` | `0` | — | `m` | — | The signed distance from the origin to the plane, negated along the normal (three.js Plane.constant). |

```ogt
struct Plane {
  doc "A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points."
  field normal Vector3 "The unit normal."
  field constant f32 "The signed distance from the origin to the plane, negated along the normal (three.js Plane.constant)." {
    unit m
  }
}
```

<sub>`api.ogt` line 99</sub>

<a id="type-Frustum"></a>

### `Frustum` (struct)

A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). frustum.fromCamera and frustum.fromProjectionMatrix make one.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `planes` | `readonly Plane[]` | `[]` | — | — | — | The six planes, normals inwards. |

```ogt
struct Frustum {
  doc "A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). frustum.fromCamera and frustum.fromProjectionMatrix make one."
  field planes List.Plane "The six planes, normals inwards."
}
```

<sub>`api.ogt` line 107</sub>
