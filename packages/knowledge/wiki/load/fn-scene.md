<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-scene"></a>

### `og.load.scene`

Load a cooked file.scene declaration. world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning. Typed content references load through their registered columns.

```ts
og.load.scene(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The scene's canonical content id, such as `base/scenes/main`. |

**Returns** `Result<Asset>`: The stable handle, pending while its blob is fetched. Query status; error gives a failed load's reason.

Binding symbol `og_load_scene` · publish builds: kept

```ogt
fn scene {
  doc "Load a cooked file.scene declaration. world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning. Typed content references load through their registered columns."
  param world World "The world that owns the asset."
  param path String "The scene's canonical content id, such as `base/scenes/main`."
  returns Result.Asset "The stable handle, pending while its blob is fetched. Query status; error gives a failed load's reason."
}
```

<sub>Module `load` line 257</sub>
