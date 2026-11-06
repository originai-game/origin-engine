<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Cloth"></a>

### `og.Cloth` (component)

A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's world position (its og.Transform after its parents': its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour).

Token `og.Cloth`, registered under the schema name `og.Cloth`; its value type is `Cloth`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `columns` | `number` | `12` | 2 … 64 | — | — | Points across. |
| `rows` | `number` | `12` | 2 … 64 | — | — | Points down. |
| `spacing` | `number` | `0.15` | ≥ 0.001 | `m` | tunable | The rest distance between neighbouring points. |
| `pins` | `number` | `1` | — | — | — | Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings). |
| `iterations` | `number` | `8` | 1 … 64 | — | tunable | Constraint passes per step. |
| `damping` | `number` | `0.985` | 0 … 1 | — | tunable | How much of its velocity each point keeps per tick. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | tunable | A steady acceleration (wind); the cloth flutters with it. |
| `gust` | `number` | `0.6` | 0 … 2 | — | tunable | How much the wind varies over the cloth and the tick (0: steady). |
| `radius` | `number` | `0.04` | ≥ 0 | `m` | tunable | The thickness it collides at (0: no collision). |
| `thickness` | `number` | `0` | ≥ 0 | `m` | tunable | Self-collision: how far apart its points keep from the points of the cloth that are not their near neighbours (0: none). About `spacing` keeps a cloth that folds or piles up from passing through itself. |
| `substeps` | `number` | `1` | 1 … 16 | — | — | Simulation steps per tick, each integrating and then solving `iterations` passes; with self-collision enough that a point moves less than `thickness` per step (a point moves at most half the thickness a step). |
| `friction` | `number` | `0` | 0 … 1 | — | tunable | How much of its sliding a point touching a solid loses each step (0: it slides freely; near 1 it sticks where it lands, so a cloth lowered onto the floor folds instead of spreading). |
| `material` | `Material` | `0` | — | — | — | The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |
| `simulation` | `number` | `0` | 0 … 1 | — | — | SIMULATION_GAMEPLAY 0 (in the tick, world state) or SIMULATION_VISUAL 1 (on the GPU, for looks only; see SIMULATION_VISUAL). |

```ogt
component Cloth {
  doc "A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's world position (its og.Transform after its parents': its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour)."
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
  field iterations u32 "Constraint passes per step." {
    default 8
    min 1
    max 64
    tunable true
  }
  field damping f32 "How much of its velocity each point keeps per tick." {
    default 0.985
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
  field wind Vector3 "A steady acceleration (wind); the cloth flutters with it." {
    unit mps2
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
  field substeps u32 "Simulation steps per tick, each integrating and then solving `iterations` passes; with self-collision enough that a point moves less than `thickness` per step (a point moves at most half the thickness a step)." {
    default 1
    min 1
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
  field simulation u32 "SIMULATION_GAMEPLAY 0 (in the tick, world state) or SIMULATION_VISUAL 1 (on the GPU, for looks only; see SIMULATION_VISUAL)." {
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 9468</sub>
