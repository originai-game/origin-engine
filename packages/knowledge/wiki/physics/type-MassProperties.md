<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-MassProperties"></a>

### `MassProperties` (struct)

A body's mass properties as computed by the solver.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mass` | `number` | `0` | — | `kg` | — | Total mass in kilograms (0 for static and kinematic bodies). |
| `centerOfMass` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | World-space centre of mass. |
| `inertia` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Local principal angular inertia (in 2D only z is non-zero). |

```ogt
struct MassProperties {
  doc "A body's mass properties as computed by the solver."
  field mass f32 "Total mass in kilograms (0 for static and kinematic bodies)." {
    unit kg
  }
  field centerOfMass Vector3 "World-space centre of mass." {
    unit m
  }
  field inertia Vector3 "Local principal angular inertia (in 2D only z is non-zero)."
}
```

<sub>Module `physics` line 2183</sub>
