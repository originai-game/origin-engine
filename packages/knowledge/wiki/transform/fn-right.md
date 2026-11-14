<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-right"></a>

### `og.transform.right`

An object's right: its local -X, as a unit vector (forward × up). A camera's or a light's is og.transform.cameraRight.

See `facing`.

Also known as: strafe direction, side vector, Unity transform.right

```ts
og.transform.right(quaternion: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The object's rotation. |

**Returns** `Vector3`.

Binding symbol `og_transform_right` · publish builds: kept

```ogt
fn right {
  doc "An object's right: its local -X, as a unit vector (forward × up). A camera's or a light's is og.transform.cameraRight."
  aliases "strafe direction, side vector, Unity transform.right"
  guide "facing"
  param quaternion Quaternion "The object's rotation."
  returns Vector3
}
```

<sub>Module `transform` line 152</sub>
