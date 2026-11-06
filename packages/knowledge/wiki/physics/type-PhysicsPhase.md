<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-PhysicsPhase"></a>

### `PhysicsPhase` (struct)

One phase of the last physics step, for og.physics.stats.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | gather (reading the components), sync (bodies and static colliders into rapier), joints (joints and vehicles), ragdolls, water (buoyancy before the step), rapier (rapier's step, every substep), readout (touching pairs and bodies read back, the rest rule), controllers (characters and platformers), soft (ropes and cloths), fluids (2D), fluids3d or writeBack (components and events written back). |
| `ms` | `number` | `0` | — | `ms` | — | Wall-clock time it took (0 where the host has no clock); never world state. |
| `split` | `boolean` | `false` | — | — | — | Whether any of its work ran on the lanes pool: rapier's stages when the step went to the pool, the other phases when they split by bodies, contact pairs, ragdolls, soft bodies or particles. |

```ogt
struct PhysicsPhase {
  doc "One phase of the last physics step, for og.physics.stats."
  field name String "gather (reading the components), sync (bodies and static colliders into rapier), joints (joints and vehicles), ragdolls, water (buoyancy before the step), rapier (rapier's step, every substep), readout (touching pairs and bodies read back, the rest rule), controllers (characters and platformers), soft (ropes and cloths), fluids (2D), fluids3d or writeBack (components and events written back)."
  field ms f64 "Wall-clock time it took (0 where the host has no clock); never world state." {
    unit ms
  }
  field split bool "Whether any of its work ran on the lanes pool: rapier's stages when the step went to the pool, the other phases when they split by bodies, contact pairs, ragdolls, soft bodies or particles."
}
```

<sub>`api.ogt` line 10552</sub>
