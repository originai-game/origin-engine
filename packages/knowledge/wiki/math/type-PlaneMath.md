<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-PlaneMath"></a>

### `PlaneMath` (interface)

Pure functions over Plane, as three.js Plane's methods, each returning a new value. The value `plane`.

```ogt
interface PlaneMath {
  doc "Pure functions over Plane, as three.js Plane's methods, each returning a new value. The value `plane`."
  # members follow, each quoted below
```

<sub>Module `math` line 436</sub>

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

<sub>Module `math` line 438</sub>

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

<sub>Module `math` line 445</sub>

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

<sub>Module `math` line 453</sub>

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

<sub>Module `math` line 459</sub>

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

<sub>Module `math` line 466</sub>

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

<sub>Module `math` line 473</sub>

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

<sub>Module `math` line 480</sub>
