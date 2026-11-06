<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Vector2Math"></a>

### `Vector2Math` (interface)

Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. The value `vec2`: `vec2.add(a, b)`. The same bits on every host.

```ogt
interface Vector2Math {
  doc "Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. The value `vec2`: `vec2.add(a, b)`. The same bits on every host."
  # members follow, each quoted below
```

<sub>Module `math` line 149</sub>

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

<sub>Module `math` line 151</sub>

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

<sub>Module `math` line 158</sub>

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

<sub>Module `math` line 165</sub>

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

<sub>Module `math` line 172</sub>

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

<sub>Module `math` line 179</sub>

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

<sub>Module `math` line 186</sub>

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

<sub>Module `math` line 192</sub>

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

<sub>Module `math` line 198</sub>

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

<sub>Module `math` line 206</sub>

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

<sub>Module `math` line 213</sub>
