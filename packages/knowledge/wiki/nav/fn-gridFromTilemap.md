<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-gridFromTilemap"></a>

### `og.nav.gridFromTilemap`

A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another.

Its cells are the map's: row 0 at the top, so og.nav.cellAt answers what og.tilemap.cellAt does for a point and og.nav.setCost takes a tile's x and y. The choice is the grid's, saved with the world.

```ts
og.nav.gridFromTilemap(world: World, desc?: TilemapGridDesc): Result<Navigation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TilemapGridDesc` | yes | The map's entity and the blocking layers. |

**Returns** `Result<Navigation>`: Refused for an entity without an og.Tilemap or a map not loaded.

Binding symbol `og_nav_grid_from_tilemap` · publish builds: kept

```ogt
fn gridFromTilemap {
  doc "A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another."
  doc "Its cells are the map's: row 0 at the top, so og.nav.cellAt answers what og.tilemap.cellAt does for a point and og.nav.setCost takes a tile's x and y. The choice is the grid's, saved with the world."
  param world World "The world."
  param desc TilemapGridDesc "The map's entity and the blocking layers."
  returns Result.Navigation "Refused for an entity without an og.Tilemap or a map not loaded."
}
```

<sub>Module `nav` line 510</sub>
