<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-unstamp"></a>

### `og.terrain.unstamp`

Take a stamp back: the terrain is rebuilt from its cook with every other edit and stamp replayed in order, as if it had never been made.

```ts
og.terrain.unstamp(world: World, data: TerrainData, stamp: number): Result<TerrainEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `stamp` | `number` | no | The id og.terrain.stamp answered. |

**Returns** `Result<TerrainEdit>`: Refused for an unknown terrain or a stamp it does not hold.

Binding symbol `og_terrain_unstamp` · publish builds: kept

```ogt
fn unstamp {
  doc "Take a stamp back: the terrain is rebuilt from its cook with every other edit and stamp replayed in order, as if it had never been made."
  param world World "The world."
  param data TerrainData "The terrain."
  param stamp u32 "The id og.terrain.stamp answered."
  returns Result.TerrainEdit "Refused for an unknown terrain or a stamp it does not hold."
}
```

<sub>Module `terrain` line 690</sub>
