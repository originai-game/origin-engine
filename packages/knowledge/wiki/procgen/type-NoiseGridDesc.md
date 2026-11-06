<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-NoiseGridDesc"></a>

### `NoiseGridDesc` (desc struct)

A grid of noise samples, row-major from (x, y), `step` apart: a height field, a biome map, a cave mask.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `noise` | `FbmDesc` | `{}` | — | — | — | The noise. |
| `width` | `number` | `64` | 1 … 4096 | — | — | Samples across. |
| `height` | `number` | `64` | 1 … 4096 | — | — | Samples down. |
| `x` | `number` | `0` | — | — | — | The first sample's x. |
| `y` | `number` | `0` | — | — | — | The first sample's y. |
| `z` | `number` | `0` | — | — | — | The slice (3D noise only). |
| `step` | `number` | `1` | ≥ 0 | — | — | The distance between samples. |

```ogt
struct NoiseGridDesc {
  doc "A grid of noise samples, row-major from (x, y), `step` apart: a height field, a biome map, a cave mask."
  desc true
  field noise FbmDesc "The noise."
  field width u32 "Samples across." {
    default 64
    min 1
    max 4096
  }
  field height u32 "Samples down." {
    default 64
    min 1
    max 4096
  }
  field x f32 "The first sample's x."
  field y f32 "The first sample's y."
  field z f32 "The slice (3D noise only)."
  field step f32 "The distance between samples." {
    default 1
    min 0
  }
}
```

<sub>Module `procgen` line 37</sub>
