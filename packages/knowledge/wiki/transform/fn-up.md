<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-up"></a>

### `og.transform.up`

An object's, a camera's or a light's up: its local +Y, as a unit vector.

See `facing`.

Also known as: up vector, Unity transform.up

```ts
og.transform.up(quaternion: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `quaternion` | `Quaternion` | no | The rotation. |

**Returns** `Vector3`.

Binding symbol `og_transform_up` · publish builds: kept

```ogt
fn up {
  doc "An object's, a camera's or a light's up: its local +Y, as a unit vector."
  aliases "up vector, Unity transform.up"
  guide "facing"
  param quaternion Quaternion "The rotation."
  returns Vector3
}
```

<sub>Module `transform` line 141</sub>
