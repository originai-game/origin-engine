<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-lookAt"></a>

### `og.transform.lookAt`

The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras.

Also known as: face, aim at, rotate towards, Unity Transform.LookAt, Godot look_at, three.js lookAt

```ts
og.transform.lookAt(eye: Vector3, target: Vector3, up?: Vector3): Quaternion
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `eye` | `Vector3` | no | Where the viewer is. |
| `target` | `Vector3` | no | What it looks at. |
| `up` | `Vector3` | yes | The up direction; the zero vector means +Y. |

**Returns** `Quaternion`.

Binding symbol `og_transform_look_at` · publish builds: kept

```ogt
fn lookAt {
  doc "The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras."
  aliases "face, aim at, rotate towards, Unity Transform.LookAt, Godot look_at, three.js lookAt"
  param eye Vector3 "Where the viewer is."
  param target Vector3 "What it looks at."
  param up Vector3 "The up direction; the zero vector means +Y." {
    optional true
  }
  returns Quaternion
}
```

<sub>Module `transform` line 102</sub>
