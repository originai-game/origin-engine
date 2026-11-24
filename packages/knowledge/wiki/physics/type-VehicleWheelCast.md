<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-VehicleWheelCast"></a>

### `VehicleWheelCast` (enum)

The 3D wheel contact cast shape.

A string: one of the variants below. The zero value is `"ray"`.

| Variant | Doc |
|---|---|
| `"ray"` | Ray through full droop plus wheel radius. |
| `"sphere"` | Sphere sweep through full droop. |
| `"cylinder"` | Steered cylinder sweep through full droop. |

```ogt
enum VehicleWheelCast {
  doc "The 3D wheel contact cast shape."
  variant ray "Ray through full droop plus wheel radius."
  variant sphere "Sphere sweep through full droop."
  variant cylinder "Steered cylinder sweep through full droop."
}
```

<sub>Module `physics` line 1260</sub>
