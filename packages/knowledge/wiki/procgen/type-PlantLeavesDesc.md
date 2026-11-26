<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantLeavesDesc"></a>

### `PlantLeavesDesc` (desc struct)

Leaf foliage configuration on terminal branches.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `PlantLeafShape` | `"blade"` | — | — | — | Leaf geometry kind. |
| `count` | `number` | `10` | ≤ 256 | — | — | Leaves per terminal branch. |
| `start` | `number` | `0.1` | 0 … 1 | `ratio` | — | Start fraction along terminal branch (0..1). |
| `end` | `number` | `1` | 0 … 1 | `ratio` | — | End fraction along terminal branch (0..1). |
| `angle` | `number` | `45` | 0 … 175 | `deg` | — | Leaf angle from branch. |
| `facing` | `number` | `0.8` | 0 … 1 | `ratio` | — | How much leaves turn flat towards the sky (0..1). |
| `rows` | `number` | `0` | ≤ 16 | — | — | Leaf phyllotaxis rows around the branch (0: spiral/random, 2: frond). |
| `size` | `number` | `0.4` | ≥ 0.001 | `m` | — | Leaf length. |
| `width` | `number` | `0.5` | 0.05 … 4 | — | — | Leaf width as a share of length. |
| `sizeVariance` | `number` | `0.4` | 0 … 1 | `ratio` | — | Per-leaf random size variation share (0..1). |
| `color` | `Color` | `{ r: 0.09, g: 0.24, b: 0.04 }` | — | — | — | Primary linear leaf colour. |
| `color2` | `Color` | `{ r: 0.17, g: 0.33, b: 0.06 }` | — | — | — | Secondary linear leaf colour. |
| `colorVariation` | `number` | `0.3` | 0 … 1 | `ratio` | — | Per-leaf brightness variation (0..1). |

```ogt
struct PlantLeavesDesc {
  doc "Leaf foliage configuration on terminal branches."
  desc true
  field shape PlantLeafShape "Leaf geometry kind."
  field count u32 "Leaves per terminal branch." {
    default 10
    max 256
  }
  field start f32 "Start fraction along terminal branch (0..1)." {
    default 0.1
    min 0
    max 1
    unit ratio
  }
  field end f32 "End fraction along terminal branch (0..1)." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field angle f32 "Leaf angle from branch." {
    default 45
    min 0
    max 175
    unit deg
  }
  field facing f32 "How much leaves turn flat towards the sky (0..1)." {
    default 0.8
    min 0
    max 1
    unit ratio
  }
  field rows u32 "Leaf phyllotaxis rows around the branch (0: spiral/random, 2: frond)." {
    max 16
  }
  field size f32 "Leaf length." {
    default 0.4
    min 0.001
    unit m
  }
  field width f32 "Leaf width as a share of length." {
    default 0.5
    min 0.05
    max 4
  }
  field sizeVariance f32 "Per-leaf random size variation share (0..1)." {
    default 0.4
    min 0
    max 1
    unit ratio
  }
  field color Color "Primary linear leaf colour." {
    default 0.09 0.24 0.04
  }
  field color2 Color "Secondary linear leaf colour." {
    default 0.17 0.33 0.06
  }
  field colorVariation f32 "Per-leaf brightness variation (0..1)." {
    default 0.3
    min 0
    max 1
    unit ratio
  }
}
```

<sub>Module `procgen` line 650</sub>
