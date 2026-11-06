<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-FrustumMath"></a>

### `FrustumMath` (interface)

Pure functions over Frustum, as three.js Frustum's methods. The value `frustum`: `frustum.intersectsBox(frustum.fromCamera(camera, 16 / 9), box)` culls as the renderer would.

```ogt
interface FrustumMath {
  doc "Pure functions over Frustum, as three.js Frustum's methods. The value `frustum`: `frustum.intersectsBox(frustum.fromCamera(camera, 16 / 9), box)` culls as the renderer would."
  # members follow, each quoted below
```

<sub>`api.ogt` line 530</sub>

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

<sub>`api.ogt` line 532</sub>

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

<sub>`api.ogt` line 538</sub>

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

<sub>`api.ogt` line 545</sub>

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

<sub>`api.ogt` line 552</sub>

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

<sub>`api.ogt` line 559</sub>
