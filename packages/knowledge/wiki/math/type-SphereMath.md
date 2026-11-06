<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-SphereMath"></a>

### `SphereMath` (interface)

Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. The value `sphere`.

```ogt
interface SphereMath {
  doc "Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. The value `sphere`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 426</sub>

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

<sub>`api.ogt` line 428</sub>

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

<sub>`api.ogt` line 434</sub>

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

<sub>`api.ogt` line 441</sub>

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

<sub>`api.ogt` line 448</sub>

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

<sub>`api.ogt` line 455</sub>

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

<sub>`api.ogt` line 462</sub>

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

<sub>`api.ogt` line 468</sub>
