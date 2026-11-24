<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-BspResult"></a>

### `BspResult` (struct)

What og.procgen.bsp made.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rooms` | `readonly CellRect[]` | `[]` | — | — | — | One room per leaf, left before right down the tree. |
| `corridors` | `readonly CellRect[]` | `[]` | — | — | — | Corridor segments, two per split plus loop corridors. |
| `edges` | `readonly [number, number][]` | `[]` | — | — | — | Room adjacency graph edges `[roomA, roomB]` connecting rooms joined by corridors. |
| `cells` | `Uint8Array` | — | — | — | — | width × height, row-major from the top: 0 wall, 1 room, 2 corridor (native Uint8Array over binary wire). |

```ogt
struct BspResult {
  doc "What og.procgen.bsp made."
  field rooms List.CellRect "One room per leaf, left before right down the tree."
  field corridors List.CellRect "Corridor segments, two per split plus loop corridors."
  field edges List.Array.u32.2 "Room adjacency graph edges `[roomA, roomB]` connecting rooms joined by corridors."
  field cells Uint8Array "width × height, row-major from the top: 0 wall, 1 room, 2 corridor (native Uint8Array over binary wire)."
}
```

<sub>Module `procgen` line 213</sub>
