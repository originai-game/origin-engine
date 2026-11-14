<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-face"></a>

### `og.transform.face`

The rotation that turns an object at a position to face a target: its +Z towards it, as three.js Object3D.lookAt for a mesh. A camera or a light uses og.transform.lookAt.

See `facing`.

Also known as: turn towards, rotate to face, look rotation, Unity Quaternion.LookRotation

```ts
og.transform.face(position: Vector3, target: Vector3, up?: Vector3): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `position` | `Vector3` | no | Where the object is. |
| `target` | `Vector3` | no | What it faces. |
| `up` | `Vector3` | yes | The up direction; the zero vector means +Y. |

**Returns** `Quaternion`.

Binding symbol `og_transform_face` · publish builds: kept

```ogt
fn face {
  doc "The rotation that turns an object at a position to face a target: its +Z towards it, as three.js Object3D.lookAt for a mesh. A camera or a light uses og.transform.lookAt."
  aliases "turn towards, rotate to face, look rotation, Unity Quaternion.LookRotation"
  guide "facing"
  param position Vector3 "Where the object is."
  param target Vector3 "What it faces."
  param up Vector3 "The up direction; the zero vector means +Y." {
    optional true
  }
  returns Quaternion
}
```

<sub>Module `transform` line 132</sub>
