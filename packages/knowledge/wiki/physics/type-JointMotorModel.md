<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-JointMotorModel"></a>

### `JointMotorModel` (enum)

How a joint motor interprets stiffness and damping: Joint.motorModel.

A string: one of the variants below. The zero value is `"acceleration"`.

| Variant | Doc |
|---|---|
| `"acceleration"` | Stiffness and damping act as acceleration (mass-independent). |
| `"force"` | Stiffness and damping act as force or torque (N/m or N·m/rad). |

```ogt
enum JointMotorModel {
  doc "How a joint motor interprets stiffness and damping: Joint.motorModel."
  variant acceleration "Stiffness and damping act as acceleration (mass-independent)."
  variant force "Stiffness and damping act as force or torque (N/m or N·m/rad)."
}
```

<sub>Module `physics` line 427</sub>
