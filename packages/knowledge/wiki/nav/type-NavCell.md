<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavCell"></a>

### `NavCell` (struct)

A grid cell, counted as the grid counts: a grid from og.nav.grid has row 0 at the bottom, rows running up from its origin; a grid from og.nav.gridFromTilemap has row 0 at the top, as og.tilemap's TileCell, so one cell means one tile in both modules.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The column (0 is the left). |
| `y` | `number` | `0` | — | — | — | The row: 0 is the bottom on a grid from og.nav.grid, the top on a grid from og.nav.gridFromTilemap. |
| `inside` | `boolean` | `false` | — | — | — | Whether the point was on the grid. |

```ogt
struct NavCell {
  doc "A grid cell, counted as the grid counts: a grid from og.nav.grid has row 0 at the bottom, rows running up from its origin; a grid from og.nav.gridFromTilemap has row 0 at the top, as og.tilemap's TileCell, so one cell means one tile in both modules."
  field x u32 "The column (0 is the left)."
  field y u32 "The row: 0 is the bottom on a grid from og.nav.grid, the top on a grid from og.nav.gridFromTilemap."
  field inside bool "Whether the point was on the grid."
}
```

<sub>Module `nav` line 384</sub>
