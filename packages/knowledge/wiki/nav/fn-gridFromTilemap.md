<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-gridFromTilemap"></a>

### `og.nav.gridFromTilemap`

A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another.

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
  param world World "The world."
  param desc TilemapGridDesc "The map's entity and the blocking layers."
  returns Result.Navigation "Refused for an entity without an og.Tilemap or a map not loaded."
}
```

<sub>`api.ogt` line 14043</sub>
