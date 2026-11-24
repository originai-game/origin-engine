<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantFlowersDesc"></a>

### `PlantFlowersDesc` (desc struct)

Blossom/flower configuration on terminal branches.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `share` | `number` | `1` | 0 … 1 | `ratio` | — | Share of branch tips that flower (0..1). |
| `count` | `number` | `0` | ≤ 64 | — | — | Extra flowers along each terminal branch. |
| `petals` | `number` | `5` | 3 … 24 | — | — | Petals per flower. |
| `size` | `number` | `0.04` | ≥ 0.001 | `m` | — | Petal length. |
| `width` | `number` | `0.6` | 0.05 … 4 | — | — | Petal width as a share of length. |
| `cup` | `number` | `20` | 0 … 85 | `deg` | — | Petal cup lift angle. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Primary petal colour. |
| `color2` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Secondary petal colour. |
| `centerColor` | `Color` | `{ r: 1, g: 0.7, b: 0.05 }` | — | — | — | Flower centre colour. |

```ogt
struct PlantFlowersDesc {
  doc "Blossom/flower configuration on terminal branches."
  desc true
  field share f32 "Share of branch tips that flower (0..1)." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field count u32 "Extra flowers along each terminal branch." {
    max 64
  }
  field petals u32 "Petals per flower." {
    default 5
    min 3
    max 24
  }
  field size f32 "Petal length." {
    default 0.04
    min 0.001
    unit m
  }
  field width f32 "Petal width as a share of length." {
    default 0.6
    min 0.05
    max 4
  }
  field cup f32 "Petal cup lift angle." {
    default 20
    min 0
    max 85
    unit deg
  }
  field color Color "Primary petal colour." {
    default 1 1 1
  }
  field color2 Color "Secondary petal colour." {
    default 1 1 1
  }
  field centerColor Color "Flower centre colour." {
    default 1 0.7 0.05
  }
}
```

<sub>Module `procgen` line 700</sub>
