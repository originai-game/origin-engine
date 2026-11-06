<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="type-TileLayerDesc"></a>

### `TileLayerDesc` (desc struct)

One layer of a map og.tilemap.create makes (empty).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name (unique in the map). |
| `tileset` | `number` | `0` | — | — | — | The index of its tileset in TilemapDesc.tilesets. |
| `terrain` | `string` | `""` | — | — | — | An auto-tiling terrain of that tileset: a terrain layer (cells 1 or 0); empty for a tile layer. |
| `z` | `number` | `0` | — | — | — | Its sprite-layer offset from Tilemap.layer. |
| `collision` | `boolean` | `false` | — | — | — | Its solid tiles collide. |
| `colliderLayers` | `number` | `0` | — | — | — | Its colliders' collision layers (0: Tilemap.colliderLayers). |
| `colliderMask` | `number` | `0` | — | — | — | The layers its colliders collide with (0: every layer). |
| `cellColliders` | `boolean` | `false` | — | — | — | Off the orthogonal grid, one collider per solid cell (a diamond or hexagon) instead of one per connected solid region (its outline, the default). |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
struct TileLayerDesc {
  doc "One layer of a map og.tilemap.create makes (empty)."
  desc true
  field name String "Its name (unique in the map)."
  field tileset u32 "The index of its tileset in TilemapDesc.tilesets."
  field terrain String "An auto-tiling terrain of that tileset: a terrain layer (cells 1 or 0); empty for a tile layer."
  field z i32 "Its sprite-layer offset from Tilemap.layer."
  field collision bool "Its solid tiles collide."
  field colliderLayers u32 "Its colliders' collision layers (0: Tilemap.colliderLayers)."
  field colliderMask u32 "The layers its colliders collide with (0: every layer)."
  field cellColliders bool "Off the orthogonal grid, one collider per solid cell (a diamond or hexagon) instead of one per connected solid region (its outline, the default)."
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>Module `tilemap` line 106</sub>
