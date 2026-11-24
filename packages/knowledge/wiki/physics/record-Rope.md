<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Rope"></a>

### `og.Rope` (component)

A small-step XPBD rope with explicit velocities and point masses. Its start and end follow any Transform entity. Changing length reels it without rebuilding its points; coupling applies endpoint and solid contact impulses to dynamic bodies. Points, velocities and broken links are world state.

Also known as: chain, cable, swinging lantern, hanging rope, XPBD rope

Token `og.Rope`, registered under the schema name `og.Rope`; its value type is `Rope`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `segments` | `number` | `16` | 1 … 256 | — | — | The number of links (points minus one). |
| `length` | `number` | `4` | ≥ 0 | `m` | tunable | The rope's length at rest. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The start's offset from the entity's position (world axes). |
| `end` | `Entity` | `0` | — | — | — | The entity the far end is tied to; 0 leaves it free. |
| `endOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | The far end's offset from `end`'s position. |
| `stiffness` | `number` | `100000` | ≥ 0 | — | tunable | Physical XPBD axial stiffness in N/m; compliance is 1/stiffness (0: constraints off). |
| `mass` | `number` | `0.1` | ≥ 0 | `kg` | — | Mass of each free point in kg (0: fixed). |
| `substeps` | `number` | `4` | 0 … 16 | — | — | Small steps per tick (0: simulation off). |
| `coupling` | `boolean` | `false` | — | — | — | Apply endpoint and solid contact reactions to dynamic bodies. |
| `breakForce` | `number` | `0` | ≥ 0 | — | — | Break links above this force in N (0: unbreakable). Broken links stay broken until rebuilt. |
| `iterations` | `number` | `12` | 1 … 64 | — | tunable | XPBD constraint passes per substep; stiffness sets physical compliance. |
| `damping` | `number` | `0.6` | 0 … 100 | `perSecond` | tunable | Velocity decay rate (0: none): each substep h keeps 1 / (1 + damping * h) of a point's velocity. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | tunable | A steady acceleration of its own, added to og.Wind's push (its velocity as an acceleration, per second). |
| `radius` | `number` | `0.05` | ≥ 0 | `m` | tunable | The thickness it collides at (0: no collision). |
| `width` | `number` | `0.08` | ≥ 0 | `m` | — | The drawn width. |
| `color` | `Color` | `{ r: 0.55, g: 0.42, b: 0.28 }` | — | — | — | The drawn colour. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Rope {
  aliases "chain, cable, swinging lantern, hanging rope, XPBD rope"
  doc "A small-step XPBD rope with explicit velocities and point masses. Its start and end follow any Transform entity. Changing length reels it without rebuilding its points; coupling applies endpoint and solid contact impulses to dynamic bodies. Points, velocities and broken links are world state."
  field segments u32 "The number of links (points minus one)." {
    default 16
    min 1
    max 256
  }
  field length f32 "The rope's length at rest." {
    default 4
    min 0
    unit m
    tunable true
  }
  field offset Vector3 "The start's offset from the entity's position (world axes)." {
    unit m
  }
  field end Entity "The entity the far end is tied to; 0 leaves it free."
  field endOffset Vector3 "The far end's offset from `end`'s position." {
    worldspace none
    unit m
  }
  field stiffness f32 "Physical XPBD axial stiffness in N/m; compliance is 1/stiffness (0: constraints off)." {
    default 100000
    min 0
    tunable true
  }
  field mass f32 "Mass of each free point in kg (0: fixed)." {
    default 0.1
    min 0
    unit kg
  }
  field substeps u32 "Small steps per tick (0: simulation off)." {
    default 4
    min 0
    max 16
  }
  field coupling bool "Apply endpoint and solid contact reactions to dynamic bodies."
  field breakForce f32 "Break links above this force in N (0: unbreakable). Broken links stay broken until rebuilt." {
    min 0
  }
  field iterations u32 "XPBD constraint passes per substep; stiffness sets physical compliance." {
    default 12
    min 1
    max 64
    tunable true
  }
  field damping f32 "Velocity decay rate (0: none): each substep h keeps 1 / (1 + damping * h) of a point's velocity." {
    default 0.6
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
  field wind Vector3 "A steady acceleration of its own, added to og.Wind's push (its velocity as an acceleration, per second)." {
    unit mps2
    tunable true
  }
  field radius f32 "The thickness it collides at (0: no collision)." {
    default 0.05
    min 0
    unit m
    tunable true
  }
  field width f32 "The drawn width." {
    default 0.08
    min 0
    unit m
  }
  field color Color "The drawn colour." {
    default 0.55 0.42 0.28
  }
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>Module `physics` line 1186</sub>
