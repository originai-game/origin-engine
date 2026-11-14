<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `transform` module

Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.

- **TypeScript**: functions are the namespace `og.transform`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.Transform`, `og.MatrixWorld`, `og.transform.lookAt`, `og.transform.fromAxisAngle`; 14 more are advanced (`og api show transform --all`).
- **Index**: [every module and guide](index.md).

```ogt
module transform {
  doc "Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain."
  # declarations follow, each on its own page
```

<sub>Module `transform` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.Transform`](transform/record-Transform.md) | component |  | Where an entity is, relative to its parent (or the world). |
| [`og.Parent`](transform/record-Parent.md) | component |  | The entity this one's Transform is relative to. |
| [`og.MatrixWorld`](transform/record-MatrixWorld.md) | component |  | The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld. |
| [`og.Volume`](transform/record-Volume.md) | component |  | The shape of a region of the world: a box along the entity's axes, or a sphere about it, with a fade past its surface. |
| [`WorldOriginPlane`](transform/type-WorldOriginPlane.md) | enum |  | The axes og.WorldOrigin moves the world's zero along. |
| [`WorldOriginPlane()`](#column-WorldOriginPlane) | column kind |  | A WorldOriginPlane field: the variant's index. |
| [`og.WorldOrigin`](transform/record-WorldOrigin.md) | resource |  | Large worlds: naming it keeps every place near the zero. |
| [`og.OriginShifted`](transform/record-OriginShifted.md) | event |  | og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. |
| [`og.transform.compose`](transform/fn-compose.md) | function | `(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4` | A matrix from position, rotation and scale, as three.js Matrix4.compose. |
| [`og.transform.decompose`](transform/fn-decompose.md) | function | `(matrix: Matrix4): Transform` | Position, rotation and scale from a matrix, as three.js Matrix4.decompose. |
| [`og.transform.lookAt`](transform/fn-lookAt.md) | function | `(eye: Vector3, target: Vector3, up?: Vector3): Quaternion` | The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras and lights. |
| [`og.transform.face`](transform/fn-face.md) | function | `(position: Vector3, target: Vector3, up?: Vector3): Quaternion` | The rotation that turns an object at a position to face a target: its +Z towards it, as three.js Object3D.lookAt for a mesh. |
| [`og.transform.forward`](transform/fn-forward.md) | function | `(quaternion: Quaternion): Vector3` | The way an object with this rotation faces: its local +Z, as a unit vector. |
| [`og.transform.right`](transform/fn-right.md) | function | `(quaternion: Quaternion): Vector3` | An object's right: its local -X, as a unit vector (forward × up). |
| [`og.transform.up`](transform/fn-up.md) | function | `(quaternion: Quaternion): Vector3` | An object's, a camera's or a light's up: its local +Y, as a unit vector. |
| [`og.transform.cameraForward`](transform/fn-cameraForward.md) | function | `(quaternion: Quaternion): Vector3` | The way a camera or a light with this rotation looks: its local -Z, as a unit vector. |
| [`og.transform.cameraRight`](transform/fn-cameraRight.md) | function | `(quaternion: Quaternion): Vector3` | A camera's or a light's right: its local +X, the right of the screen, as a unit vector. |
| [`og.transform.fromAxisAngle`](transform/fn-fromAxisAngle.md) | function | `(axis: Vector3, angle: number): Quaternion` | A rotation about an axis, as three.js Quaternion.setFromAxisAngle. |

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

<sub>Module `transform` line 55</sub>
