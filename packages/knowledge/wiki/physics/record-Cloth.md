<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Cloth"></a>

### `og.Cloth` (component)

A small-step XPBD cloth: a columns by rows grid, pinned to its entity, with stretch, dihedral bending, long-range attachments, relative-air drag and lift, and self-collision. Velocities and area-lumped inverse masses are explicit world state; there is no self-collision speed cap.

Also known as: flag, banner, curtain, sail, Unity Cloth, Unreal Chaos Cloth

Token `og.Cloth`, registered under the schema name `og.Cloth`; its value type is `Cloth`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `columns` | `number` | `12` | 2 … 64 | — | — | Points across. |
| `rows` | `number` | `12` | 2 … 64 | — | — | Points down. |
| `spacing` | `number` | `0.15` | ≥ 0.001 | `m` | tunable | The rest distance between neighbouring points. |
| `pins` | `number` | `1` | — | — | — | Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings). |
| `stiffness` | `number` | `100000` | ≥ 0 | — | tunable | Physical XPBD stretch stiffness in N/m (0: off). |
| `bending` | `number` | `0` | ≥ 0 | — | tunable | Physical XPBD dihedral stiffness (0: off). |
| `tether` | `number` | `0` | ≥ 0 | — | tunable | Long-range attachment stiffness in N/m (0: off). |
| `density` | `number` | `1` | ≥ 0 | — | — | Surface mass density in kg/m² (0: fixed). |
| `drag` | `number` | `0` | ≥ 0 | — | tunable | Quadratic relative-air drag coefficient (0: off). |
| `lift` | `number` | `0` | ≥ 0 | — | tunable | Relative-air lift coefficient (0: off). |
| `iterations` | `number` | `8` | 1 … 64 | — | tunable | XPBD constraint passes per substep. |
| `damping` | `number` | `0.9` | 0 … 100 | `perSecond` | tunable | Velocity decay rate (0: none): each substep h keeps 1 / (1 + damping * h) of a point's velocity. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | Air velocity added to the per-point wind field. Drag and lift use air minus point velocity; both coefficients zero disable aerodynamic forces. See `physics §8`. |
| `gust` | `number` | `0.6` | 0 … 2 | — | tunable | How much the wind varies over the cloth and the tick (0: steady). |
| `radius` | `number` | `0.04` | ≥ 0 | `m` | tunable | The thickness it collides at (0: no collision). |
| `thickness` | `number` | `0` | ≥ 0 | `m` | tunable | Self-collision: how far apart its points keep from the points of the cloth that are not their near neighbours (0: none). About `spacing` keeps a cloth that folds or piles up from passing through itself. |
| `substeps` | `number` | `1` | 0 … 16 | — | — | Small steps per tick, each integrating explicit velocity and solving iterations XPBD passes (0: off). |
| `friction` | `number` | `0` | 0 … 1 | — | tunable | How much of its sliding a point touching a solid loses each step (0: it slides freely; near 1 it sticks where it lands, so a cloth lowered onto the floor folds instead of spreading). |
| `material` | `Material` | `0` | — | — | — | The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |
| `simulation` | `Simulation` | `"gameplay"` | — | — | — | gameplay (in the tick, world state) or visual (on the GPU, for looks only). See `physics §8`. |

```ogt
component Cloth {
  doc "A small-step XPBD cloth: a columns by rows grid, pinned to its entity, with stretch, dihedral bending, long-range attachments, relative-air drag and lift, and self-collision. Velocities and area-lumped inverse masses are explicit world state; there is no self-collision speed cap."
  aliases "flag, banner, curtain, sail, Unity Cloth, Unreal Chaos Cloth"
  field columns u32 "Points across." {
    default 12
    min 2
    max 64
  }
  field rows u32 "Points down." {
    default 12
    min 2
    max 64
  }
  field spacing f32 "The rest distance between neighbouring points." {
    default 0.15
    min 0.001
    unit m
    tunable true
  }
  field pins u32 "Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings)." {
    default 1
  }
  field stiffness f32 "Physical XPBD stretch stiffness in N/m (0: off)." {
    default 100000
    min 0
    tunable true
  }
  field bending f32 "Physical XPBD dihedral stiffness (0: off)." {
    min 0
    tunable true
  }
  field tether f32 "Long-range attachment stiffness in N/m (0: off)." {
    min 0
    tunable true
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
  field iterations u32 "XPBD constraint passes per substep." {
    default 8
    min 1
    max 64
    tunable true
  }
  field damping f32 "Velocity decay rate (0: none): each substep h keeps 1 / (1 + damping * h) of a point's velocity." {
    default 0.9
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
  field wind Vector3 "Air velocity added to the per-point wind field. Drag and lift use air minus point velocity; both coefficients zero disable aerodynamic forces." {
    guide "physics §8"
    unit mps
    tunable true
  }
  field gust f32 "How much the wind varies over the cloth and the tick (0: steady)." {
    default 0.6
    min 0
    max 2
    tunable true
  }
  field radius f32 "The thickness it collides at (0: no collision)." {
    default 0.04
    min 0
    unit m
    tunable true
  }
  field thickness f32 "Self-collision: how far apart its points keep from the points of the cloth that are not their near neighbours (0: none). About `spacing` keeps a cloth that folds or piles up from passing through itself." {
    min 0
    unit m
    tunable true
  }
  field substeps u32 "Small steps per tick, each integrating explicit velocity and solving iterations XPBD passes (0: off)." {
    default 1
    min 0
    max 16
  }
  field friction f32 "How much of its sliding a point touching a solid loses each step (0: it slides freely; near 1 it sticks where it lands, so a cloth lowered onto the floor folds instead of spreading)." {
    min 0
    max 1
    tunable true
  }
  field material Material "The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth."
  field visible bool "Whether the renderer draws it." {
    default true
  }
  field simulation Simulation "gameplay (in the tick, world state) or visual (on the GPU, for looks only)." {
    guide "physics §8"
  }
}
```

<sub>Module `physics` line 1383</sub>
