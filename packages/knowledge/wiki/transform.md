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

<sub>`api.ogt` line 475</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Transform`](#record-Transform) | component | Where an entity is, relative to its parent (or the world). |
| [`og.Parent`](#record-Parent) | component | The entity this one's Transform is relative to. Set by SpawnDesc.parent. |
| [`og.MatrixWorld`](#record-MatrixWorld) | component | The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld. |
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

<sub>`api.ogt` line 501</sub>

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

<sub>`api.ogt` line 509</sub>

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

<sub>`api.ogt` line 515</sub>

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

<sub>`api.ogt` line 525</sub>

## Components, resources and events

<a id="record-Transform"></a>

### `og.Transform` (component)

Where an entity is, relative to its parent (or the world).

Token `og.Transform`, registered under the schema name `og.Transform`; its value type is `Transform`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The position, as three.js Object3D.position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The rotation, as three.js Object3D.quaternion. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The scale, as three.js Object3D.scale. |

```ogt
component Transform {
  doc "Where an entity is, relative to its parent (or the world)."
  field position Vector3 "The position, as three.js Object3D.position." {
    unit m
  }
  field quaternion Quaternion "The rotation, as three.js Object3D.quaternion."
  field scale Vector3 "The scale, as three.js Object3D.scale." {
    default 1 1 1
  }
}
```

<sub>`api.ogt` line 478</sub>

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

<sub>`api.ogt` line 489</sub>

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

<sub>`api.ogt` line 494</sub>
