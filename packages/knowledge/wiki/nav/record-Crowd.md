<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="record-Crowd"></a>

### `og.Crowd` (component)

Reciprocal crowd avoidance (ORCA) for an agent: beside an og.NavAgent, each tick its velocity is the one nearest what its path and steering want that keeps it clear of its nearest agents for `horizon` seconds and of the navmesh's walls for `obstacleHorizon`, each pair of crowd agents sharing the avoidance. Every agent solves from the same start-of-tick state in a fixed order, so crowds replay identically on every host; agents meeting perfectly symmetrically (a ring swapping places) are kept from locking by a margin of up to 5% of the radius picked from the entity, so a game adds no jitter of its own. Without one an agent moves as before (og.Steering's separation only).

Token `og.Crowd`, registered under the schema name `og.Crowd`; its value type is `Crowd`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `neighbours` | `number` | `8` | 0 … 32 | — | tunable | How many nearest agents it avoids (at most 32). |
| `horizon` | `number` | `2` | 0 … 10 | `s` | tunable | Seconds ahead it keeps clear of other agents: longer turns aside earlier and more gently. |
| `obstacleHorizon` | `number` | `0.5` | 0 … 10 | `s` | tunable | Seconds ahead it keeps clear of walls (a navmesh's edges). |
| `weight` | `number` | `0.5` | 0 … 1 | — | tunable | Its share of each avoidance with another crowd agent (0.5: half each); an agent without og.Crowd is avoided wholly by it. |

```ogt
component Crowd {
  doc "Reciprocal crowd avoidance (ORCA) for an agent: beside an og.NavAgent, each tick its velocity is the one nearest what its path and steering want that keeps it clear of its nearest agents for `horizon` seconds and of the navmesh's walls for `obstacleHorizon`, each pair of crowd agents sharing the avoidance. Every agent solves from the same start-of-tick state in a fixed order, so crowds replay identically on every host; agents meeting perfectly symmetrically (a ring swapping places) are kept from locking by a margin of up to 5% of the radius picked from the entity, so a game adds no jitter of its own. Without one an agent moves as before (og.Steering's separation only)."
  field neighbours u32 "How many nearest agents it avoids (at most 32)." {
    default 8
    min 0
    max 32
    tunable true
  }
  field horizon f32 "Seconds ahead it keeps clear of other agents: longer turns aside earlier and more gently." {
    default 2
    min 0
    max 10
    unit s
    tunable true
  }
  field obstacleHorizon f32 "Seconds ahead it keeps clear of walls (a navmesh's edges)." {
    default 0.5
    min 0
    max 10
    unit s
    tunable true
  }
  field weight f32 "Its share of each avoidance with another crowd agent (0.5: half each); an agent without og.Crowd is avoided wholly by it." {
    default 0.5
    min 0
    max 1
    tunable true
  }
}
```

<sub>`api.ogt` line 13689</sub>
