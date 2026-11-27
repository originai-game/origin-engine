<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-create"></a>

### `og.terrain.create`

Create a procedural terrain at run time from height, control and hole TypedArrays, ready for og.Terrain, og.Collider (shape heightfield), og.Grass, og.TerrainScatter, og.terrain.sample, edit and stamp.

Also known as: procedural terrain, runtime heightmap, generate terrain, create heightfield

```ts
og.terrain.create(world: World, desc?: TerrainCreateDesc): Result<TerrainInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TerrainCreateDesc` | yes | Dimensions, heights, optional control/holes and layers. |

**Returns** `Result<TerrainInfo>`: Refused for invalid dimensions, mismatched array lengths, non-finite heights or more than 16 layers.

Binding symbol `og_terrain_create` · publish builds: kept

```ogt
fn create {
  doc "Create a procedural terrain at run time from height, control and hole TypedArrays, ready for og.Terrain, og.Collider (shape heightfield), og.Grass, og.TerrainScatter, og.terrain.sample, edit and stamp."
  aliases "procedural terrain, runtime heightmap, generate terrain, create heightfield"
  param world World "The world."
  param desc TerrainCreateDesc "Dimensions, heights, optional control/holes and layers."
  returns Result.TerrainInfo "Refused for invalid dimensions, mismatched array lengths, non-finite heights or more than 16 layers."
}
```

<sub>Module `terrain` line 696</sub>
