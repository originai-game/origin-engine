<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PoissonDesc"></a>

### `PoissonDesc` (desc struct)

Points no closer than `radius` to each other, filling an area evenly with no grid look (Bridson 2007): trees in a forest, loot in a field.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seed` | `number` | `0` | — | — | — | The stream's seed: `world.resource(og.Rng)`'s value `.fork("trees").seed`. |
| `width` | `number` | `0` | ≥ 0 | `m` | — | The area's width, from x 0. |
| `height` | `number` | `0` | ≥ 0 | `m` | — | The area's height, from y 0. |
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The least distance between two points. |
| `attempts` | `number` | `30` | 1 … 256 | — | — | Candidates tried around a point before it is done (more fills tighter). |
| `maxPoints` | `number` | `4096` | 0 … 1048576 | — | — | The most points made. |

```ogt
struct PoissonDesc {
  doc "Points no closer than `radius` to each other, filling an area evenly with no grid look (Bridson 2007): trees in a forest, loot in a field."
  desc true
  field seed u32 "The stream's seed: `world.resource(og.Rng)`'s value `.fork(\"trees\").seed`."
  field width f32 "The area's width, from x 0." {
    min 0
    unit m
  }
  field height f32 "The area's height, from y 0." {
    min 0
    unit m
  }
  field radius f32 "The least distance between two points." {
    default 1
    min 0
    unit m
  }
  field attempts u32 "Candidates tried around a point before it is done (more fills tighter)." {
    default 30
    min 1
    max 256
  }
  field maxPoints u32 "The most points made." {
    default 4096
    min 0
    max 1048576
  }
}
```

<sub>`api.ogt` line 1853</sub>
