<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Fluid2D"></a>

### `og.Fluid2D` (component)

Deterministic 2D Position Based Fluids in the xy plane: predicted-position neighbour hashing, symmetric mass-weighted density projection, artificial pressure, stable XSPH viscosity and optional vorticity confinement. Groups interpolate material properties by mix; solid contacts exchange impulses with dynamic bodies. Count, seed or size changes rebuild the spawn lattice. Particles and mix are world state; drawn as metaballs on layer.

Token `og.Fluid2D`, registered under the schema name `og.Fluid2D`; its value type is `Fluid2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `1000` | 0 … 8192 | — | — | How many particles (spawned when the component appears or count changes). |
| `size` | `Vector2` | `{ x: 4, y: 3 }` | — | `m` | — | The spawn box, centred on the entity. |
| `bounds` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `none` | The container around the entity (0 0: none; colliders still hold it). |
| `radius` | `number` | `0.18` | ≥ 0.01 | `m` | tunable | The interaction radius: particles closer than this push and drag each other. |
| `restDensity` | `number` | `2` | ≥ 0 | — | tunable | Target poly6 count including self, multiplied by material density (0: pressure off). |
| `stiffness` | `number` | `1` | 0 … 1 | — | tunable | PBF density relaxation fraction per iteration (0: off). |
| `nearStiffness` | `number` | `0.001` | ≥ 0 | — | tunable | Dimensionless PBF artificial pressure s_corr coefficient (0: off). |
| `viscosity` | `number` | `0.15` | 0 … 4 | — | tunable | Stable, mass-conserving XSPH velocity smoothing rate. |
| `group` | `number` | `0` | — | — | — | Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone), and are drawn as one surface whose colour follows the mix; 0 keeps the fluid to itself. |
| `density` | `number` | `1` | 0.05 … 20 | — | tunable | How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one. |
| `mixing` | `number` | `0` | 0 … 1 | — | tunable | How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `substeps` | `number` | `2` | 1 … 8 | — | — | Simulation steps per tick. |
| `solverIterations` | `number` | `3` | 0 … 64 | — | — | PBF density projection passes per substep (0: pressure off). |
| `vorticity` | `number` | `0` | ≥ 0 | — | tunable | Vorticity confinement acceleration coefficient (0: off). |
| `seed` | `number` | `0` | — | — | — | The seed of the spawn jitter. |
| `color` | `Color` | `{ r: 0.15, g: 0.45, b: 0.95 }` | — | — | — | The surface's colour. |
| `opacity` | `number` | `0.9` | 0 … 1 | — | — | The surface's opacity. |
| `blobScale` | `number` | `1.6` | 0.1 … 4 | — | — | The drawn size of each particle's blob (times radius). |
| `threshold` | `number` | `0.45` | 0.01 … 4 | — | — | The field value where the surface starts (higher: thinner, separate drops). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it is drawn on. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Fluid2D {
  doc "Deterministic 2D Position Based Fluids in the xy plane: predicted-position neighbour hashing, symmetric mass-weighted density projection, artificial pressure, stable XSPH viscosity and optional vorticity confinement. Groups interpolate material properties by mix; solid contacts exchange impulses with dynamic bodies. Count, seed or size changes rebuild the spawn lattice. Particles and mix are world state; drawn as metaballs on layer."
  field count u32 "How many particles (spawned when the component appears or count changes)." {
    default 1000
    min 0
    max 8192
  }
  field size Vector2 "The spawn box, centred on the entity." {
    default 4 3
    unit m
  }
  field bounds Vector2 "The container around the entity (0 0: none; colliders still hold it)." {
    worldspace none
    unit m
  }
  field radius f32 "The interaction radius: particles closer than this push and drag each other." {
    default 0.18
    min 0.01
    unit m
    tunable true
  }
  field restDensity f32 "Target poly6 count including self, multiplied by material density (0: pressure off)." {
    default 2
    min 0
    tunable true
  }
  field stiffness f32 "PBF density relaxation fraction per iteration (0: off)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field nearStiffness f32 "Dimensionless PBF artificial pressure s_corr coefficient (0: off)." {
    default 0.001
    min 0
    tunable true
  }
  field viscosity f32 "Stable, mass-conserving XSPH velocity smoothing rate." {
    default 0.15
    min 0
    max 4
    tunable true
  }
  field group u32 "Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone), and are drawn as one surface whose colour follows the mix; 0 keeps the fluid to itself."
  field density f32 "How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one." {
    default 1
    min 0.05
    max 20
    tunable true
  }
  field mixing f32 "How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out)." {
    min 0
    max 1
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity." {
    default 1
    min -10
    max 10
    tunable true
  }
  field substeps u32 "Simulation steps per tick." {
    default 2
    min 1
    max 8
  }
  field solverIterations u32 "PBF density projection passes per substep (0: pressure off)." {
    default 3
    min 0
    max 64
  }
  field vorticity f32 "Vorticity confinement acceleration coefficient (0: off)." {
    min 0
    tunable true
  }
  field seed u32 "The seed of the spawn jitter."
  field color Color "The surface's colour." {
    default 0.15 0.45 0.95
  }
  field opacity f32 "The surface's opacity." {
    default 0.9
    min 0
    max 1
  }
  field blobScale f32 "The drawn size of each particle's blob (times radius)." {
    default 1.6
    min 0.1
    max 4
  }
  field threshold f32 "The field value where the surface starts (higher: thinner, separate drops)." {
    default 0.45
    min 0.01
    max 4
  }
  field layer i32 "The sprite layer it is drawn on."
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>Module `physics` line 1716</sub>
