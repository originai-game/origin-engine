<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="record-Steering"></a>

### `og.Steering` (component)

Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays).

Token `og.Steering`, registered under the schema name `og.Steering`; its value type is `Steering`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seek` | `number` | `1` | 0 … 10 | — | tunable | Weight of following the path to the target (seek, then arrive at the end). |
| `flee` | `number` | `0` | 0 … 10 | — | tunable | Weight of fleeing fleeFrom. |
| `fleeFrom` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | What it flees. |
| `fleeRadius` | `number` | `6` | 0 … 100 | `m` | tunable | It flees (and evades) only inside this distance (0: always). |
| `wander` | `number` | `0` | 0 … 10 | — | tunable | Weight of wandering. |
| `wanderRadius` | `number` | `1` | 0 … 20 | `m` | tunable | The wander circle's radius. |
| `wanderDistance` | `number` | `2` | 0 … 20 | `m` | tunable | How far ahead the wander circle is. |
| `wanderJitter` | `number` | `3` | 0 … 50 | `radps` | tunable | How fast the wander point moves round the circle. |
| `pursue` | `Entity` | `0` | — | — | — | An entity to pursue (seek where it will be, from its NavAgent or og.Velocity); 0: none. |
| `pursueWeight` | `number` | `1` | 0 … 10 | — | tunable | Weight of pursuing. |
| `evade` | `Entity` | `0` | — | — | — | An entity to evade (flee where it will be); 0: none. |
| `evadeWeight` | `number` | `1` | 0 … 10 | — | tunable | Weight of evading. |
| `separation` | `number` | `0` | 0 … 10 | — | tunable | Weight of keeping apart from other agents. |
| `separationRadius` | `number` | `1` | 0 … 20 | `m` | tunable | Other agents closer than this push it away. |
| `avoidance` | `number` | `0` | 0 … 10 | — | tunable | Weight of avoiding colliders ahead. |
| `avoidDistance` | `number` | `2` | 0 … 50 | `m` | tunable | How far ahead the avoidance rays reach. |
| `avoidMask` | `number` | `0` | — | — | — | Collision layers the avoidance rays hit (0: every layer). |
| `wanderAngle` | `number` | `0` | — | `rad` | read only | The wander angle (written by og.nav). |

```ogt
component Steering {
  doc "Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays)."
  field seek f32 "Weight of following the path to the target (seek, then arrive at the end)." {
    default 1
    min 0
    max 10
    tunable true
  }
  field flee f32 "Weight of fleeing fleeFrom." {
    min 0
    max 10
    tunable true
  }
  field fleeFrom Vector3 "What it flees." {
    worldspace world
    unit m
  }
  field fleeRadius f32 "It flees (and evades) only inside this distance (0: always)." {
    default 6
    min 0
    max 100
    unit m
    tunable true
  }
  field wander f32 "Weight of wandering." {
    min 0
    max 10
    tunable true
  }
  field wanderRadius f32 "The wander circle's radius." {
    default 1
    min 0
    max 20
    unit m
    tunable true
  }
  field wanderDistance f32 "How far ahead the wander circle is." {
    default 2
    min 0
    max 20
    unit m
    tunable true
  }
  field wanderJitter f32 "How fast the wander point moves round the circle." {
    default 3
    min 0
    max 50
    unit radps
    tunable true
  }
  field pursue Entity "An entity to pursue (seek where it will be, from its NavAgent or og.Velocity); 0: none."
  field pursueWeight f32 "Weight of pursuing." {
    default 1
    min 0
    max 10
    tunable true
  }
  field evade Entity "An entity to evade (flee where it will be); 0: none."
  field evadeWeight f32 "Weight of evading." {
    default 1
    min 0
    max 10
    tunable true
  }
  field separation f32 "Weight of keeping apart from other agents." {
    min 0
    max 10
    tunable true
  }
  field separationRadius f32 "Other agents closer than this push it away." {
    default 1
    min 0
    max 20
    unit m
    tunable true
  }
  field avoidance f32 "Weight of avoiding colliders ahead." {
    min 0
    max 10
    tunable true
  }
  field avoidDistance f32 "How far ahead the avoidance rays reach." {
    default 2
    min 0
    max 50
    unit m
    tunable true
  }
  field avoidMask u32 "Collision layers the avoidance rays hit (0: every layer)."
  field wanderAngle f32 "The wander angle (written by og.nav)." {
    unit rad
    readonly true
  }
}
```

<sub>`api.ogt` line 13659</sub>
