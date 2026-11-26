<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-ThermalErosionDesc"></a>

### `ThermalErosionDesc` (desc struct)

CFL-stable mass-conserving talus thermal erosion pass.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `talus` | `number` | `35` | 0 … 89 | `deg` | — | Angle of repose above which rock/soil slumps. |
| `iterations` | `number` | `20` | 0 … 10000 | — | — | Relaxation passes. |
| `rate` | `number` | `0.5` | 0 … 1 | — | — | Fraction of excess height moved per pass. |

```ogt
struct ThermalErosionDesc {
  doc "CFL-stable mass-conserving talus thermal erosion pass."
  desc true
  field talus f32 "Angle of repose above which rock/soil slumps." {
    default 35
    min 0
    max 89
    unit deg
  }
  field iterations u32 "Relaxation passes." {
    default 20
    min 0
    max 10000
  }
  field rate f32 "Fraction of excess height moved per pass." {
    default 0.5
    min 0
    max 1
  }
}
```

<sub>Module `procgen` line 346</sub>
