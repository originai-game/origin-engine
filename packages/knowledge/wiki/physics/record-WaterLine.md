<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-WaterLine"></a>

### `og.WaterLine` (component)

2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it.

Token `og.WaterLine`, registered under the schema name `og.WaterLine`; its value type is `WaterLine`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `12` | ≥ 0.1 | `m` | tunable | The pool's width. |
| `depth` | `number` | `3` | ≥ 0 | `m` | tunable | The pool's depth below the surface. |
| `columns` | `number` | `96` | 4 … 512 | — | — | Springs along the surface. |
| `tension` | `number` | `60` | ≥ 0 | — | tunable | How hard each spring pulls back to rest. |
| `damping` | `number` | `2.5` | ≥ 0 | — | tunable | How fast a spring's motion dies. |
| `spread` | `number` | `0.25` | 0 … 0.5 | — | tunable | How much of a spring's height its neighbours feel each pass (waves travel). |
| `splash` | `number` | `0.04` | ≥ 0 | — | tunable | How strongly crossing bodies splash. |
| `buoyancy` | `number` | `12` | ≥ 0 | — | tunable | The upward force on a submerged body per square metre of its box under water (0: bodies sink). |
| `drag` | `number` | `2` | ≥ 0 | `perSecond` | tunable | How much a submerged body's velocity is slowed. |

```ogt
component WaterLine {
  doc "2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it."
  field width f32 "The pool's width." {
    default 12
    min 0.1
    unit m
    tunable true
  }
  field depth f32 "The pool's depth below the surface." {
    default 3
    min 0
    unit m
    tunable true
  }
  field columns u32 "Springs along the surface." {
    default 96
    min 4
    max 512
  }
  field tension f32 "How hard each spring pulls back to rest." {
    default 60
    min 0
    tunable true
  }
  field damping f32 "How fast a spring's motion dies." {
    default 2.5
    min 0
    tunable true
  }
  field spread f32 "How much of a spring's height its neighbours feel each pass (waves travel)." {
    default 0.25
    min 0
    max 0.5
    tunable true
  }
  field splash f32 "How strongly crossing bodies splash." {
    default 0.04
    min 0
    tunable true
  }
  field buoyancy f32 "The upward force on a submerged body per square metre of its box under water (0: bodies sink)." {
    default 12
    min 0
    tunable true
  }
  field drag f32 "How much a submerged body's velocity is slowed." {
    default 2
    min 0
    unit perSecond
    tunable true
  }
}
```

<sub>`api.ogt` line 10029</sub>
