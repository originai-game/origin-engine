<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-MoistureDesc"></a>

### `MoistureDesc` (desc struct)

Moisture field weights for biome painting.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `number` | `1` | 0 … 1000000 | — | — | Weight of drainage flow accumulation. |
| `scale` | `number` | `64` | 0.001 … 1000000000 | — | — | Flow count at which moisture reaches saturation. |
| `noise` | `number` | `0.5` | 0 … 1000000 | — | — | Weight of procedural moisture noise. |
| `frequency` | `number` | `6` | 0.001 … 10000 | — | — | Moisture noise frequency across the terrain. |

```ogt
struct MoistureDesc {
  doc "Moisture field weights for biome painting."
  desc true
  field flow f32 "Weight of drainage flow accumulation." {
    default 1
    min 0
    max 1000000
  }
  field scale f32 "Flow count at which moisture reaches saturation." {
    default 64
    min 0.001
    max 1000000000
  }
  field noise f32 "Weight of procedural moisture noise." {
    default 0.5
    min 0
    max 1000000
  }
  field frequency f32 "Moisture noise frequency across the terrain." {
    default 6
    min 0.001
    max 10000
  }
}
```

<sub>Module `procgen` line 388</sub>
