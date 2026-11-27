<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-BiomeRuleDesc"></a>

### `BiomeRuleDesc` (desc struct)

One biome painting rule mapping altitude, slope and moisture bands onto a terrain layer index.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `layer` | `number` | `0` | ≤ 15 | — | — | Target terrain layer index (0 to 15). |
| `weight` | `number` | `1` | 0 … 1 | — | — | Relative rule weight. |
| `altitudeMin` | `number` | `-1000000` | — | `m` | — | Minimum altitude. |
| `altitudeMax` | `number` | `1000000` | — | `m` | — | Maximum altitude. |
| `altitudeFade` | `number` | `2` | 0 … 1000000 | `m` | — | Fade band outside altitude range. |
| `slopeMin` | `number` | `0` | 0 … 90 | `deg` | — | Minimum slope. |
| `slopeMax` | `number` | `90` | 0 … 90 | `deg` | — | Maximum slope. |
| `slopeFade` | `number` | `4` | 0 … 90 | `deg` | — | Fade band outside slope range. |
| `moistureMin` | `number` | `0` | 0 … 1 | — | — | Minimum moisture (0 to 1). |
| `moistureMax` | `number` | `1` | 0 … 1 | — | — | Maximum moisture (0 to 1). |
| `moistureFade` | `number` | `0.1` | 0 … 1 | — | — | Fade band outside moisture range. |

```ogt
struct BiomeRuleDesc {
  doc "One biome painting rule mapping altitude, slope and moisture bands onto a terrain layer index."
  desc true
  field layer u32 "Target terrain layer index (0 to 15)." {
    max 15
  }
  field weight f32 "Relative rule weight." {
    default 1
    min 0
    max 1
  }
  field altitudeMin f32 "Minimum altitude." {
    default -1000000
    unit m
  }
  field altitudeMax f32 "Maximum altitude." {
    default 1000000
    unit m
  }
  field altitudeFade f32 "Fade band outside altitude range." {
    default 2
    min 0
    max 1000000
    unit m
  }
  field slopeMin f32 "Minimum slope." {
    min 0
    max 90
    unit deg
  }
  field slopeMax f32 "Maximum slope." {
    default 90
    min 0
    max 90
    unit deg
  }
  field slopeFade f32 "Fade band outside slope range." {
    default 4
    min 0
    max 90
    unit deg
  }
  field moistureMin f32 "Minimum moisture (0 to 1)." {
    min 0
    max 1
  }
  field moistureMax f32 "Maximum moisture (0 to 1)." {
    default 1
    min 0
    max 1
  }
  field moistureFade f32 "Fade band outside moisture range." {
    default 0.1
    min 0
    max 1
  }
}
```

<sub>Module `procgen` line 425</sub>
