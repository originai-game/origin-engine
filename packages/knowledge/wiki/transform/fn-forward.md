<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-forward"></a>

### `og.transform.forward`

The way an object with this rotation faces: its local +Z, as a unit vector. A camera's or a light's is og.transform.cameraForward.

See `facing`.

Also known as: front, facing direction, heading vector, Unity transform.forward, three.js getWorldDirection

```ts
og.transform.forward(quaternion: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The object's rotation. |

**Returns** `Vector3`.

Binding symbol `og_transform_forward` · publish builds: kept

```ogt
fn forward {
  doc "The way an object with this rotation faces: its local +Z, as a unit vector. A camera's or a light's is og.transform.cameraForward."
  aliases "front, facing direction, heading vector, Unity transform.forward, three.js getWorldDirection"
  guide "facing"
  param quaternion Quaternion "The object's rotation."
  returns Vector3
}
```

<sub>Module `transform` line 144</sub>
