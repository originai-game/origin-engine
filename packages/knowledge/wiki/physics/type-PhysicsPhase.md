<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-PhysicsPhase"></a>

### `PhysicsPhase` (struct)

One phase of the last physics step, for og.physics.stats.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | The phase: gather, sync, joints, ragdolls, water, rapier, readout, controllers, soft, fluids, fluids3d or writeBack. See `physics §3`. |
| `ms` | `number` | `0` | — | `ms` | — | Wall-clock time it took (0 where the host has no clock); never world state. |
| `split` | `boolean` | `false` | — | — | — | Whether any of its work ran on the lanes pool: rapier's stages when the step went to the pool, the other phases when they split by bodies, contact pairs, ragdolls, soft bodies or particles. |

```ogt
struct PhysicsPhase {
  doc "One phase of the last physics step, for og.physics.stats."
  field name String "The phase: gather, sync, joints, ragdolls, water, rapier, readout, controllers, soft, fluids, fluids3d or writeBack." {
    guide "physics §3"
  }
  field ms f64 "Wall-clock time it took (0 where the host has no clock); never world state." {
    unit ms
  }
  field split bool "Whether any of its work ran on the lanes pool: rapier's stages when the step went to the pool, the other phases when they split by bodies, contact pairs, ragdolls, soft bodies or particles."
}
```

<sub>Module `physics` line 2087</sub>
