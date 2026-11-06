<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-setLayer"></a>

### `og.terrain.setLayer`

Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame.

```ts
og.terrain.setLayer(world: World, data: TerrainData, index: number, layer?: TerrainLayerDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `index` | `number` | no | The layer, from 0. |
| `layer` | `TerrainLayerDesc` | yes | The layer. |

**Returns** `Result<void>`: Refused for an unknown terrain or layer.

Binding symbol `og_terrain_set_layer` · publish builds: kept

```ogt
fn setLayer {
  doc "Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame."
  param world World "The world."
  param data TerrainData "The terrain."
  param index u32 "The layer, from 0."
  param layer TerrainLayerDesc "The layer."
  returns Result.Void "Refused for an unknown terrain or layer."
}
```

<sub>`api.ogt` line 25480</sub>
