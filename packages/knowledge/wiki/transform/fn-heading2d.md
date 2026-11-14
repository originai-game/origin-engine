<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-heading2d"></a>

### `og.transform.heading2d`

The 2D heading a thing with this rotation faces: the angle about +Z from +X towards +Y of its forward, so 0 faces +X and half pi faces +Y (up). The heading of a direction is math.atan2(y, x).

See `facing`.

Also known as: 2D facing angle, sprite rotation angle, Unity eulerAngles.z, Godot Node2D.rotation

```ts
og.transform.heading2d(quaternion: Quaternion): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The thing's rotation. |

**Returns** `number`: The heading in radians, from -π to π.

Binding symbol `og_transform_heading2d` · publish builds: kept

```ogt
fn heading2d {
  doc "The 2D heading a thing with this rotation faces: the angle about +Z from +X towards +Y of its forward, so 0 faces +X and half pi faces +Y (up). The heading of a direction is math.atan2(y, x)."
  aliases "2D facing angle, sprite rotation angle, Unity eulerAngles.z, Godot Node2D.rotation"
  guide "facing"
  param quaternion Quaternion "The thing's rotation."
  returns f32 "The heading in radians, from -π to π."
}
```

<sub>Module `transform` line 217</sub>
