<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="record-NavAgent"></a>

### `og.NavAgent` (component)

An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents.

Also known as: NavMeshAgent, NavigationAgent, pathfinding agent, move to target, AI movement, obstacle avoidance

Token `og.NavAgent`, registered under the schema name `og.NavAgent`; its value type is `NavAgent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nav` | `Navigation` | `0` | — | — | — | The grid, navmesh or flow field it plans on (og.nav.grid, og.nav.load, og.nav.flowField); 0 moves straight at the target. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | worldspace `world` | Where it goes while seeking. |
| `seeking` | `boolean` | `false` | — | — | — | Moves towards the target while true. |
| `speed` | `number` | `3.5` | 0 … 100 | `mps` | tunable | Top speed. |
| `acceleration` | `number` | `20` | 0 … 1000 | `mps2` | tunable | How fast it changes velocity. |
| `radius` | `number` | `0.4` | 0 … 10 | `m` | tunable | Its radius: grid paths keep this clear of walls; separation uses it. |
| `stopDistance` | `number` | `0.1` | 0 … 10 | `m` | tunable | It has arrived within this distance of the target. |
| `slowRadius` | `number` | `1` | 0 … 20 | `m` | tunable | It slows down inside this distance of the target. |
| `repathTicks` | `number` | `30` | 0 … 3600 | `ticks` | — | Re-plans this often while seeking (0: only when the target moves). |
| `repathDistance` | `number` | `0.5` | 0 … 100 | `m` | — | Re-plans when the target moved farther than this from the last plan. |
| `smooth` | `boolean` | `true` | — | — | — | Pulls grid paths straight where nothing is in the way. |
| `faceMovement` | `boolean` | `false` | — | — | — | Turns the entity to face where it moves (3D: about +Y, its +Z forward; 2D: about +Z, its +X forward). |
| `turnSpeed` | `number` | `10` | 0 … 100 | `radps` | — | How fast it turns when facing its movement. |
| `drive` | `boolean` | `true` | — | — | — | Moves the entity (its og.Velocity, or its og.Transform) while true; false keeps planning but leaves the motion to the game (a knock-back, a stun). |
| `status` | `number` | `0` | — | — | read only | NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE or NAV_PARTIAL (written by og.nav). |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | read only | Its velocity (written by og.nav). |
| `waypoint` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | The path corner it heads for (written by og.nav). |
| `remaining` | `number` | `0` | — | `m` | read only | The path length left (written by og.nav). |
| `plans` | `number` | `0` | — | — | read only | Paths planned so far (written by og.nav). |

```ogt
component NavAgent {
  doc "An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents."
  aliases "NavMeshAgent, NavigationAgent, pathfinding agent, move to target, AI movement, obstacle avoidance"
  field nav Navigation "The grid, navmesh or flow field it plans on (og.nav.grid, og.nav.load, og.nav.flowField); 0 moves straight at the target."
  field target Vector3 "Where it goes while seeking." {
    worldspace world
  }
  field seeking bool "Moves towards the target while true."
  field speed f32 "Top speed." {
    default 3.5
    min 0
    max 100
    unit mps
    tunable true
  }
  field acceleration f32 "How fast it changes velocity." {
    default 20
    min 0
    max 1000
    unit mps2
    tunable true
  }
  field radius f32 "Its radius: grid paths keep this clear of walls; separation uses it." {
    default 0.4
    min 0
    max 10
    unit m
    tunable true
  }
  field stopDistance f32 "It has arrived within this distance of the target." {
    default 0.1
    min 0
    max 10
    unit m
    tunable true
  }
  field slowRadius f32 "It slows down inside this distance of the target." {
    default 1
    min 0
    max 20
    unit m
    tunable true
  }
  field repathTicks u32 "Re-plans this often while seeking (0: only when the target moves)." {
    default 30
    min 0
    max 3600
    unit ticks
  }
  field repathDistance f32 "Re-plans when the target moved farther than this from the last plan." {
    default 0.5
    min 0
    max 100
    unit m
  }
  field smooth bool "Pulls grid paths straight where nothing is in the way." {
    default true
  }
  field faceMovement bool "Turns the entity to face where it moves (3D: about +Y, its +Z forward; 2D: about +Z, its +X forward)."
  field turnSpeed f32 "How fast it turns when facing its movement." {
    default 10
    min 0
    max 100
    unit radps
  }
  field drive bool "Moves the entity (its og.Velocity, or its og.Transform) while true; false keeps planning but leaves the motion to the game (a knock-back, a stun)." {
    default true
  }
  field status u32 "NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE or NAV_PARTIAL (written by og.nav)." {
    readonly true
  }
  field velocity Vector3 "Its velocity (written by og.nav)." {
    unit mps
    readonly true
  }
  field waypoint Vector3 "The path corner it heads for (written by og.nav)." {
    worldspace world
    unit m
    readonly true
  }
  field remaining f32 "The path length left (written by og.nav)." {
    unit m
    readonly true
  }
  field plans u32 "Paths planned so far (written by og.nav)." {
    readonly true
  }
}
```

<sub>Module `nav` line 28</sub>
