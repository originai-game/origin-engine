<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-WaterLine"></a>

### `og.WaterLine` (component)

2D shallow water: a width × depth pool at the entity's y, centred on its x. Waves travel at sqrt(gravity × depth), with CFL substeps and reflecting walls. Bodies crossing splash it and float by buoyancy; og.Water2D draws it.

Token `og.WaterLine`, registered under the schema name `og.WaterLine`; its value type is `WaterLine`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `12` | ≥ 0.1 | `m` | tunable | The pool's width. |
| `depth` | `number` | `3` | ≥ 0 | `m` | tunable | The pool's depth below the surface. |
| `columns` | `number` | `96` | 4 … 512 | — | — | Grid points along the surface. |
| `damping` | `number` | `2.5` | ≥ 0 | `perSecond` | tunable | Exponential horizontal momentum damping per second; zero preserves motion. |
| `splash` | `number` | `0.04` | ≥ 0 | — | tunable | How strongly crossing bodies splash. |
| `buoyancy` | `number` | `12` | ≥ 0 | — | tunable | The upward force on a submerged body per square metre of its box under water (0: bodies sink). |
| `drag` | `number` | `2` | ≥ 0 | `perSecond` | tunable | How much a submerged body's velocity is slowed. |

```ogt
component WaterLine {
  doc "2D shallow water: a width × depth pool at the entity's y, centred on its x. Waves travel at sqrt(gravity × depth), with CFL substeps and reflecting walls. Bodies crossing splash it and float by buoyancy; og.Water2D draws it."
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
  field columns u32 "Grid points along the surface." {
    default 96
    min 4
    max 512
  }
  field damping f32 "Exponential horizontal momentum damping per second; zero preserves motion." {
    default 2.5
    min 0
    unit perSecond
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

<sub>Module `physics` line 2058</sub>
