<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="type-TileLayerInfo"></a>

### `TileLayerInfo` (struct)

One layer of a map.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `index` | `number` | `0` | — | — | — | Its index (what drawing order and z start from). |
| `z` | `number` | `0` | — | — | — | Its sprite-layer offset from Tilemap.layer. |
| `parallax` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Its scroll factors: 1 1 moves with the world, 0.5 at half the camera's speed (far away), 0 stays with the camera. |
| `collision` | `boolean` | `false` | — | — | — | Its solid tiles collide. |
| `colliderLayers` | `number` | `0` | — | — | — | The collision layers (og.Collider.layers bits) of its colliders, from its `colliderLayers` line or Tiled int property; 0 (unset) takes Tilemap.colliderLayers. |
| `colliderMask` | `number` | `0` | — | — | — | The layers its colliders collide with (og.Collider.mask bits), from its `colliderMask` line or Tiled int property; 0 (unset) is every layer. |
| `cellColliders` | `boolean` | `false` | — | — | — | Off the orthogonal grid, whether each solid cell collides on its own (a diamond or hexagon per cell, from its `cellColliders` line or Tiled bool property) rather than each connected solid region as one collider, its outline (the default: a static map costs physics per region, not per cell). |
| `terrain` | `string` | `""` | — | — | — | The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer. |
| `visible` | `boolean` | `false` | — | — | — | Whether it is drawn. |
| `lit` | `boolean` | `false` | — | — | — | Whether og.Lighting2D lights it when its sprite layer is in the lit range (true unless the Tiled layer's bool property `lit` or the `.ogt` layer's `lit` line says false). |

```ogt
struct TileLayerInfo {
  doc "One layer of a map."
  field name String "Its name."
  field index u32 "Its index (what drawing order and z start from)."
  field z i32 "Its sprite-layer offset from Tilemap.layer."
  field parallax Vector2 "Its scroll factors: 1 1 moves with the world, 0.5 at half the camera's speed (far away), 0 stays with the camera."
  field collision bool "Its solid tiles collide."
  field colliderLayers u32 "The collision layers (og.Collider.layers bits) of its colliders, from its `colliderLayers` line or Tiled int property; 0 (unset) takes Tilemap.colliderLayers."
  field colliderMask u32 "The layers its colliders collide with (og.Collider.mask bits), from its `colliderMask` line or Tiled int property; 0 (unset) is every layer."
  field cellColliders bool "Off the orthogonal grid, whether each solid cell collides on its own (a diamond or hexagon per cell, from its `cellColliders` line or Tiled bool property) rather than each connected solid region as one collider, its outline (the default: a static map costs physics per region, not per cell)."
  field terrain String "The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer."
  field visible bool "Whether it is drawn."
  field lit bool "Whether og.Lighting2D lights it when its sprite layer is in the lit range (true unless the Tiled layer's bool property `lit` or the `.ogt` layer's `lit` line says false)."
}
```

<sub>`api.ogt` line 12625</sub>
