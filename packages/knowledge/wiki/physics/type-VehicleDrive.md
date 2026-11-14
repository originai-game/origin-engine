<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-VehicleDrive"></a>

### `VehicleDrive` (enum)

Which wheels the engine drives: Vehicle.drive.

A string: one of the variants below. The zero value is `"rear"`.

| Variant | Doc |
|---|---|
| `"rear"` | The rear wheels drive. |
| `"front"` | The front wheels drive. |
| `"all"` | Every wheel drives. |

```ogt
enum VehicleDrive {
  doc "Which wheels the engine drives: Vehicle.drive."
  variant rear "The rear wheels drive."
  variant front "The front wheels drive."
  variant all "Every wheel drives."
}
```

<sub>Module `physics` line 776</sub>
