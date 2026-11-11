<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-stamp"></a>

### `og.terrain.stamp`

Carve a terrain under a mesh (its og.TerrainStamp's), or cut holes under it: the triangles are rasterised from above over the height samples, deterministically, and what changed is world state like og.terrain.edit's (a snapshot keeps it, the world hash covers it, a restore replays it, and the replay needs only the log, not the mesh). The renderer, physics and og.terrain.sample see the change from the next frame and tick. Answers the stamp's id in TerrainEdit.stamp.

```ts
og.terrain.stamp(world: World, desc?: TerrainStampDesc): Result<TerrainEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TerrainStampDesc` | yes | The terrain, the mesh, where both stand, and what to do. |

**Returns** `Result<TerrainEdit>`: Refused for an unknown terrain, a geometry that is not loaded, or a mesh that covers none of the terrain.

Binding symbol `og_terrain_stamp` · publish builds: kept

```ogt
fn stamp {
  doc "Carve a terrain under a mesh (its og.TerrainStamp's), or cut holes under it: the triangles are rasterised from above over the height samples, deterministically, and what changed is world state like og.terrain.edit's (a snapshot keeps it, the world hash covers it, a restore replays it, and the replay needs only the log, not the mesh). The renderer, physics and og.terrain.sample see the change from the next frame and tick. Answers the stamp's id in TerrainEdit.stamp."
  param world World "The world."
  param desc TerrainStampDesc "The terrain, the mesh, where both stand, and what to do."
  returns Result.TerrainEdit "Refused for an unknown terrain, a geometry that is not loaded, or a mesh that covers none of the terrain."
}
```

<sub>Module `terrain` line 680</sub>
