<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-heading"></a>

### `og.transform.heading`

The heading an object with this rotation faces: the angle about +Y from +Z towards +X of its forward, so 0 faces +Z and half pi faces +X. The heading of a direction is math.atan2(x, z).

See `facing`.

Also known as: yaw, facing angle, compass direction, Unity eulerAngles.y, three.js rotation.y

```ts
og.transform.heading(quaternion: Quaternion): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The object's rotation. |

**Returns** `number`: The heading in radians, from -π to π.

Binding symbol `og_transform_heading` · publish builds: kept

```ogt
fn heading {
  doc "The heading an object with this rotation faces: the angle about +Y from +Z towards +X of its forward, so 0 faces +Z and half pi faces +X. The heading of a direction is math.atan2(x, z)."
  aliases "yaw, facing angle, compass direction, Unity eulerAngles.y, three.js rotation.y"
  guide "facing"
  param quaternion Quaternion "The object's rotation."
  returns f32 "The heading in radians, from -π to π."
}
```

<sub>Module `transform` line 184</sub>
