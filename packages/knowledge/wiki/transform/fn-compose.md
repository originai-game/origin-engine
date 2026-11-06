<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

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
