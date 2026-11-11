<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Fluid2D"></a>

### `og.Fluid2D` (component)

A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: one thresholded surface of `color` on sprite layer `layer`.

Token `og.Fluid2D`, registered under the schema name `og.Fluid2D`; its value type is `Fluid2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `1000` | 1 … 8192 | — | — | How many particles (spawned when the component appears or count changes). |
| `size` | `Vector2` | `{ x: 4, y: 3 }` | — | `m` | — | The spawn box, centred on the entity. |
| `bounds` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `none` | The container around the entity (0 0: none; colliders still hold it). |
| `radius` | `number` | `0.18` | ≥ 0.01 | `m` | tunable | The interaction radius: particles closer than this push and drag each other. |
| `restDensity` | `number` | `1` | ≥ 0 | — | tunable | The density it settles at (neighbours within radius, weighted). |
| `stiffness` | `number` | `0.1` | ≥ 0 | — | tunable | The pressure's strength (above about 0.15 it gets springy, then unstable). |
| `nearStiffness` | `number` | `0.5` | ≥ 0 | — | tunable | The near pressure's strength (keeps particles apart, gives surface tension). |
| `viscosity` | `number` | `0.15` | 0 … 4 | — | tunable | Linear viscosity (honey above 1). |
| `group` | `number` | `0` | — | — | — | Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone), and are drawn as one surface whose colour follows the mix; 0 keeps the fluid to itself. |
| `density` | `number` | `1` | 0.05 … 20 | — | tunable | How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one. |
| `mixing` | `number` | `0` | 0 … 1 | — | tunable | How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `substeps` | `number` | `2` | 1 … 8 | — | — | Simulation steps per tick. |
| `seed` | `number` | `0` | — | — | — | The seed of the spawn jitter. |
| `color` | `Vector3` | `{ x: 0.15, y: 0.45, z: 0.95 }` | — | — | — | The surface's linear colour. |
| `opacity` | `number` | `0.9` | 0 … 1 | — | — | The surface's opacity. |
| `blobScale` | `number` | `1.6` | 0.1 … 4 | — | — | The drawn size of each particle's blob (times radius). |
| `threshold` | `number` | `0.45` | 0.01 … 4 | — | — | The field value where the surface starts (higher: thinner, separate drops). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it is drawn on. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Fluid2D {
  doc "A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: one thresholded surface of `color` on sprite layer `layer`."
  field count u32 "How many particles (spawned when the component appears or count changes)." {
    default 1000
    min 1
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
  field restDensity f32 "The density it settles at (neighbours within radius, weighted)." {
    default 1
    min 0
    tunable true
  }
  field stiffness f32 "The pressure's strength (above about 0.15 it gets springy, then unstable)." {
    default 0.1
    min 0
    tunable true
  }
  field nearStiffness f32 "The near pressure's strength (keeps particles apart, gives surface tension)." {
    default 0.5
    min 0
    tunable true
  }
  field viscosity f32 "Linear viscosity (honey above 1)." {
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
  field seed u32 "The seed of the spawn jitter."
  field color Vector3 "The surface's linear colour." {
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

<sub>Module `physics` line 1291</sub>
