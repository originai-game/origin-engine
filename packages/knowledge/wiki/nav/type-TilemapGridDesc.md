<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-TilemapGridDesc"></a>

### `TilemapGridDesc` (desc struct)

A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Tilemap. |
| `layers` | `readonly string[]` | `[]` | — | — | — | The layers whose solid tiles block; empty: every collision layer. |
| `inflate` | `number` | `0` | ≤ 8 | — | — | Also block this many cells around each blocked one (wide agents). |

```ogt
struct TilemapGridDesc {
  doc "A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile."
  desc true
  field entity Entity "The entity with the og.Tilemap."
  field layers List.String "The layers whose solid tiles block; empty: every collision layer."
  field inflate u32 "Also block this many cells around each blocked one (wide agents)." {
    max 8
  }
}
```

<sub>Module `nav` line 374</sub>
