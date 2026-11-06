<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `transform` module

Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.

- **TypeScript**: functions are the namespace `og.transform`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
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
| [`WorldOriginPlane`](transform/type-WorldOriginPlane.md) | enum |  | The axes og.WorldOrigin moves the world's zero along. |
| [`WorldOriginPlane()`](#column-WorldOriginPlane) | column kind |  | A WorldOriginPlane field: the variant's index. |
| [`og.WorldOrigin`](transform/record-WorldOrigin.md) | resource |  | Large worlds (R12): naming it keeps every place in the world near its zero. |
| [`og.OriginShifted`](transform/record-OriginShifted.md) | event |  | og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. |
| [`og.transform.compose`](transform/fn-compose.md) | function | `(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4` | A matrix from position, rotation and scale, as three.js Matrix4.compose. |
| [`og.transform.decompose`](transform/fn-decompose.md) | function | `(matrix: Matrix4): Transform` | Position, rotation and scale from a matrix, as three.js Matrix4.decompose. |
| [`og.transform.lookAt`](transform/fn-lookAt.md) | function | `(eye: Vector3, target: Vector3, up?: Vector3): Quaternion` | The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras. |
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

<sub>Module `transform` line 35</sub>
