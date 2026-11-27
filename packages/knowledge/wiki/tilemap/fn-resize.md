<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="fn-resize"></a>

### `og.tilemap.resize`

Change a map's size (a loaded one too: from then on it is a run-time map, saved whole). Every og.Tilemap showing it keeps its edits, moved with the cells by (x, y); cells past the new edges are dropped, new ones are empty, terrain re-tiles, colliders rebuild at the og.tilemap system and its objects (TilemapInfo.objects) move with the cells.

```ts
og.tilemap.resize(world: World, desc?: ResizeDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ResizeDesc` | yes | The map, its new size and the shift. |

**Returns** `Result<void>`: Refused for an unknown map, a size of 0 or past 16,777,216 cells, or an odd shift along a staggered or hex map's shifted axis (it would change which rows are shifted).

Binding symbol `og_tilemap_resize` · publish builds: kept

```ogt
fn resize {
  doc "Change a map's size (a loaded one too: from then on it is a run-time map, saved whole). Every og.Tilemap showing it keeps its edits, moved with the cells by (x, y); cells past the new edges are dropped, new ones are empty, terrain re-tiles, colliders rebuild at the og.tilemap system and its objects (TilemapInfo.objects) move with the cells."
  param world World "The world."
  param desc ResizeDesc "The map, its new size and the shift."
  returns Result.Void "Refused for an unknown map, a size of 0 or past 16,777,216 cells, or an odd shift along a staggered or hex map's shifted axis (it would change which rows are shifted)."
}
```

<sub>Module `tilemap` line 216</sub>
