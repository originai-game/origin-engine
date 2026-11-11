<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-look"></a>

### `og.load.look`

Load a look: an `.ogt` of `type look` binding a frame's art direction as data, with `name`, `extends` (a preset slug or another look), `ui` and `antialias` lines and one block per resource it sets (LookInfo.resources), its fields by name. The cook validates every field against the reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path.

See `loading §6`, `looks §1`.

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
  doc "Load a look: an `.ogt` of `type look` binding a frame's art direction as data, with `name`, `extends` (a preset slug or another look), `ui` and `antialias` lines and one block per resource it sets (LookInfo.resources), its fields by name. The cook validates every field against the reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path."
  guide "loading §6"
  guide "looks §1"
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `looks/main.ogt`."
  returns Result.Look "Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load."
}
```

<sub>Module `load` line 352</sub>
