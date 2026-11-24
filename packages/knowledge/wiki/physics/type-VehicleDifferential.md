<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-VehicleDifferential"></a>

### `VehicleDifferential` (enum)

Driven-wheel angular coupling.

A string: one of the variants below. The zero value is `"open"`.

| Variant | Doc |
|---|---|
| `"open"` | Equal torque, independent wheel speed. |
| `"lsd"` | Limited-slip coupling at differentialStrength. |
| `"locked"` | Locked angular velocities. |

```ogt
enum VehicleDifferential {
  doc "Driven-wheel angular coupling."
  variant open "Equal torque, independent wheel speed."
  variant lsd "Limited-slip coupling at differentialStrength."
  variant locked "Locked angular velocities."
}
```

<sub>Module `physics` line 1272</sub>
