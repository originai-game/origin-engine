<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-tilemap"></a>

### `og.load.tilemap`

Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. Loading the same path again answers the same handles.

```ts
og.load.tilemap(world: World, path: string): Result<TilemapInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer and tilemaps own it. |
| `path` | `string` | no | The map's asset id, such as `maps/room.tmj`. |

**Returns** `Result<TilemapInfo>`: Refused for a path not in the bundle, a missing tileset or texture, or a terrain its tileset lacks.

Binding symbol `og_load_tilemap` · publish builds: kept

```ogt
fn tilemap {
  doc "Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. Loading the same path again answers the same handles."
  param world World "The world whose renderer and tilemaps own it."
  param path String "The map's asset id, such as `maps/room.tmj`."
  returns Result.TilemapInfo "Refused for a path not in the bundle, a missing tileset or texture, or a terrain its tileset lacks."
}
```

<sub>`api.ogt` line 8314</sub>
