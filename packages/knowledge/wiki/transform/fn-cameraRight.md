<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-cameraRight"></a>

### `og.transform.cameraRight`

A camera's or a light's right: its local +X, the right of the screen, as a unit vector.

See `facing`.

Also known as: screen right, camera strafe direction

```ts
og.transform.cameraRight(quaternion: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The camera's or the light's rotation (og.Camera.quaternion). |

**Returns** `Vector3`.

Binding symbol `og_transform_camera_right` · publish builds: kept

```ogt
fn cameraRight {
  doc "A camera's or a light's right: its local +X, the right of the screen, as a unit vector."
  aliases "screen right, camera strafe direction"
  guide "facing"
  param quaternion Quaternion "The camera's or the light's rotation (og.Camera.quaternion)."
  returns Vector3
}
```

<sub>Module `transform` line 157</sub>
