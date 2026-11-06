<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-QuaternionMath"></a>

### `QuaternionMath` (interface)

Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. The value `quat`: `quat.multiply(a, b)`. The same bits on every host (trigonometry from + - * / and sqrt).

```ogt
interface QuaternionMath {
  doc "Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. The value `quat`: `quat.multiply(a, b)`. The same bits on every host (trigonometry from + - * / and sqrt)."
  # members follow, each quoted below
```

<sub>Module `math` line 223</sub>

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

<sub>Module `math` line 225</sub>

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

<sub>Module `math` line 232</sub>

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

<sub>Module `math` line 240</sub>

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

<sub>Module `math` line 249</sub>

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

<sub>Module `math` line 255</sub>
