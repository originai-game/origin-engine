<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-BspResult"></a>

### `BspResult` (struct)

What og.procgen.bsp made.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rooms` | `readonly CellRect[]` | `[]` | — | — | — | One room per leaf, left before right down the tree. |
| `corridors` | `readonly CellRect[]` | `[]` | — | — | — | Corridor segments, two per split. |
| `cells` | `readonly number[]` | `[]` | — | — | — | width × height, row-major from the top: 0 wall, 1 room, 2 corridor. og.tilemap.fill writes it into a layer (map the values to tiles first). |

```ogt
struct BspResult {
  doc "What og.procgen.bsp made."
  field rooms List.CellRect "One room per leaf, left before right down the tree."
  field corridors List.CellRect "Corridor segments, two per split."
  field cells List.u32 "width × height, row-major from the top: 0 wall, 1 room, 2 corridor. og.tilemap.fill writes it into a layer (map the values to tiles first)."
}
```

<sub>Module `procgen` line 155</sub>
