<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Fluid3D"></a>

### `og.Fluid3D` (component)

Deterministic 3D Position Based Fluids with predicted-position neighbour hashing, symmetric mass-weighted density projection, artificial pressure, stable XSPH viscosity and optional vorticity confinement. Groups interpolate material properties by mix. Contacts exchange impulses with dynamic bodies. The emitter and sink use entity-local axes. Count, seed or size changes rebuild the lattice; lowering maxCount truncates in particle order even with emission off. World state; rendered as a refracting and reflecting surface.

See `physics §8`.

Token `og.Fluid3D`, registered under the schema name `og.Fluid3D`; its value type is `Fluid3D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `2000` | ≤ 262144 | — | — | How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it). The tick simulates at most 32768; a visual fluid (simulation visual) up to 262144. |
| `size` | `Vector3` | `{ x: 2, y: 1, z: 2 }` | — | `m` | — | The spawn box, centred on the entity. |
| `bounds` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | The container around the entity (0 on an axis: open on that axis; colliders and the terrain still hold it). |
| `radius` | `number` | `0.2` | ≥ 0.02 | `m` | tunable | The interaction radius: particles closer than this push and drag each other. |
| `restDensity` | `number` | `2` | ≥ 0 | — | tunable | Target poly6 count including self, multiplied by material density (0: pressure off). |
| `stiffness` | `number` | `1` | 0 … 1 | — | tunable | PBF density relaxation fraction per iteration (0: off). |
| `nearStiffness` | `number` | `0.001` | ≥ 0 | — | tunable | Dimensionless PBF artificial pressure s_corr coefficient (0: off). |
| `viscosity` | `number` | `0.4` | 0 … 4 | — | tunable | Stable, mass-conserving XSPH velocity smoothing rate. |
| `group` | `number` | `0` | — | — | — | Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone); 0 keeps the fluid to itself. |
| `density` | `number` | `1` | 0.05 … 20 | — | tunable | How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one. |
| `mixing` | `number` | `0` | 0 … 1 | — | tunable | How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `substeps` | `number` | `2` | 1 … 8 | — | — | Simulation steps per tick (each costs a full pass over the particles). |
| `solverIterations` | `number` | `3` | 0 … 64 | — | — | PBF density projection passes per substep (0: pressure off). |
| `vorticity` | `number` | `0` | ≥ 0 | — | tunable | Vorticity confinement acceleration coefficient (0: off). |
| `seed` | `number` | `0` | — | — | — | The seed of the spawn and emitter jitter. |
| `maxCount` | `number` | `8192` | 0 … 262144 | — | — | The most particles alive at once (the emitter stops there). |
| `emitRate` | `number` | `0` | 0 … 20000 | — | tunable | Particles the emitter adds per second (0: no emitter). |
| `emitOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The emitter's offset in entity-local axes. |
| `emitVelocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | Emitted velocity in entity-local axes, rotated into the world. |
| `emitRadius` | `number` | `0.1` | ≥ 0 | `m` | tunable | The half-size of the cube new particles appear in around the emitter. |
| `sinkOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sink's centre in entity-local axes; the box rotates with it. |
| `sinkSize` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sink box: particles that enter it are removed (0 0 0: no sink). |
| `color` | `Color` | `{ r: 0.2, g: 0.55, b: 0.9 }` | — | — | — | The fluid's colour (what light passing through it keeps). |
| `opacity` | `number` | `0.6` | 0 … 1 | — | — | How opaque a metre of it is (0: clear as glass, 1: opaque). |
| `reflectivity` | `number` | `1` | 0 … 2 | — | — | How much it reflects (scales the Fresnel term). |
| `refraction` | `number` | `1` | 0 … 4 | — | — | How far refraction bends what is seen through it. |
| `drawScale` | `number` | `0.6` | 0.1 … 2 | — | — | The drawn size of each particle (times radius). |
| `smoothing` | `number` | `1` | 0 … 2 | — | — | How much the surface is smoothed (0: beads, 1: one surface). |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |
| `simulation` | `Simulation` | `"gameplay"` | — | — | — | gameplay (in the tick, world state) or visual (MLS-MPM on the GPU, for looks only): the tick holds none of its particles, so og.physics.fluid3dParticles answers none and buoyancy and groups ignore it. Without RenderCapabilities.visualSimulation it draws the spawn box still. See `physics §8`. |

```ogt
component Fluid3D {
  doc "Deterministic 3D Position Based Fluids with predicted-position neighbour hashing, symmetric mass-weighted density projection, artificial pressure, stable XSPH viscosity and optional vorticity confinement. Groups interpolate material properties by mix. Contacts exchange impulses with dynamic bodies. The emitter and sink use entity-local axes. Count, seed or size changes rebuild the lattice; lowering maxCount truncates in particle order even with emission off. World state; rendered as a refracting and reflecting surface."
  guide "physics §8"
  field count u32 "How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it). The tick simulates at most 32768; a visual fluid (simulation visual) up to 262144." {
    default 2000
    max 262144
  }
  field size Vector3 "The spawn box, centred on the entity." {
    default 2 1 2
    unit m
  }
  field bounds Vector3 "The container around the entity (0 on an axis: open on that axis; colliders and the terrain still hold it)." {
    worldspace none
    unit m
  }
  field radius f32 "The interaction radius: particles closer than this push and drag each other." {
    default 0.2
    min 0.02
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
    default 0.4
    min 0
    max 4
    tunable true
  }
  field group u32 "Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone); 0 keeps the fluid to itself."
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
  field substeps u32 "Simulation steps per tick (each costs a full pass over the particles)." {
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
  field seed u32 "The seed of the spawn and emitter jitter."
  field maxCount u32 "The most particles alive at once (the emitter stops there)." {
    default 8192
    min 0
    max 262144
  }
  field emitRate f32 "Particles the emitter adds per second (0: no emitter)." {
    min 0
    max 20000
    tunable true
  }
  field emitOffset Vector3 "The emitter's offset in entity-local axes." {
    unit m
  }
  field emitVelocity Vector3 "Emitted velocity in entity-local axes, rotated into the world." {
    unit mps
    tunable true
  }
  field emitRadius f32 "The half-size of the cube new particles appear in around the emitter." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field sinkOffset Vector3 "The sink's centre in entity-local axes; the box rotates with it." {
    unit m
  }
  field sinkSize Vector3 "The sink box: particles that enter it are removed (0 0 0: no sink)." {
    unit m
  }
  field color Color "The fluid's colour (what light passing through it keeps)." {
    default 0.2 0.55 0.9
  }
  field opacity f32 "How opaque a metre of it is (0: clear as glass, 1: opaque)." {
    default 0.6
    min 0
    max 1
  }
  field reflectivity f32 "How much it reflects (scales the Fresnel term)." {
    default 1
    min 0
    max 2
  }
  field refraction f32 "How far refraction bends what is seen through it." {
    default 1
    min 0
    max 4
  }
  field drawScale f32 "The drawn size of each particle (times radius)." {
    default 0.6
    min 0.1
    max 2
  }
  field smoothing f32 "How much the surface is smoothed (0: beads, 1: one surface)." {
    default 1
    min 0
    max 2
  }
  field visible bool "Whether the renderer draws it." {
    default true
  }
  field simulation Simulation "gameplay (in the tick, world state) or visual (MLS-MPM on the GPU, for looks only): the tick holds none of its particles, so og.physics.fluid3dParticles answers none and buoyancy and groups ignore it. Without RenderCapabilities.visualSimulation it draws the spawn box still." {
    guide "physics §8"
  }
}
```

<sub>Module `physics` line 1719</sub>
