<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="record-Tilemap"></a>

### `og.Tilemap` (component)

Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it.

Token `og.Tilemap`, registered under the schema name `og.Tilemap`; its value type is `Tilemap`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `TilemapAsset` | `0` | — | — | — | The map (og.load.tilemap's `map`); 0 is none. |
| `layer` | `number` | `-10` | — | — | — | The sprite layer of the map's layer 0; every layer draws at this plus its z (default: its index). |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha, times each layer's. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `collision` | `boolean` | `true` | — | — | — | Give the layers marked collision static colliders (rectangles merged from the solid tiles). |
| `colliderLayers` | `number` | `1` | — | — | — | The collision layers of those colliders, as bits (Collider.layers), for every collision layer of the map that names none of its own (a map layer's `colliderLayers`, with its `colliderMask`, puts a layer of grates on a one-way layer while the walls stay here). |
| `friction` | `number` | `0.5` | 0 … 2 | — | — | The colliders' friction. |
| `palette` | `Texture` | `0` | — | — | — | A palette swap of every layer (og.Sprite.palette): the tile sets' index textures (the cook's `<id>#index`) drawn through row paletteRow of this texture of N rows; 0 is none. |
| `paletteRow` | `number` | `0` | — | — | — | The row of `palette` drawn (past the last, the last). |

```ogt
component Tilemap {
  doc "Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it."
  field map TilemapAsset "The map (og.load.tilemap's `map`); 0 is none."
  field layer i32 "The sprite layer of the map's layer 0; every layer draws at this plus its z (default: its index)." {
    default -10
  }
  field color Color "A linear tint." {
    default 1 1 1
  }
  field opacity f32 "The alpha, times each layer's." {
    default 1
    min 0
    max 1
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field collision bool "Give the layers marked collision static colliders (rectangles merged from the solid tiles)." {
    default true
  }
  field colliderLayers u32 "The collision layers of those colliders, as bits (Collider.layers), for every collision layer of the map that names none of its own (a map layer's `colliderLayers`, with its `colliderMask`, puts a layer of grates on a one-way layer while the walls stay here)." {
    default 1
  }
  field friction f32 "The colliders' friction." {
    default 0.5
    min 0
    max 2
  }
  field palette Texture "A palette swap of every layer (og.Sprite.palette): the tile sets' index textures (the cook's `<id>#index`) drawn through row paletteRow of this texture of N rows; 0 is none."
  field paletteRow u32 "The row of `palette` drawn (past the last, the last)."
}
```

<sub>`api.ogt` line 12577</sub>
