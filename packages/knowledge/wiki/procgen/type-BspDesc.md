<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-BspDesc"></a>

### `BspDesc` (desc struct)

A dungeon by binary space partition: the area is split in two along its longer side, again and again, each leaf gets a room and every split's halves are joined by an L-shaped corridor.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seed` | `number` | `0` | — | — | — | The stream's seed: `world.resource(og.Rng)`'s value `.fork("dungeon").seed`. |
| `width` | `number` | `48` | 1 … 4096 | — | — | Columns. |
| `height` | `number` | `32` | 1 … 4096 | — | — | Rows. |
| `minRoom` | `number` | `4` | ≥ 1 | — | — | A room's least side, in cells. |
| `maxRoom` | `number` | `12` | ≥ 1 | — | — | A room's greatest side; an area larger than it with its padding is always split. |
| `depth` | `number` | `5` | ≤ 16 | — | — | The most splits, down any branch (up to 2^depth rooms). |
| `padding` | `number` | `1` | — | — | — | Wall cells kept between a room and its leaf's edge. |
| `corridor` | `number` | `1` | ≥ 1 | — | — | Corridor width, in cells. |

```ogt
struct BspDesc {
  doc "A dungeon by binary space partition: the area is split in two along its longer side, again and again, each leaf gets a room and every split's halves are joined by an L-shaped corridor."
  desc true
  field seed u32 "The stream's seed: `world.resource(og.Rng)`'s value `.fork(\"dungeon\").seed`."
  field width u32 "Columns." {
    default 48
    min 1
    max 4096
  }
  field height u32 "Rows." {
    default 32
    min 1
    max 4096
  }
  field minRoom u32 "A room's least side, in cells." {
    default 4
    min 1
  }
  field maxRoom u32 "A room's greatest side; an area larger than it with its padding is always split." {
    default 12
    min 1
  }
  field depth u32 "The most splits, down any branch (up to 2^depth rooms)." {
    default 5
    max 16
  }
  field padding u32 "Wall cells kept between a room and its leaf's edge." {
    default 1
  }
  field corridor u32 "Corridor width, in cells." {
    default 1
    min 1
  }
}
```

<sub>`api.ogt` line 1896</sub>
