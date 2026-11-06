<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-look"></a>

### `og.load.look`

Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `ui` (a UI style: a built-in style's slug, such as `cartoon`, or the game's own style file's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract's ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera) with that resource's fields by name, an asset id where a field is a handle (`lut "luts/warm.cube"`, `palette "palettes/dusk.hex"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path.

```ts
og.load.look(world: World, path: string): Result<Look>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `looks/main.ogt`. |

**Returns** `Result<Look>`: Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load.

Binding symbol `og_load_look` · publish builds: kept

```ogt
fn look {
  doc "Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `ui` (a UI style: a built-in style's slug, such as `cartoon`, or the game's own style file's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract's ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera) with that resource's fields by name, an asset id where a field is a handle (`lut \"luts/warm.cube\"`, `palette \"palettes/dusk.hex\"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `looks/main.ogt`."
  returns Result.Look "Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load."
}
```

<sub>`api.ogt` line 8243</sub>
