<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

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

<sub>Module `math` line 75</sub>

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

<sub>Module `math` line 78</sub>

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

<sub>Module `math` line 85</sub>

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

<sub>Module `math` line 92</sub>

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

<sub>Module `math` line 99</sub>

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

<sub>Module `math` line 106</sub>

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

<sub>Module `math` line 113</sub>

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

<sub>Module `math` line 119</sub>

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

<sub>Module `math` line 125</sub>

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

<sub>Module `math` line 133</sub>

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

<sub>Module `math` line 140</sub>
