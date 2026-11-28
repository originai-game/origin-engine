<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-look"></a>

### `og.load.look`

Load a cooked file.look TypeScript declaration by canonical content ID. Its name, extends (preset slugs or typed look refs), ui, antialias and reflected resource patches bind a frame's art direction as data. The cook validates every field and flattens extends into a native binary payload. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per ID.

```ts
og.load.look(world: World, path: string): Result<Look>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The canonical content id, such as `base/looks/main`. |

**Returns** `Result<Look>`: Refused for a content ID not declared in the bundle or an asset it names that cannot load.

Binding symbol `og_load_look` · publish builds: kept

```ogt
fn look {
  doc "Load a cooked file.look TypeScript declaration by canonical content ID. Its name, extends (preset slugs or typed look refs), ui, antialias and reflected resource patches bind a frame's art direction as data. The cook validates every field and flattens extends into a native binary payload. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per ID."
  param world World "The world whose renderer owns it."
  param path String "The canonical content id, such as `base/looks/main`."
  returns Result.Look "Refused for a content ID not declared in the bundle or an asset it names that cannot load."
}
```

<sub>Module `load` line 424</sub>
