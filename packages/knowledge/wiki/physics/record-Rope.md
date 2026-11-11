<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Rope"></a>

### `og.Rope` (component)

A Verlet rope: `segments` links from the entity's world position (its og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`.

Also known as: chain, cable, swinging lantern, hanging rope, Verlet rope

Token `og.Rope`, registered under the schema name `og.Rope`; its value type is `Rope`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `segments` | `number` | `16` | 1 … 256 | — | — | The number of links (points minus one). |
| `length` | `number` | `4` | ≥ 0 | `m` | tunable | The rope's length at rest. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The start's offset from the entity's position (world axes). |
| `end` | `Entity` | `0` | — | — | — | The entity the far end is tied to; 0 leaves it free. |
| `endOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | The far end's offset from `end`'s position. |
| `iterations` | `number` | `12` | 1 … 64 | — | tunable | Constraint passes per step: more is stiffer, at more cost. |
| `damping` | `number` | `0.99` | 0 … 1 | — | tunable | How much of its velocity each point keeps per tick (1: none lost). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | tunable | A steady acceleration (wind). |
| `radius` | `number` | `0.05` | ≥ 0 | `m` | tunable | The thickness it collides at (0: no collision). |
| `width` | `number` | `0.08` | ≥ 0 | `m` | — | The drawn width. |
| `color` | `Vector3` | `{ x: 0.55, y: 0.42, z: 0.28 }` | — | — | — | The drawn linear colour. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Rope {
  aliases "chain, cable, swinging lantern, hanging rope, Verlet rope"
  doc "A Verlet rope: `segments` links from the entity's world position (its og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`."
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
  field iterations u32 "Constraint passes per step: more is stiffer, at more cost." {
    default 12
    min 1
    max 64
    tunable true
  }
  field damping f32 "How much of its velocity each point keeps per tick (1: none lost)." {
    default 0.99
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
  field wind Vector3 "A steady acceleration (wind)." {
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
  field color Vector3 "The drawn linear colour." {
    default 0.55 0.42 0.28
  }
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>Module `physics` line 950</sub>
