<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="fn-create"></a>

### `og.tilemap.create`

Make an empty map in game code: draw it and edit it as a loaded one (an og.Tilemap with its `map`, then setTile and fill). Its handle comes from the world's own counter, so equal calls in equal order give equal handles, in a replay and after a restore. A world that makes none saves and hashes as before.

```ts
og.tilemap.create(world: World, desc?: TilemapDesc): Result<TilemapInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TilemapDesc` | yes | Its size, tiles, orientation, tilesets and layers. |

**Returns** `Result<TilemapInfo>`: Refused for a size of 0 or past 16,777,216 cells, a tileset the bundle lacks, a layer naming a tileset index or terrain that does not exist, two layers of one name, or a terrain layer on a staggered or hex map.

Binding symbol `og_tilemap_create` · publish builds: kept

```ogt
fn create {
  doc "Make an empty map in game code: draw it and edit it as a loaded one (an og.Tilemap with its `map`, then setTile and fill). Its handle comes from the world's own counter, so equal calls in equal order give equal handles, in a replay and after a restore. A world that makes none saves and hashes as before."
  param world World "The world."
  param desc TilemapDesc "Its size, tiles, orientation, tilesets and layers."
  returns Result.TilemapInfo "Refused for a size of 0 or past 16,777,216 cells, a tileset the bundle lacks, a layer naming a tileset index or terrain that does not exist, two layers of one name, or a terrain layer on a staggered or hex map."
}
```

<sub>`api.ogt` line 12769</sub>
