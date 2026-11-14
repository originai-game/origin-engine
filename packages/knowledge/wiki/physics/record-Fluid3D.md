<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Fluid3D"></a>

### `og.Fluid3D` (component)

A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider and the terrain (they push nothing back) and stay inside `bounds` when set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes them. Fluids of one `group` layer by `density` and blend by `mixing`. Drawn as a screen-space surface that refracts and reflects.

See `physics §8`.

Token `og.Fluid3D`, registered under the schema name `og.Fluid3D`; its value type is `Fluid3D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `2000` | ≤ 262144 | — | — | How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it). The tick simulates at most 32768; a visual fluid (SIMULATION_VISUAL) up to 262144. |
| `size` | `Vector3` | `{ x: 2, y: 1, z: 2 }` | — | `m` | — | The spawn box, centred on the entity. |
| `bounds` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | The container around the entity (0 on an axis: open on that axis; colliders and the terrain still hold it). |
| `radius` | `number` | `0.2` | ≥ 0.02 | `m` | tunable | The interaction radius: particles closer than this push and drag each other. |
| `restDensity` | `number` | `10` | ≥ 0 | — | tunable | The density it settles at (neighbours within radius, weighted). |
| `stiffness` | `number` | `0.02` | ≥ 0 | — | tunable | The pressure's strength (above about 0.05 it jitters, then springs apart). |
| `nearStiffness` | `number` | `0.4` | ≥ 0 | — | tunable | The near pressure's strength (keeps particles apart, gives surface tension). |
| `viscosity` | `number` | `0.4` | 0 … 4 | — | tunable | Linear viscosity (it also calms the surface; honey above 1). |
| `group` | `number` | `0` | — | — | — | Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone); 0 keeps the fluid to itself. |
| `density` | `number` | `1` | 0.05 … 20 | — | tunable | How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one. |
| `mixing` | `number` | `0` | 0 … 1 | — | tunable | How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `substeps` | `number` | `2` | 1 … 8 | — | — | Simulation steps per tick (each costs a full pass over the particles). |
| `seed` | `number` | `0` | — | — | — | The seed of the spawn and emitter jitter. |
| `maxCount` | `number` | `8192` | 1 … 262144 | — | — | The most particles alive at once (the emitter stops there). |
| `emitRate` | `number` | `0` | 0 … 20000 | — | tunable | Particles the emitter adds per second (0: no emitter). |
| `emitOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the emitter is, from the entity (world axes). |
| `emitVelocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | The velocity emitted particles start with. |
| `emitRadius` | `number` | `0.1` | ≥ 0 | `m` | tunable | The half-size of the cube new particles appear in around the emitter. |
| `sinkOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sink's centre, from the entity (world axes). |
| `sinkSize` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sink box: particles that enter it are removed (0 0 0: no sink). |
| `color` | `Vector3` | `{ x: 0.2, y: 0.55, z: 0.9 }` | — | — | — | The fluid's linear colour (what light passing through it keeps). |
| `opacity` | `number` | `0.6` | 0 … 1 | — | — | How opaque a metre of it is (0: clear as glass, 1: opaque). |
| `reflectivity` | `number` | `1` | 0 … 2 | — | — | How much it reflects (scales the Fresnel term). |
| `refraction` | `number` | `1` | 0 … 4 | — | — | How far refraction bends what is seen through it. |
| `drawScale` | `number` | `0.6` | 0.1 … 2 | — | — | The drawn size of each particle (times radius). |
| `smoothing` | `number` | `1` | 0 … 2 | — | — | How much the surface is smoothed (0: beads, 1: one surface). |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |
| `simulation` | `number` | `0` | 0 … 1 | — | — | SIMULATION_GAMEPLAY 0 (in the tick, world state) or SIMULATION_VISUAL 1 (MLS-MPM on the GPU, for looks only): the tick holds none of its particles, so og.physics.fluid3dParticles answers none and buoyancy and groups ignore it. Without RenderCapabilities.visualSimulation it draws the spawn box still. See `physics §8`. |

```ogt
component Fluid3D {
  doc "A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider and the terrain (they push nothing back) and stay inside `bounds` when set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes them. Fluids of one `group` layer by `density` and blend by `mixing`. Drawn as a screen-space surface that refracts and reflects."
  guide "physics §8"
  field count u32 "How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it). The tick simulates at most 32768; a visual fluid (SIMULATION_VISUAL) up to 262144." {
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
  field restDensity f32 "The density it settles at (neighbours within radius, weighted)." {
    default 10
    min 0
    tunable true
  }
  field stiffness f32 "The pressure's strength (above about 0.05 it jitters, then springs apart)." {
    default 0.02
    min 0
    tunable true
  }
  field nearStiffness f32 "The near pressure's strength (keeps particles apart, gives surface tension)." {
    default 0.4
    min 0
    tunable true
  }
  field viscosity f32 "Linear viscosity (it also calms the surface; honey above 1)." {
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
  field seed u32 "The seed of the spawn and emitter jitter."
  field maxCount u32 "The most particles alive at once (the emitter stops there)." {
    default 8192
    min 1
    max 262144
  }
  field emitRate f32 "Particles the emitter adds per second (0: no emitter)." {
    min 0
    max 20000
    tunable true
  }
  field emitOffset Vector3 "Where the emitter is, from the entity (world axes)." {
    unit m
  }
  field emitVelocity Vector3 "The velocity emitted particles start with." {
    unit mps
    tunable true
  }
  field emitRadius f32 "The half-size of the cube new particles appear in around the emitter." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field sinkOffset Vector3 "The sink's centre, from the entity (world axes)." {
    unit m
  }
  field sinkSize Vector3 "The sink box: particles that enter it are removed (0 0 0: no sink)." {
    unit m
  }
  field color Vector3 "The fluid's linear colour (what light passing through it keeps)." {
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
  field simulation u32 "SIMULATION_GAMEPLAY 0 (in the tick, world state) or SIMULATION_VISUAL 1 (MLS-MPM on the GPU, for looks only): the tick holds none of its particles, so og.physics.fluid3dParticles answers none and buoyancy and groups ignore it. Without RenderCapabilities.visualSimulation it draws the spawn box still." {
    guide "physics §8"
    min 0
    max 1
  }
}
```

<sub>Module `physics` line 1433</sub>
