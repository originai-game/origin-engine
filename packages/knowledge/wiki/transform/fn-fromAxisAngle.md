<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

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

<sub>`api.ogt` line 1595</sub>
