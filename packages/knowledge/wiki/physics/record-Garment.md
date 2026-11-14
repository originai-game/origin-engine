<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Garment"></a>

### `og.Garment` (component)

Cloth on a skinned character: glTF mesh `mesh` of the entity's og.Model (a skirt, a cape, a coat's tails), posed by the entity's og.Skeleton and simulated from there. Each tick its points swing under gravity and wind behind the motion, but stay within `maxDistance` of the skinned pose and `backstop` behind it, keeping their edges' lengths. Points within `pinTop` of its top, or whose vertex colour's red or alpha is 0, stay on the pose; red times alpha scales `maxDistance`. One-way like og.Cloth, simulated in the physics step as world state. 3D only; its blendshapes are not applied.

See `physics §8`.

Token `og.Garment`, registered under the schema name `og.Garment`; its value type is `Garment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mesh` | `number` | `0` | — | — | — | The glTF mesh of the entity's model that is the garment (its primitives together), as og.Blendshapes.mesh. |
| `maxDistance` | `number` | `0.2` | ≥ 0 | `m` | tunable | How far a point may stray from where the skinned pose puts it. |
| `pinTop` | `number` | `0.04` | ≥ 0 | `m` | tunable | Points within this height of the garment's top at rest (the bind pose, the model's +y) stay on the skinned pose: a skirt's waistband, a cape's collar. |
| `backstop` | `number` | `0` | ≥ 0 | `m` | tunable | How far a point may go behind where the skinned pose puts it, against the skinned normal (0: never behind it, so it cannot sink into the body). |
| `stiffness` | `number` | `1` | 0 … 1 | — | tunable | How stiff its edges are, 0 to 1 a pass: 1 holds their length, lower stretches. |
| `bending` | `number` | `0.1` | 0 … 1 | — | tunable | How much it resists folding, 0 to 1 a pass (across each pair of triangles that share an edge): 0 drapes like silk, near 1 like leather. |
| `iterations` | `number` | `6` | 1 … 64 | — | tunable | Constraint passes per substep. |
| `substeps` | `number` | `1` | 1 … 16 | — | — | Simulation steps per tick. |
| `damping` | `number` | `1.85` | 0 … 100 | `perSecond` | tunable | How fast its points' velocity decays (0: none lost): each tick a point keeps 1 / (1 + damping * dt) of it, as RigidBody.linearDamping. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | tunable | A steady acceleration of its own, added to og.Wind's push (its velocity as an acceleration, per second); the garment flutters with both. A visual garment (simulation visual) takes the sum as air moving at its magnitude in m/s. See `physics §8`. |
| `gust` | `number` | `0.6` | 0 … 2 | — | tunable | How much the wind varies over the garment and the tick (0: steady). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | The thickness it collides with solid colliders at (0: no collision; the skinned pose, through `maxDistance` and `backstop`, holds it on the body). A character's own collider counts: keep it inside the body, or leave this 0. |
| `friction` | `number` | `0` | 0 … 1 | — | tunable | How much of its sliding a point touching a solid loses each step. |
| `simulation` | `Simulation` | `"gameplay"` | — | — | — | gameplay (in the tick, world state) or visual (on the GPU, for looks only). See `physics §8`. |

```ogt
component Garment {
  doc "Cloth on a skinned character: glTF mesh `mesh` of the entity's og.Model (a skirt, a cape, a coat's tails), posed by the entity's og.Skeleton and simulated from there. Each tick its points swing under gravity and wind behind the motion, but stay within `maxDistance` of the skinned pose and `backstop` behind it, keeping their edges' lengths. Points within `pinTop` of its top, or whose vertex colour's red or alpha is 0, stay on the pose; red times alpha scales `maxDistance`. One-way like og.Cloth, simulated in the physics step as world state. 3D only; its blendshapes are not applied."
  guide "physics §8"
  field mesh u32 "The glTF mesh of the entity's model that is the garment (its primitives together), as og.Blendshapes.mesh."
  field maxDistance f32 "How far a point may stray from where the skinned pose puts it." {
    default 0.2
    min 0
    unit m
    tunable true
  }
  field pinTop f32 "Points within this height of the garment's top at rest (the bind pose, the model's +y) stay on the skinned pose: a skirt's waistband, a cape's collar." {
    default 0.04
    min 0
    unit m
    tunable true
  }
  field backstop f32 "How far a point may go behind where the skinned pose puts it, against the skinned normal (0: never behind it, so it cannot sink into the body)." {
    min 0
    unit m
    tunable true
  }
  field stiffness f32 "How stiff its edges are, 0 to 1 a pass: 1 holds their length, lower stretches." {
    default 1
    min 0
    max 1
    tunable true
  }
  field bending f32 "How much it resists folding, 0 to 1 a pass (across each pair of triangles that share an edge): 0 drapes like silk, near 1 like leather." {
    default 0.1
    min 0
    max 1
    tunable true
  }
  field iterations u32 "Constraint passes per substep." {
    default 6
    min 1
    max 64
    tunable true
  }
  field substeps u32 "Simulation steps per tick." {
    default 1
    min 1
    max 16
  }
  field damping f32 "How fast its points' velocity decays (0: none lost): each tick a point keeps 1 / (1 + damping * dt) of it, as RigidBody.linearDamping." {
    default 1.85
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity." {
    default 1
    min -10
    max 10
    tunable true
  }
  field wind Vector3 "A steady acceleration of its own, added to og.Wind's push (its velocity as an acceleration, per second); the garment flutters with both. A visual garment (simulation visual) takes the sum as air moving at its magnitude in m/s." {
    guide "physics §8"
    unit mps2
    tunable true
  }
  field gust f32 "How much the wind varies over the garment and the tick (0: steady)." {
    default 0.6
    min 0
    max 2
    tunable true
  }
  field radius f32 "The thickness it collides with solid colliders at (0: no collision; the skinned pose, through `maxDistance` and `backstop`, holds it on the body). A character's own collider counts: keep it inside the body, or leave this 0." {
    min 0
    unit m
    tunable true
  }
  field friction f32 "How much of its sliding a point touching a solid loses each step." {
    min 0
    max 1
    tunable true
  }
  field simulation Simulation "gameplay (in the tick, world state) or visual (on the GPU, for looks only)." {
    guide "physics §8"
  }
}
```

<sub>Module `physics` line 1208</sub>
