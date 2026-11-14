<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-VehicleModel"></a>

### `VehicleModel` (enum)

How a vehicle's engine is modelled: Vehicle.model.

A string: one of the variants below. The zero value is `"arcade"`.

| Variant | Doc |
|---|---|
| `"arcade"` | The arcade car: engineForce, faded out towards maxSpeed. |
| `"drivetrain"` | An engine with a torque curve driving through a gearbox. |

```ogt
enum VehicleModel {
  doc "How a vehicle's engine is modelled: Vehicle.model."
  variant arcade "The arcade car: engineForce, faded out towards maxSpeed."
  variant drivetrain "An engine with a torque curve driving through a gearbox."
}
```

<sub>Module `physics` line 790</sub>
