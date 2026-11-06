<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-layer"></a>

### `og.terrain.layer`

A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it).

```ts
og.terrain.layer(world: World, data: TerrainData, index: number): Result<TerrainLayerDesc>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `index` | `number` | no | The layer, from 0. |

**Returns** `Result<TerrainLayerDesc>`: Refused for an unknown terrain or layer.

Binding symbol `og_terrain_layer` · publish builds: kept

```ogt
fn layer {
  doc "A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it)."
  param world World "The world."
  param data TerrainData "The terrain."
  param index u32 "The layer, from 0."
  returns Result.TerrainLayerDesc "Refused for an unknown terrain or layer."
}
```

<sub>`api.ogt` line 25473</sub>
