<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-WfcDesc"></a>

### `WfcDesc` (desc struct)

Wave function collapse, the simple tiled model: a grid filled with tiles whose edges all fit. The cell with the fewest tiles left (ties: the first) is decided by a weighted draw and the choice spreads to its neighbours; a dead end restarts on a fork of the seed.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seed` | `number` | `0` | — | — | — | The stream's seed: `world.resource(og.Rng)`'s value `.fork("village").seed`. |
| `width` | `number` | `16` | 1 … 1024 | — | — | Columns. |
| `height` | `number` | `16` | 1 … 1024 | — | — | Rows. |
| `tiles` | `readonly WfcTile[]` | `[]` | — | — | — | The tiles (with rotations, at most 1,024 variants). |
| `fixed` | `readonly WfcCell[]` | `[]` | — | — | — | Cells decided before the first draw. |
| `wrap` | `boolean` | `false` | — | — | — | The edges wrap round (a tiling texture). |
| `attempts` | `number` | `10` | 1 … 1000 | — | — | Tries before refusing. |

```ogt
struct WfcDesc {
  doc "Wave function collapse, the simple tiled model: a grid filled with tiles whose edges all fit. The cell with the fewest tiles left (ties: the first) is decided by a weighted draw and the choice spreads to its neighbours; a dead end restarts on a fork of the seed."
  desc true
  field seed u32 "The stream's seed: `world.resource(og.Rng)`'s value `.fork(\"village\").seed`."
  field width u32 "Columns." {
    default 16
    min 1
    max 1024
  }
  field height u32 "Rows." {
    default 16
    min 1
    max 1024
  }
  field tiles List.WfcTile "The tiles (with rotations, at most 1,024 variants)."
  field fixed List.WfcCell "Cells decided before the first draw."
  field wrap bool "The edges wrap round (a tiling texture)."
  field attempts u32 "Tries before refusing." {
    default 10
    min 1
    max 1000
  }
}
```

<sub>`api.ogt` line 1975</sub>
