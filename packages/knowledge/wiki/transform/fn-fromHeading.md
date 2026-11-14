<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-fromHeading"></a>

### `og.transform.fromHeading`

The rotation of an upright object facing a heading: a turn about +Y from +Z towards +X, so 0 faces +Z and half pi faces +X.

See `facing`.

Also known as: yaw to rotation, rotation from an angle, Unity Quaternion.Euler(0, y, 0), three.js rotation.y

```ts
og.transform.fromHeading(heading: number): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `heading` | `number` | no | The heading in radians. |

**Returns** `Quaternion`.

Binding symbol `og_transform_from_heading` · publish builds: kept

```ogt
fn fromHeading {
  doc "The rotation of an upright object facing a heading: a turn about +Y from +Z towards +X, so 0 faces +Z and half pi faces +X."
  aliases "yaw to rotation, rotation from an angle, Unity Quaternion.Euler(0, y, 0), three.js rotation.y"
  guide "facing"
  param heading f32 "The heading in radians."
  returns Quaternion
}
```

<sub>Module `transform` line 192</sub>
