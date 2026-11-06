<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Matrix4Math"></a>

### `Matrix4Math` (interface)

Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. The value `mat4`: `mat4.multiply(a, b)`.

```ogt
interface Matrix4Math {
  doc "Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. The value `mat4`: `mat4.multiply(a, b)`."
  # members follow, each quoted below
```

<sub>Module `math` line 264</sub>

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

<sub>Module `math` line 266</sub>

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

<sub>Module `math` line 273</sub>

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

<sub>Module `math` line 279</sub>

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

<sub>Module `math` line 286</sub>

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

<sub>Module `math` line 293</sub>
