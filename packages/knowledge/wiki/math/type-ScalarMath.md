<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-ScalarMath"></a>

### `ScalarMath` (interface)

JavaScript's Math functions whose results differ between JavaScript engines (Math.sin, Math.pow and the rest are only approximations, and V8, JavaScriptCore and SpiderMonkey round them differently), written with + - * / and sqrt only: the same bits on every host and every browser. The value `math`: `math.sin(a)`. A system whose result reaches the world (lockstep, replays, saves) uses these; `og check` names each Math call that should be one in a game with og.Lockstep.

Math.sqrt, Math.abs, Math.floor, Math.round, Math.min and Math.max are exact everywhere and stay as they are.

```ogt
interface ScalarMath {
  doc "JavaScript's Math functions whose results differ between JavaScript engines (Math.sin, Math.pow and the rest are only approximations, and V8, JavaScriptCore and SpiderMonkey round them differently), written with + - * / and sqrt only: the same bits on every host and every browser. The value `math`: `math.sin(a)`. A system whose result reaches the world (lockstep, replays, saves) uses these; `og check` names each Math call that should be one in a game with og.Lockstep."
  doc "Math.sqrt, Math.abs, Math.floor, Math.round, Math.min and Math.max are exact everywhere and stay as they are."
  # members follow, each quoted below
```

<sub>`api.ogt` line 651</sub>

<a id="member-ScalarMath-sin"></a>

#### `ScalarMath.sin`

The sine of x radians, as Math.sin.

```ts
ScalarMath.sin(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The angle, in radians. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn sin {
  doc "The sine of x radians, as Math.sin."
  host false
  param x f64 "The angle, in radians."
  returns f64
}
```

<sub>`api.ogt` line 654</sub>

<a id="member-ScalarMath-cos"></a>

#### `ScalarMath.cos`

The cosine of x radians, as Math.cos.

```ts
ScalarMath.cos(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The angle, in radians. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn cos {
  doc "The cosine of x radians, as Math.cos."
  host false
  param x f64 "The angle, in radians."
  returns f64
}
```

<sub>`api.ogt` line 660</sub>

<a id="member-ScalarMath-tan"></a>

#### `ScalarMath.tan`

The tangent of x radians, as Math.tan.

```ts
ScalarMath.tan(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The angle, in radians. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn tan {
  doc "The tangent of x radians, as Math.tan."
  host false
  param x f64 "The angle, in radians."
  returns f64
}
```

<sub>`api.ogt` line 666</sub>

<a id="member-ScalarMath-asin"></a>

#### `ScalarMath.asin`

The arcsine, as Math.asin: -pi/2 to pi/2, NaN outside -1 to 1.

```ts
ScalarMath.asin(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The sine. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn asin {
  doc "The arcsine, as Math.asin: -pi/2 to pi/2, NaN outside -1 to 1."
  host false
  param x f64 "The sine."
  returns f64
}
```

<sub>`api.ogt` line 672</sub>

<a id="member-ScalarMath-acos"></a>

#### `ScalarMath.acos`

The arccosine, as Math.acos: 0 to pi, NaN outside -1 to 1.

```ts
ScalarMath.acos(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The cosine. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn acos {
  doc "The arccosine, as Math.acos: 0 to pi, NaN outside -1 to 1."
  host false
  param x f64 "The cosine."
  returns f64
}
```

<sub>`api.ogt` line 678</sub>

<a id="member-ScalarMath-atan"></a>

#### `ScalarMath.atan`

The arctangent, as Math.atan: -pi/2 to pi/2.

```ts
ScalarMath.atan(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The tangent. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn atan {
  doc "The arctangent, as Math.atan: -pi/2 to pi/2."
  host false
  param x f64 "The tangent."
  returns f64
}
```

<sub>`api.ogt` line 684</sub>

<a id="member-ScalarMath-atan2"></a>

#### `ScalarMath.atan2`

The angle of the point (x, y) from +x, as Math.atan2: -pi to pi, 0 at the origin.

```ts
ScalarMath.atan2(y: number, x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `y` | `number` | no | The y coordinate. |
| `x` | `number` | no | The x coordinate. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn atan2 {
  doc "The angle of the point (x, y) from +x, as Math.atan2: -pi to pi, 0 at the origin."
  host false
  param y f64 "The y coordinate."
  param x f64 "The x coordinate."
  returns f64
}
```

<sub>`api.ogt` line 690</sub>

<a id="member-ScalarMath-exp"></a>

#### `ScalarMath.exp`

e to the x, as Math.exp.

```ts
ScalarMath.exp(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The exponent. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn exp {
  doc "e to the x, as Math.exp."
  host false
  param x f64 "The exponent."
  returns f64
}
```

<sub>`api.ogt` line 697</sub>

<a id="member-ScalarMath-log"></a>

#### `ScalarMath.log`

The natural logarithm, as Math.log: -Infinity at 0, NaN below.

```ts
ScalarMath.log(x: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The value. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn log {
  doc "The natural logarithm, as Math.log: -Infinity at 0, NaN below."
  host false
  param x f64 "The value."
  returns f64
}
```

<sub>`api.ogt` line 703</sub>

<a id="member-ScalarMath-pow"></a>

#### `ScalarMath.pow`

x to the y, as Math.pow and `**`: an integer y is multiplication by squaring (x * x for 2), any other y exp(y log x), NaN for a negative x.

```ts
ScalarMath.pow(x: number, y: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | The base. |
| `y` | `number` | no | The exponent. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn pow {
  doc "x to the y, as Math.pow and `**`: an integer y is multiplication by squaring (x * x for 2), any other y exp(y log x), NaN for a negative x."
  host false
  param x f64 "The base."
  param y f64 "The exponent."
  returns f64
}
```

<sub>`api.ogt` line 709</sub>
