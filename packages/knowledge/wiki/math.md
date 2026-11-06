<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `math` module

Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module math {
  doc "Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host."
  place top
  # declarations follow, each on its own page
```

<sub>Module `math` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Vector2`](math/type-Vector2.md) | struct |  | A 2D vector, as three.js Vector2. |
| [`Vector3`](math/type-Vector3.md) | struct |  | A 3D vector, as three.js Vector3. |
| [`Quaternion`](math/type-Quaternion.md) | struct |  | A unit quaternion rotation, as three.js Quaternion. |
| [`Matrix4`](math/type-Matrix4.md) | struct |  | A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. |
| [`Color`](math/type-Color.md) | struct |  | A linear RGB colour, as three.js Color with colour management on. |
| [`Ray`](math/type-Ray.md) | struct |  | A ray, as three.js Ray. |
| [`Box3`](math/type-Box3.md) | struct |  | An axis-aligned box, as three.js Box3. |
| [`Sphere`](math/type-Sphere.md) | struct |  | A sphere, as three.js Sphere. |
| [`Plane`](math/type-Plane.md) | struct |  | A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points. |
| [`Frustum`](math/type-Frustum.md) | struct |  | A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). |
| [`Vector3Math`](math/type-Vector3Math.md) | interface |  | Pure functions over Vector3, as three.js Vector3's methods of the same names: each returns a new value and changes none of its arguments. |
| [`Vector2Math`](math/type-Vector2Math.md) | interface |  | Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. |
| [`QuaternionMath`](math/type-QuaternionMath.md) | interface |  | Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. |
| [`Matrix4Math`](math/type-Matrix4Math.md) | interface |  | Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. |
| [`Box3Math`](math/type-Box3Math.md) | interface |  | Pure functions over Box3, as three.js Box3's methods, each returning a new value. |
| [`SphereMath`](math/type-SphereMath.md) | interface |  | Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. |
| [`PlaneMath`](math/type-PlaneMath.md) | interface |  | Pure functions over Plane, as three.js Plane's methods, each returning a new value. |
| [`FrustumMath`](math/type-FrustumMath.md) | interface |  | Pure functions over Frustum, as three.js Frustum's methods. |
| [`RayMath`](math/type-RayMath.md) | interface |  | Pure functions over Ray, as three.js Ray's methods. |
| [`ColorMath`](math/type-ColorMath.md) | interface |  | Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. |
| [`ScalarMath`](math/type-ScalarMath.md) | interface |  | JavaScript's Math functions whose results differ between JavaScript engines (Math.sin, Math.pow and the rest are only approximations, and V8, JavaScriptCore and SpiderMonkey round them differently), … |
| [`vec3`](#value-vec3) | value |  | Vector3's pure functions: `vec3.add(a, b)`, `vec3.normalize(v)`, `vec3.applyQuaternion(v, q)`. |
| [`vec2`](#value-vec2) | value |  | Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`. |
| [`quat`](#value-quat) | value |  | Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`. |
| [`mat4`](#value-mat4) | value |  | Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`. |
| [`box3`](#value-box3) | value |  | Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`. |
| [`sphere`](#value-sphere) | value |  | Sphere's pure functions: `sphere.containsPoint(s, p)`. |
| [`plane`](#value-plane) | value |  | Plane's pure functions: `plane.distanceToPoint(pl, p)`. |
| [`frustum`](#value-frustum) | value |  | Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`. |
| [`ray`](#value-ray) | value |  | Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`. |
| [`color`](#value-color) | value |  | Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`. |
| [`math`](#value-math) | value |  | Math's functions with the same bits on every host: `math.sin(a)`, `math.atan2(y, x)`, `math.pow(x, y)`. |

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

<sub>Module `math` line 677</sub>

<a id="value-vec2"></a>

### `vec2`

Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`.

```ts
const vec2: Vector2Math
```

```ogt
value vec2 Vector2Math "Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`."
```

<sub>Module `math` line 678</sub>

<a id="value-quat"></a>

### `quat`

Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`.

```ts
const quat: QuaternionMath
```

```ogt
value quat QuaternionMath "Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`."
```

<sub>Module `math` line 679</sub>

<a id="value-mat4"></a>

### `mat4`

Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`.

```ts
const mat4: Matrix4Math
```

```ogt
value mat4 Matrix4Math "Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`."
```

<sub>Module `math` line 680</sub>

<a id="value-box3"></a>

### `box3`

Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`.

```ts
const box3: Box3Math
```

```ogt
value box3 Box3Math "Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`."
```

<sub>Module `math` line 681</sub>

<a id="value-sphere"></a>

### `sphere`

Sphere's pure functions: `sphere.containsPoint(s, p)`.

```ts
const sphere: SphereMath
```

```ogt
value sphere SphereMath "Sphere's pure functions: `sphere.containsPoint(s, p)`."
```

<sub>Module `math` line 682</sub>

<a id="value-plane"></a>

### `plane`

Plane's pure functions: `plane.distanceToPoint(pl, p)`.

```ts
const plane: PlaneMath
```

```ogt
value plane PlaneMath "Plane's pure functions: `plane.distanceToPoint(pl, p)`."
```

<sub>Module `math` line 683</sub>

<a id="value-frustum"></a>

### `frustum`

Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`.

```ts
const frustum: FrustumMath
```

```ogt
value frustum FrustumMath "Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`."
```

<sub>Module `math` line 684</sub>

<a id="value-ray"></a>

### `ray`

Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`.

```ts
const ray: RayMath
```

```ogt
value ray RayMath "Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`."
```

<sub>Module `math` line 685</sub>

<a id="value-color"></a>

### `color`

Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`.

```ts
const color: ColorMath
```

```ogt
value color ColorMath "Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`."
```

<sub>Module `math` line 686</sub>

<a id="value-math"></a>

### `math`

Math's functions with the same bits on every host: `math.sin(a)`, `math.atan2(y, x)`, `math.pow(x, y)`.

```ts
const math: ScalarMath
```

```ogt
value math ScalarMath "Math's functions with the same bits on every host: `math.sin(a)`, `math.atan2(y, x)`, `math.pow(x, y)`."
```

<sub>Module `math` line 687</sub>
