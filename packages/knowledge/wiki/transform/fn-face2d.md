<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-face2d"></a>

### `og.transform.face2d`

The rotation that turns a 2D thing at a position to face a target: about +Z, its +X towards it, staying in the xy plane. og.transform.face is the 3D one, and turns a sprite out of the plane.

See `facing`.

Also known as: 2D look at, rotate a sprite towards, aim in 2D, Unity 2D LookAt, Godot Node2D.look_at

```ts
og.transform.face2d(position: Vector3, target: Vector3): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `position` | `Vector3` | no | Where the thing is (its xy). |
| `target` | `Vector3` | no | What it faces (its xy); the same point as the position gives no turn. |

**Returns** `Quaternion`.

Binding symbol `og_transform_face2d` · publish builds: kept

```ogt
fn face2d {
  doc "The rotation that turns a 2D thing at a position to face a target: about +Z, its +X towards it, staying in the xy plane. og.transform.face is the 3D one, and turns a sprite out of the plane."
  aliases "2D look at, rotate a sprite towards, aim in 2D, Unity 2D LookAt, Godot Node2D.look_at"
  guide "facing"
  param position Vector3 "Where the thing is (its xy)."
  param target Vector3 "What it faces (its xy); the same point as the position gives no turn."
  returns Quaternion
}
```

<sub>Module `transform` line 200</sub>
