<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-RayMath"></a>

### `RayMath` (interface)

Pure functions over Ray, as three.js Ray's methods. The value `ray`. Each intersection answers the nearest point at or after the origin, or null.

```ogt
interface RayMath {
  doc "Pure functions over Ray, as three.js Ray's methods. The value `ray`. Each intersection answers the nearest point at or after the origin, or null."
  # members follow, each quoted below
```

<sub>`api.ogt` line 568</sub>

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

<sub>`api.ogt` line 570</sub>

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

<sub>`api.ogt` line 577</sub>

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

<sub>`api.ogt` line 584</sub>

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

<sub>`api.ogt` line 591</sub>

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

<sub>`api.ogt` line 598</sub>

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

<sub>`api.ogt` line 605</sub>
