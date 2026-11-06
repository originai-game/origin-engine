<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-edit"></a>

### `og.terrain.edit`

Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none).

```ts
og.terrain.edit(world: World, data: TerrainData, brush?: TerrainBrush): Result<TerrainEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `brush` | `TerrainBrush` | yes | What to do and where. |

**Returns** `Result<TerrainEdit>`: Refused for an unknown terrain, a layer beyond its layers, or a brush that misses it.

Binding symbol `og_terrain_edit` · publish builds: kept

```ogt
fn edit {
  doc "Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none)."
  param world World "The world."
  param data TerrainData "The terrain."
  param brush TerrainBrush "What to do and where."
  returns Result.TerrainEdit "Refused for an unknown terrain, a layer beyond its layers, or a brush that misses it."
}
```

<sub>`api.ogt` line 25326</sub>
