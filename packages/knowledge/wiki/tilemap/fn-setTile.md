<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="fn-setTile"></a>

### `og.tilemap.setTile`

Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps.

```ts
og.tilemap.setTile(world: World, desc?: SetTileDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SetTileDesc` | yes | The map, the layer, the cell and the tile. |

**Returns** `Result<void>`: Refused for an entity whose map is not adopted yet, an unknown layer, a cell outside the map or a tile the tileset lacks.

Binding symbol `og_tilemap_set_tile` · publish builds: kept

```ogt
fn setTile {
  doc "Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps."
  param world World "The world."
  param desc SetTileDesc "The map, the layer, the cell and the tile."
  returns Result.Void "Refused for an entity whose map is not adopted yet, an unknown layer, a cell outside the map or a tile the tileset lacks."
}
```

<sub>`api.ogt` line 12684</sub>
