<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-forward2d"></a>

### `og.transform.forward2d`

The way a 2D thing with this rotation faces: its local +X, as a unit vector. og.transform.forward is the 3D one (+Z).

See `facing`.

Also known as: 2D facing direction, sprite forward, Unity transform.right in 2D, Godot Node2D transform.x

```ts
og.transform.forward2d(quaternion: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The thing's rotation. |

**Returns** `Vector3`.

Binding symbol `og_transform_forward2d` · publish builds: kept

```ogt
fn forward2d {
  doc "The way a 2D thing with this rotation faces: its local +X, as a unit vector. og.transform.forward is the 3D one (+Z)."
  aliases "2D facing direction, sprite forward, Unity transform.right in 2D, Godot Node2D transform.x"
  guide "facing"
  param quaternion Quaternion "The thing's rotation."
  returns Vector3
}
```

<sub>Module `transform` line 209</sub>
