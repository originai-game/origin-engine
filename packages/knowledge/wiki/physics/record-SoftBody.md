<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-SoftBody"></a>

### `og.SoftBody` (component)

A PBD soft volume: a closed surface of points (2D: a ring in the xy plane) held by distance constraints along its edges and by one constraint on the volume it encloses (2D: its area), dropped under gravity and pushed out of every solid collider, so it lands, squashes and springs back: jelly, a water balloon, a slime. Built at the entity's world pose (its og.Transform after its parents') when it appears or its shape changes. It moves nothing (one-way like og.Cloth). Its points are world state; og.physics.softBodyPoints reads them and the renderer draws its surface with `material`.

Token `og.SoftBody`, registered under the schema name `og.SoftBody`; its value type is `SoftBody`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `number` | `0` | 0 … 2 | — | — | SOFT_SPHERE 0, SOFT_BOX 1 or SOFT_MESH 2. |
| `radius` | `number` | `0.5` | ≥ 0.01 | `m` | tunable | A sphere's radius. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | — | A box's full extents. |
| `detail` | `number` | `2` | 0 … 7 | — | — | How finely its surface is divided: a sphere 0 to 3 (12, 42, 162 or 642 points; 2D 12 to 96), a box 0 to 7 (cells along a face's edge, minus one). See `physics §8`. |
| `mesh` | `Geometry` | `0` | — | — | — | SOFT_MESH's geometry: a loaded glTF's (registered automatically) or one given to og.physics.setMesh; closed, its vertices welded where they meet. |
| `stiffness` | `number` | `1` | 0 … 1 | — | tunable | How stiff its edges are, 0 to 1 a pass: 1 holds their length, lower lets the surface stretch and wobble. |
| `pressure` | `number` | `1` | 0 … 4 | — | tunable | The volume it keeps, as a share of its rest volume: 1 holds it, above inflates it, 0 holds none (it slumps like cloth). |
| `volumeStiffness` | `number` | `1` | 0 … 1 | — | tunable | How much of its volume a pass restores, 0 to 1. |
| `iterations` | `number` | `8` | 1 … 64 | — | tunable | Constraint passes per substep. |
| `substeps` | `number` | `2` | 1 … 16 | — | — | Simulation steps per tick. |
| `damping` | `number` | `0.995` | 0 … 1 | — | tunable | How much of its velocity each point keeps per tick. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `thickness` | `number` | `0.03` | ≥ 0 | `m` | tunable | The distance its points keep from solids (0: no collision). |
| `friction` | `number` | `0.3` | 0 … 1 | — | tunable | How much of its sliding a point touching a solid loses each step. |
| `pinTop` | `number` | `0` | ≥ 0 | `m` | — | Points within this height of its top at rest are pinned to the entity and follow it (0: none): a water balloon on a string. |
| `material` | `Material` | `0` | — | — | — | The material it is drawn with (og.render.standardMaterial); 0 is a plain one. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |
| `simulation` | `number` | `0` | 0 … 1 | — | — | SIMULATION_GAMEPLAY 0 (in the tick, world state) or SIMULATION_VISUAL 1 (on the GPU, for looks only; see SIMULATION_VISUAL). See `physics §8`. |
| `volume` | `number` | `1` | — | — | read only | Written: its volume (2D: area) now over its rest volume; below 1 squashed, above inflated; 1 when SIMULATION_VISUAL. |
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Written: the mean of its points, in world space. |

```ogt
component SoftBody {
  doc "A PBD soft volume: a closed surface of points (2D: a ring in the xy plane) held by distance constraints along its edges and by one constraint on the volume it encloses (2D: its area), dropped under gravity and pushed out of every solid collider, so it lands, squashes and springs back: jelly, a water balloon, a slime. Built at the entity's world pose (its og.Transform after its parents') when it appears or its shape changes. It moves nothing (one-way like og.Cloth). Its points are world state; og.physics.softBodyPoints reads them and the renderer draws its surface with `material`."
  field shape u32 "SOFT_SPHERE 0, SOFT_BOX 1 or SOFT_MESH 2." {
    min 0
    max 2
  }
  field radius f32 "A sphere's radius." {
    default 0.5
    min 0.01
    unit m
    tunable true
  }
  field size Vector3 "A box's full extents." {
    default 1 1 1
    min 0
    unit m
  }
  field detail u32 "How finely its surface is divided: a sphere 0 to 3 (12, 42, 162 or 642 points; 2D 12 to 96), a box 0 to 7 (cells along a face's edge, minus one)." {
    guide "physics §8"
    default 2
    min 0
    max 7
  }
  field mesh Geometry "SOFT_MESH's geometry: a loaded glTF's (registered automatically) or one given to og.physics.setMesh; closed, its vertices welded where they meet."
  field stiffness f32 "How stiff its edges are, 0 to 1 a pass: 1 holds their length, lower lets the surface stretch and wobble." {
    default 1
    min 0
    max 1
    tunable true
  }
  field pressure f32 "The volume it keeps, as a share of its rest volume: 1 holds it, above inflates it, 0 holds none (it slumps like cloth)." {
    default 1
    min 0
    max 4
    tunable true
  }
  field volumeStiffness f32 "How much of its volume a pass restores, 0 to 1." {
    default 1
    min 0
    max 1
    tunable true
  }
  field iterations u32 "Constraint passes per substep." {
    default 8
    min 1
    max 64
    tunable true
  }
  field substeps u32 "Simulation steps per tick." {
    default 2
    min 1
    max 16
  }
  field damping f32 "How much of its velocity each point keeps per tick." {
    default 0.995
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
  field thickness f32 "The distance its points keep from solids (0: no collision)." {
    default 0.03
    min 0
    unit m
    tunable true
  }
  field friction f32 "How much of its sliding a point touching a solid loses each step." {
    default 0.3
    min 0
    max 1
    tunable true
  }
  field pinTop f32 "Points within this height of its top at rest are pinned to the entity and follow it (0: none): a water balloon on a string." {
    min 0
    unit m
  }
  field material Material "The material it is drawn with (og.render.standardMaterial); 0 is a plain one."
  field visible bool "Whether the renderer draws it." {
    default true
  }
  field simulation u32 "SIMULATION_GAMEPLAY 0 (in the tick, world state) or SIMULATION_VISUAL 1 (on the GPU, for looks only; see SIMULATION_VISUAL)." {
    guide "physics §8"
    min 0
    max 1
  }
  field volume f32 "Written: its volume (2D: area) now over its rest volume; below 1 squashed, above inflated; 1 when SIMULATION_VISUAL." {
    default 1
    readonly true
  }
  field center Vector3 "Written: the mean of its points, in world space." {
    worldspace world
    unit m
    readonly true
  }
}
```

<sub>Module `physics` line 1198</sub>
