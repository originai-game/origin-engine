<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-JointMotor"></a>

### `JointMotor` (enum)

What drives a joint: Joint.motor.

See `physics §1`.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No motor. |
| `"velocity"` | Drive towards `motorTarget` speed (rad/s or m/s). |
| `"position"` | Spring towards `motorTarget` angle or position (a servo). |

```ogt
enum JointMotor {
  doc "What drives a joint: Joint.motor."
  guide "physics §1"
  variant none "No motor."
  variant velocity "Drive towards `motorTarget` speed (rad/s or m/s)."
  variant position "Spring towards `motorTarget` angle or position (a servo)."
}
```

<sub>Module `physics` line 366</sub>
