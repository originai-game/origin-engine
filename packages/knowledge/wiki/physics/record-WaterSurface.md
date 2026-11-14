<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-WaterSurface"></a>

### `og.WaterSurface` (component)

Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them.

Token `og.WaterSurface`, registered under the schema name `og.WaterSurface`; its value type is `WaterSurface`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `resolution` | `number` | `64` | 8 … 256 | — | — | Grid points along each side. |
| `size` | `number` | `16` | ≥ 0.1 | `m` | tunable | The side of the square of water. |
| `waveSpeed` | `number` | `3` | ≥ 0 | `mps` | tunable | How fast waves travel. |
| `damping` | `number` | `0.9` | 0 … 100 | `perSecond` | tunable | How fast waves die (0: no loss): each tick a wave keeps 1 / (1 + damping * dt) of its motion, as RigidBody.linearDamping. |
| `splash` | `number` | `1` | ≥ 0 | — | tunable | How strongly bodies push waves. |
| `buoyancy` | `number` | `9.81` | ≥ 0 | — | tunable | The upward force on a submerged body per cubic metre of its box under water (0: bodies sink). |
| `drag` | `number` | `1.5` | ≥ 0 | `perSecond` | tunable | How much a submerged body's velocity is slowed. |

```ogt
component WaterSurface {
  doc "Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them."
  field resolution u32 "Grid points along each side." {
    default 64
    min 8
    max 256
  }
  field size f32 "The side of the square of water." {
    default 16
    min 0.1
    unit m
    tunable true
  }
  field waveSpeed f32 "How fast waves travel." {
    default 3
    min 0
    unit mps
    tunable true
  }
  field damping f32 "How fast waves die (0: no loss): each tick a wave keeps 1 / (1 + damping * dt) of its motion, as RigidBody.linearDamping." {
    default 0.9
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field splash f32 "How strongly bodies push waves." {
    default 1
    min 0
    tunable true
  }
  field buoyancy f32 "The upward force on a submerged body per cubic metre of its box under water (0: bodies sink)." {
    default 9.81
    min 0
    tunable true
  }
  field drag f32 "How much a submerged body's velocity is slowed." {
    default 1.5
    min 0
    unit perSecond
    tunable true
  }
}
```

<sub>Module `physics` line 1634</sub>
