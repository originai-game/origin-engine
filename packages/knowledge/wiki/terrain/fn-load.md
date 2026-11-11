<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.terrain.load`

Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain.

```ts
og.terrain.load(world: World, path: string): Result<TerrainInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The terrain file, such as `terrain/island.ogt`. |

**Returns** `Result<TerrainInfo>`: Refused for a path that is not a cooked terrain, or a layer texture that does not load.

Binding symbol `og_terrain_load` · publish builds: kept

```ogt
fn load {
  doc "Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain."
  param world World "The world."
  param path String "The terrain file, such as `terrain/island.ogt`."
  returns Result.TerrainInfo "Refused for a path that is not a cooked terrain, or a layer texture that does not load."
}
```

<sub>Module `terrain` line 640</sub>
