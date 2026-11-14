<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-fromHeading2d"></a>

### `og.transform.fromHeading2d`

The rotation of a 2D thing facing a heading: a turn about +Z from +X towards +Y, so 0 faces +X and half pi faces +Y (up).

See `facing`.

Also known as: 2D angle to rotation, rotate a sprite by an angle, Unity Quaternion.Euler(0, 0, z), Godot Node2D.rotation

```ts
og.transform.fromHeading2d(heading: number): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `heading` | `number` | no | The 2D heading in radians. |

**Returns** `Quaternion`.

Binding symbol `og_transform_from_heading2d` · publish builds: kept

```ogt
fn fromHeading2d {
  doc "The rotation of a 2D thing facing a heading: a turn about +Z from +X towards +Y, so 0 faces +X and half pi faces +Y (up)."
  aliases "2D angle to rotation, rotate a sprite by an angle, Unity Quaternion.Euler(0, 0, z), Godot Node2D.rotation"
  guide "facing"
  param heading f32 "The 2D heading in radians."
  returns Quaternion
}
```

<sub>Module `transform` line 225</sub>
