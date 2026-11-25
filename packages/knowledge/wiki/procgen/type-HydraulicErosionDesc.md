<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-HydraulicErosionDesc"></a>

### `HydraulicErosionDesc` (desc struct)

Droplet-based hydraulic erosion pass (deterministic seeded drops).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `drops` | `number` | `40000` | 0 … 2000000 | — | — | Number of simulated rain drops. |
| `lifetime` | `number` | `30` | 1 … 256 | — | — | Maximum simulation steps per drop. |
| `radius` | `number` | `3` | 1 … 12 | — | — | Erosion brush radius in height samples. |
| `inertia` | `number` | `0.05` | 0 … 0.99 | — | — | Directional momentum retained per step. |
| `capacity` | `number` | `4` | 0.01 … 64 | — | — | Sediment carrying capacity factor. |
| `deposition` | `number` | `0.3` | 0.01 … 1 | — | — | Excess sediment deposition speed. |
| `erosion` | `number` | `0.3` | 0.01 … 1 | — | — | Terrain erosion speed. |
| `evaporation` | `number` | `0.01` | 0.001 … 0.5 | — | — | Water evaporation fraction per step. |
| `gravity` | `number` | `4` | 0.1 … 32 | — | — | Downslope acceleration factor. |

```ogt
struct HydraulicErosionDesc {
  doc "Droplet-based hydraulic erosion pass (deterministic seeded drops)."
  desc true
  field drops u32 "Number of simulated rain drops." {
    default 40000
    min 0
    max 2000000
  }
  field lifetime u32 "Maximum simulation steps per drop." {
    default 30
    min 1
    max 256
  }
  field radius u32 "Erosion brush radius in height samples." {
    default 3
    min 1
    max 12
  }
  field inertia f32 "Directional momentum retained per step." {
    default 0.05
    min 0
    max 0.99
  }
  field capacity f32 "Sediment carrying capacity factor." {
    default 4
    min 0.01
    max 64
  }
  field deposition f32 "Excess sediment deposition speed." {
    default 0.3
    min 0.01
    max 1
  }
  field erosion f32 "Terrain erosion speed." {
    default 0.3
    min 0.01
    max 1
  }
  field evaporation f32 "Water evaporation fraction per step." {
    default 0.01
    min 0.001
    max 0.5
  }
  field gravity f32 "Downslope acceleration factor." {
    default 4
    min 0.1
    max 32
  }
}
```

<sub>Module `procgen` line 296</sub>
