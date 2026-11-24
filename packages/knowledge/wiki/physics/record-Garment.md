<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Garment"></a>

### `og.Garment` (component)

A skinned small-step XPBD garment with explicit velocities, area-lumped masses, dihedral bending and relative-air drag and lift. Scaled maxDistance bounds each point about its skinned target; backstopRadius enables a spherical backstop behind it. bodyRadius enables skeleton capsules. pinTop and vertex red times alpha control freedom. teleportDistance resets large pose jumps. Solid contacts exchange momentum with dynamic bodies. World state, 3D only; blendshapes are not applied.

See `physics §8`.

Token `og.Garment`, registered under the schema name `og.Garment`; its value type is `Garment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mesh` | `number` | `0` | — | — | — | The glTF mesh of the entity's model that is the garment (its primitives together), as og.Blendshapes.mesh. |
| `maxDistance` | `number` | `0.2` | ≥ 0 | `m` | tunable | How far a point may stray from where the skinned pose puts it. |
| `pinTop` | `number` | `0.04` | ≥ 0 | `m` | tunable | Points within this height of the garment's top at rest (the bind pose, the model's +y) stay on the skinned pose: a skirt's waistband, a cape's collar. |
| `backstop` | `number` | `0` | ≥ 0 | `m` | tunable | Offset of the spherical backstop behind the skin, scaled by the pose; used with backstopRadius. |
| `backstopRadius` | `number` | `0` | ≥ 0 | `m` | — | Spherical backstop radius, scaled by the pose (0: off). |
| `bodyRadius` | `number` | `0` | ≥ 0 | `m` | — | Radius of skeleton bone capsules in model metres, scaled by the character (0: off). Hosts may supply tapered capsules through GarmentPose. |
| `teleportDistance` | `number` | `0` | ≥ 0 | `m` | — | Reset on target translations larger than this, scaled by the pose (0: off). |
| `density` | `number` | `1` | ≥ 0 | — | — | Surface mass density in kg/m² (0: fixed). |
| `drag` | `number` | `0` | ≥ 0 | — | tunable | Quadratic relative-air drag coefficient (0: off). |
| `lift` | `number` | `0` | ≥ 0 | — | tunable | Relative-air lift coefficient (0: off). |
| `stiffness` | `number` | `100000` | ≥ 0 | — | tunable | Physical XPBD edge stiffness in N/m (0: off). |
| `bending` | `number` | `0.1` | ≥ 0 | — | tunable | Physical XPBD dihedral stiffness (0: off). |
| `iterations` | `number` | `6` | 1 … 64 | — | tunable | Constraint passes per substep. |
| `substeps` | `number` | `1` | 1 … 16 | — | — | Simulation steps per tick. |
| `damping` | `number` | `1.85` | 0 … 100 | `perSecond` | tunable | Velocity decay rate (0: none): each substep h keeps 1 / (1 + damping * h) of a point's velocity. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | Air velocity added to the per-point wind field. Drag and lift act on relative air velocity (zero coefficients: off). See `physics §8`. |
| `gust` | `number` | `0.6` | 0 … 2 | — | tunable | How much the wind varies over the garment and the tick (0: steady). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | The thickness it collides with solid colliders at (0: no collision; the skinned pose, through `maxDistance` and `backstop`, holds it on the body). A character's own collider counts: keep it inside the body, or leave this 0. |
| `friction` | `number` | `0` | 0 … 1 | — | tunable | How much of its sliding a point touching a solid loses each step. |
| `simulation` | `Simulation` | `"gameplay"` | — | — | — | gameplay (in the tick, world state) or visual (on the GPU, for looks only). See `physics §8`. |

```ogt
component Garment {
  doc "A skinned small-step XPBD garment with explicit velocities, area-lumped masses, dihedral bending and relative-air drag and lift. Scaled maxDistance bounds each point about its skinned target; backstopRadius enables a spherical backstop behind it. bodyRadius enables skeleton capsules. pinTop and vertex red times alpha control freedom. teleportDistance resets large pose jumps. Solid contacts exchange momentum with dynamic bodies. World state, 3D only; blendshapes are not applied."
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
  field backstop f32 "Offset of the spherical backstop behind the skin, scaled by the pose; used with backstopRadius." {
    min 0
    unit m
    tunable true
  }
  field backstopRadius f32 "Spherical backstop radius, scaled by the pose (0: off)." {
    min 0
    unit m
  }
  field bodyRadius f32 "Radius of skeleton bone capsules in model metres, scaled by the character (0: off). Hosts may supply tapered capsules through GarmentPose." {
    min 0
    unit m
  }
  field teleportDistance f32 "Reset on target translations larger than this, scaled by the pose (0: off)." {
    min 0
    unit m
  }
  field density f32 "Surface mass density in kg/m² (0: fixed)." {
    default 1
    min 0
  }
  field drag f32 "Quadratic relative-air drag coefficient (0: off)." {
    min 0
    tunable true
  }
  field lift f32 "Relative-air lift coefficient (0: off)." {
    min 0
    tunable true
  }
  field stiffness f32 "Physical XPBD edge stiffness in N/m (0: off)." {
    default 100000
    min 0
    tunable true
  }
  field bending f32 "Physical XPBD dihedral stiffness (0: off)." {
    default 0.1
    min 0
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
  field damping f32 "Velocity decay rate (0: none): each substep h keeps 1 / (1 + damping * h) of a point's velocity." {
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
  field wind Vector3 "Air velocity added to the per-point wind field. Drag and lift act on relative air velocity (zero coefficients: off)." {
    guide "physics §8"
    unit mps
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

<sub>Module `physics` line 1489</sub>
