<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-VehicleWheel"></a>

### `og.VehicleWheel` (component)

A wheel of the og.Vehicle it is a child of (down its og.Parent chain), at its og.Transform relative to the chassis: the hard point its suspension hangs from. A vehicle with og.VehicleWheel children (up to 16, in entity order) has those wheels in place of its four implicit ones, and og.physics.vehicleWheels lists them in that order: six-wheeled trucks, trikes, rear-wheel steering. 3D only; the 2D vehicle keeps its two wheels.

Token `og.VehicleWheel`, registered under the schema name `og.VehicleWheel`; its value type is `VehicleWheel`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `0.4` | ≥ 0.01 | `m` | tunable | The wheel's radius. |
| `suspensionRest` | `number` | `0.35` | ≥ 0 | `m` | tunable | The suspension's length at rest, down from the wheel's position. |
| `suspensionTravel` | `number` | `0.25` | ≥ 0 | `m` | tunable | How far the suspension compresses or extends from rest. |
| `suspensionStiffness` | `number` | `30` | ≥ 0 | — | tunable | The suspension's stiffness. |
| `suspensionDamping` | `number` | `4` | ≥ 0 | — | tunable | The suspension's damping. |
| `grip` | `number` | `2` | ≥ 0 | — | tunable | The tyre's grip, as og.Vehicle's `grip`. |
| `steer` | `number` | `0` | -1 … 1 | — | tunable | How much of the vehicle's steering it takes: 1 a front wheel, 0 none, negative steers the other way (rear-wheel steering). |
| `drive` | `boolean` | `false` | — | — | — | Whether the engine drives it (og.Vehicle.drive is then unused). |
| `brake` | `boolean` | `true` | — | — | — | Whether the brake brakes it. |
| `handbrake` | `boolean` | `false` | — | — | — | Whether the handbrake brakes it, and driftGrip applies to it. |

```ogt
component VehicleWheel {
  doc "A wheel of the og.Vehicle it is a child of (down its og.Parent chain), at its og.Transform relative to the chassis: the hard point its suspension hangs from. A vehicle with og.VehicleWheel children (up to 16, in entity order) has those wheels in place of its four implicit ones, and og.physics.vehicleWheels lists them in that order: six-wheeled trucks, trikes, rear-wheel steering. 3D only; the 2D vehicle keeps its two wheels."
  field radius f32 "The wheel's radius." {
    default 0.4
    min 0.01
    unit m
    tunable true
  }
  field suspensionRest f32 "The suspension's length at rest, down from the wheel's position." {
    default 0.35
    min 0
    unit m
    tunable true
  }
  field suspensionTravel f32 "How far the suspension compresses or extends from rest." {
    default 0.25
    min 0
    unit m
    tunable true
  }
  field suspensionStiffness f32 "The suspension's stiffness." {
    default 30
    min 0
    tunable true
  }
  field suspensionDamping f32 "The suspension's damping." {
    default 4
    min 0
    tunable true
  }
  field grip f32 "The tyre's grip, as og.Vehicle's `grip`." {
    default 2
    min 0
    tunable true
  }
  field steer f32 "How much of the vehicle's steering it takes: 1 a front wheel, 0 none, negative steers the other way (rear-wheel steering)." {
    min -1
    max 1
    tunable true
  }
  field drive bool "Whether the engine drives it (og.Vehicle.drive is then unused)."
  field brake bool "Whether the brake brakes it." {
    default true
  }
  field handbrake bool "Whether the handbrake brakes it, and driftGrip applies to it."
}
```

<sub>Module `physics` line 996</sub>
