<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-cameraForward"></a>

### `og.transform.cameraForward`

The way a camera or a light with this rotation looks: its local -Z, as a unit vector.

See `facing`.

Also known as: view direction, look direction, camera front, three.js Camera.getWorldDirection

```ts
og.transform.cameraForward(quaternion: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The camera's or the light's rotation (og.Camera.quaternion). |

**Returns** `Vector3`.

Binding symbol `og_transform_camera_forward` · publish builds: kept

```ogt
fn cameraForward {
  doc "The way a camera or a light with this rotation looks: its local -Z, as a unit vector."
  aliases "view direction, look direction, camera front, three.js Camera.getWorldDirection"
  guide "facing"
  param quaternion Quaternion "The camera's or the light's rotation (og.Camera.quaternion)."
  returns Vector3
}
```

<sub>Module `transform` line 168</sub>
