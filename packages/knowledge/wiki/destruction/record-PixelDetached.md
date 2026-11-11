<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-PixelDetached"></a>

### `og.PixelDetached` (event)

Cells of an og.PixelTerrain (or of an og.PixelPiece fracture2d broke) broke loose during the tick, read the tick after: one for each piece, in the order they were cut, for its sound, its dust, its score.

Token `og.PixelDetached`, registered under the schema name `og.PixelDetached`; its value type is `PixelDetached`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `terrain` | `Entity` | `0` | — | — | — | The og.PixelTerrain. |
| `piece` | `Entity` | `0` | — | — | — | The og.PixelPiece it became; 0 for an island under minPiece, which vanished. |
| `cells` | `number` | `0` | — | — | — | Its cells. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Its centre. |
| `cause` | `number` | `0` | — | — | — | What broke it loose: CAUSE_CARVE, CAUSE_EXPLOSION or CAUSE_FRACTURE. |

```ogt
event PixelDetached {
  doc "Cells of an og.PixelTerrain (or of an og.PixelPiece fracture2d broke) broke loose during the tick, read the tick after: one for each piece, in the order they were cut, for its sound, its dust, its score."
  field terrain Entity "The og.PixelTerrain."
  field piece Entity "The og.PixelPiece it became; 0 for an island under minPiece, which vanished."
  field cells u32 "Its cells."
  field point Vector3 "Its centre." {
    unit m
    worldspace world
  }
  field cause u32 "What broke it loose: CAUSE_CARVE, CAUSE_EXPLOSION or CAUSE_FRACTURE."
}
```

<sub>Module `destruction` line 407</sub>
