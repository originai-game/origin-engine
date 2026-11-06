<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Velocity"></a>

### `og.Velocity` (component)

Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing); a kinematic body moves by it each tick (never added, never written). Set it to launch or stop a body; push one with og.physics.applyForce, applyImpulse, applyTorque or applyTorqueImpulse.

Token `og.Velocity`, registered under the schema name `og.Velocity`; its value type is `Velocity`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `linear` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | The linear velocity (2D uses x and y). |
| `angular` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `radps` | tunable | The angular velocity about each axis (2D uses z). |

```ogt
component Velocity {
  doc "Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing); a kinematic body moves by it each tick (never added, never written). Set it to launch or stop a body; push one with og.physics.applyForce, applyImpulse, applyTorque or applyTorqueImpulse."
  field linear Vector3 "The linear velocity (2D uses x and y)." {
    unit mps
    tunable true
  }
  field angular Vector3 "The angular velocity about each axis (2D uses z)." {
    unit radps
    tunable true
  }
}
```

<sub>`api.ogt` line 8645</sub>
