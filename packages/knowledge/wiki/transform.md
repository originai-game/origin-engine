<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `transform` module

Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.

- **TypeScript**: functions are the namespace `og.transform`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module transform {
  doc "Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1479</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Transform`](#record-Transform) | component | Where an entity is, relative to its parent (or the world). |
| [`og.Parent`](#record-Parent) | component | The entity this one's Transform is relative to. Set by SpawnDesc.parent. |
| [`og.MatrixWorld`](#record-MatrixWorld) | component | The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld. |
| [`WorldOriginPlane`](#type-WorldOriginPlane) | enum | The axes og.WorldOrigin moves the world's zero along. |
| [`WorldOriginPlane()`](#column-WorldOriginPlane) | column kind | A WorldOriginPlane field: the variant's index. |
| [`og.WorldOrigin`](#record-WorldOrigin) | resource | Large worlds (R12): naming it keeps every place in the world near its zero. As each tick starts, before any system runs, when the focus is farther from the zero than `step` along one of the plane's axes, the whole world shifts by the multiple of `step` nearest the focus, so the focus is within half a step of the zero again: every Transform of an entity without a parent, and every field the API marks `worldspace`, moves by the same amount (a body, its contacts and warm starts, a path, a particle, a tween to a place, the camera), and nothing in the world sees a change but the numbers. Precision then holds however far the world goes, while the zero's own place is kept here, in f64. |
| [`og.OriginShifted`](#record-OriginShifted) | event | og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. Add it to the world positions a game keeps of its own. |
| [`og.transform.compose`](#fn-compose) | function | A matrix from position, rotation and scale, as three.js Matrix4.compose. |
| [`og.transform.decompose`](#fn-decompose) | function | Position, rotation and scale from a matrix, as three.js Matrix4.decompose. |
| [`og.transform.lookAt`](#fn-lookAt) | function | The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras. |
| [`og.transform.fromAxisAngle`](#fn-fromAxisAngle) | function | A rotation about an axis, as three.js Quaternion.setFromAxisAngle. |

## Functions

<a id="fn-compose"></a>

### `og.transform.compose`

A matrix from position, rotation and scale, as three.js Matrix4.compose.

```ts
og.transform.compose(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `position` | `Vector3` | no | The translation. |
| `quaternion` | `Quaternion` | no | The rotation. |
| `scale` | `Vector3` | no | The scale. |

**Returns** `Matrix4`.

Binding symbol `og_transform_compose` · publish builds: kept

```ogt
fn compose {
  doc "A matrix from position, rotation and scale, as three.js Matrix4.compose."
  param position Vector3 "The translation."
  param quaternion Quaternion "The rotation."
  param scale Vector3 "The scale."
  returns Matrix4
}
```

<sub>`api.ogt` line 1565</sub>

<a id="fn-decompose"></a>

### `og.transform.decompose`

Position, rotation and scale from a matrix, as three.js Matrix4.decompose.

```ts
og.transform.decompose(matrix: Matrix4): Transform
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `matrix` | `Matrix4` | no | An affine matrix. |

**Returns** `Transform`.

Binding symbol `og_transform_decompose` · publish builds: kept

```ogt
fn decompose {
  doc "Position, rotation and scale from a matrix, as three.js Matrix4.decompose."
  param matrix Matrix4 "An affine matrix."
  returns Transform
}
```

<sub>`api.ogt` line 1573</sub>

<a id="fn-lookAt"></a>

### `og.transform.lookAt`

The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras.

```ts
og.transform.lookAt(eye: Vector3, target: Vector3, up?: Vector3): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `eye` | `Vector3` | no | Where the viewer is. |
| `target` | `Vector3` | no | What it looks at. |
| `up` | `Vector3` | yes | The up direction; the zero vector means +Y. |

**Returns** `Quaternion`.

Binding symbol `og_transform_look_at` · publish builds: kept

```ogt
fn lookAt {
  doc "The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras."
  param eye Vector3 "Where the viewer is."
  param target Vector3 "What it looks at."
  param up Vector3 "The up direction; the zero vector means +Y." {
    optional true
  }
  returns Quaternion
}
```

<sub>`api.ogt` line 1579</sub>

<a id="fn-fromAxisAngle"></a>

### `og.transform.fromAxisAngle`

A rotation about an axis, as three.js Quaternion.setFromAxisAngle.

```ts
og.transform.fromAxisAngle(axis: Vector3, angle: number): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `axis` | `Vector3` | no | A unit axis. |
| `angle` | `number` | no | The angle in radians. |

**Returns** `Quaternion`.

Binding symbol `og_transform_from_axis_angle` · publish builds: kept

```ogt
fn fromAxisAngle {
  doc "A rotation about an axis, as three.js Quaternion.setFromAxisAngle."
  param axis Vector3 "A unit axis."
  param angle f32 "The angle in radians."
  returns Quaternion
}
```

<sub>`api.ogt` line 1589</sub>

## Components, resources and events

<a id="record-Transform"></a>

### `og.Transform` (component)

Where an entity is, relative to its parent (or the world).

Token `og.Transform`, registered under the schema name `og.Transform`; its value type is `Transform`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `root` | The position, as three.js Object3D.position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The rotation, as three.js Object3D.quaternion. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The scale, as three.js Object3D.scale. |

```ogt
component Transform {
  doc "Where an entity is, relative to its parent (or the world)."
  field position Vector3 "The position, as three.js Object3D.position." {
    worldspace root
    unit m
  }
  field quaternion Quaternion "The rotation, as three.js Object3D.quaternion."
  field scale Vector3 "The scale, as three.js Object3D.scale." {
    default 1 1 1
  }
}
```

<sub>`api.ogt` line 1482</sub>

<a id="record-Parent"></a>

### `og.Parent` (component)

The entity this one's Transform is relative to. Set by SpawnDesc.parent.

Token `og.Parent`, registered under the schema name `og.Parent`; its value type is `Parent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The parent. |

```ogt
component Parent {
  doc "The entity this one's Transform is relative to. Set by SpawnDesc.parent."
  field entity Entity "The parent."
}
```

<sub>`api.ogt` line 1494</sub>

<a id="record-MatrixWorld"></a>

### `og.MatrixWorld` (component)

The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld.

Token `og.MatrixWorld`, registered under the schema name `og.MatrixWorld`; its value type is `MatrixWorld`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `matrix` | `Matrix4` | `{ elements: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] }` | — | — | read only | The composed world matrix. |

```ogt
component MatrixWorld {
  doc "The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld."
  field matrix Matrix4 "The composed world matrix." {
    readonly true
  }
}
```

<sub>`api.ogt` line 1499</sub>

<a id="record-WorldOrigin"></a>

### `og.WorldOrigin` (resource)

Large worlds (R12): naming it keeps every place in the world near its zero. As each tick starts, before any system runs, when the focus is farther from the zero than `step` along one of the plane's axes, the whole world shifts by the multiple of `step` nearest the focus, so the focus is within half a step of the zero again: every Transform of an entity without a parent, and every field the API marks `worldspace`, moves by the same amount (a body, its contacts and warm starts, a path, a particle, a tween to a place, the camera), and nothing in the world sees a change but the numbers. Precision then holds however far the world goes, while the zero's own place is kept here, in f64.

The shift is world state, decided from world state alone (the focus's Transform, else the first og.StreamAnchor, else og.Camera.position), so a replay, a rollback, a restore and every peer of an og.net session shift on the same tick by the same amount; a session refuses the og.Camera fallback (each peer's camera is its own) and holds the zero until a focus or an anchor exists. A world that never names it keeps its hash and frames. Values a game keeps outside the marked fields (a world position in its own component, a key it set in an og.ai blackboard) are its own: OriginShifted says when to add its `delta` to them. 2D worlds shift too, on `xy`.

Token `og.WorldOrigin`, registered under the schema name `og.WorldOrigin`; its value type is `WorldOrigin`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `number` | `2048` | ≥ 64 | `m` | — | How far the focus may stray from the zero along an axis before the world shifts, and the grid the zero moves on (whole metres). |
| `plane` | `WorldOriginPlane` | `"xz"` | — | — | — | The axes the zero moves along: xz (the default), xyz, or xy for a 2D world. |
| `focus` | `Entity` | `0` | — | — | — | The entity the zero follows (a player, a vehicle; a root). 0: the first og.StreamAnchor by entity id, else og.Camera.position. |
| `off` | `boolean` | `false` | — | — | — | Hold the zero where it is. |
| `x` | `number` | `0` | — | `m` | read only | Written: where the zero is in the world, along x (a place's absolute position is its own plus this). |
| `y` | `number` | `0` | — | `m` | read only | Written: where the zero is, along y. |
| `z` | `number` | `0` | — | `m` | read only | Written: where the zero is, along z. |
| `shifts` | `number` | `0` | — | — | read only | Written: how many times the world has shifted. |

```ogt
resource WorldOrigin {
  doc "Large worlds (R12): naming it keeps every place in the world near its zero. As each tick starts, before any system runs, when the focus is farther from the zero than `step` along one of the plane's axes, the whole world shifts by the multiple of `step` nearest the focus, so the focus is within half a step of the zero again: every Transform of an entity without a parent, and every field the API marks `worldspace`, moves by the same amount (a body, its contacts and warm starts, a path, a particle, a tween to a place, the camera), and nothing in the world sees a change but the numbers. Precision then holds however far the world goes, while the zero's own place is kept here, in f64."
  doc "The shift is world state, decided from world state alone (the focus's Transform, else the first og.StreamAnchor, else og.Camera.position), so a replay, a rollback, a restore and every peer of an og.net session shift on the same tick by the same amount; a session refuses the og.Camera fallback (each peer's camera is its own) and holds the zero until a focus or an anchor exists. A world that never names it keeps its hash and frames. Values a game keeps outside the marked fields (a world position in its own component, a key it set in an og.ai blackboard) are its own: OriginShifted says when to add its `delta` to them. 2D worlds shift too, on `xy`."
  field step f32 "How far the focus may stray from the zero along an axis before the world shifts, and the grid the zero moves on (whole metres)." {
    default 2048
    min 64
    unit m
  }
  field plane WorldOriginPlane "The axes the zero moves along: xz (the default), xyz, or xy for a 2D world."
  field focus Entity "The entity the zero follows (a player, a vehicle; a root). 0: the first og.StreamAnchor by entity id, else og.Camera.position."
  field off bool "Hold the zero where it is."
  field x f64 "Written: where the zero is in the world, along x (a place's absolute position is its own plus this)." {
    readonly true
    unit m
  }
  field y f64 "Written: where the zero is, along y." {
    readonly true
    unit m
  }
  field z f64 "Written: where the zero is, along z." {
    readonly true
    unit m
  }
  field shifts u32 "Written: how many times the world has shifted." {
    readonly true
  }
}
```

<sub>`api.ogt` line 1520</sub>

<a id="record-OriginShifted"></a>

### `og.OriginShifted` (event)

og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. Add it to the world positions a game keeps of its own.

Token `og.OriginShifted`, registered under the schema name `og.OriginShifted`; its value type is `OriginShifted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `delta` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | How far every place moved (a whole multiple of og.WorldOrigin.step along each axis of its plane). |
| `x` | `number` | `0` | — | `m` | — | Where the zero is now in the world, along x (og.WorldOrigin.x). |
| `y` | `number` | `0` | — | `m` | — | Along y. |
| `z` | `number` | `0` | — | `m` | — | Along z. |

```ogt
event OriginShifted {
  doc "og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. Add it to the world positions a game keeps of its own."
  field delta Vector3 "How far every place moved (a whole multiple of og.WorldOrigin.step along each axis of its plane)." {
    worldspace none
    unit m
  }
  field x f64 "Where the zero is now in the world, along x (og.WorldOrigin.x)." {
    unit m
  }
  field y f64 "Along y." {
    unit m
  }
  field z f64 "Along z." {
    unit m
  }
}
```

<sub>`api.ogt` line 1548</sub>

## Types

<a id="type-WorldOriginPlane"></a>

### `WorldOriginPlane` (enum)

The axes og.WorldOrigin moves the world's zero along.

A string: one of the variants below. The zero value is `"xz"`.

| Variant | Doc |
|---|---|
| `"xz"` | x and z: the ground plane of a 3D world (heights stay absolute). |
| `"xyz"` | x, y and z: a world that is also deep or high (space, flight). |
| `"xy"` | x and y: a 2D world (its z, the drawing order, stays). |

```ogt
enum WorldOriginPlane {
  doc "The axes og.WorldOrigin moves the world's zero along."
  variant xz "x and z: the ground plane of a 3D world (heights stay absolute)."
  variant xyz "x, y and z: a world that is also deep or high (space, flight)."
  variant xy "x and y: a 2D world (its z, the drawing order, stays)."
}
```

<sub>`api.ogt` line 1506</sub>

## Column kinds

Each is a factory, `WorldOriginPlane(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-WorldOriginPlane"></a>`WorldOriginPlane` | `WorldOriginPlane` | `Uint32Array` | 1 | A WorldOriginPlane field: the variant's index. |

```ogt
column WorldOriginPlane {
  doc "A WorldOriginPlane field: the variant's index."
  array Uint32Array
  stride 1
  value WorldOriginPlane
}
```

<sub>`api.ogt` line 1513</sub>
