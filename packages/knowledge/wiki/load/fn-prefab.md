<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-prefab"></a>

### `og.load.prefab`

Load a cooked file.prefab declaration: one entity tree authored once and placed many times, docs/design.md §2.5. world.instantiate spawns it as it spawns a scene, its root under the desc's root entity, each entity unnamed (place a prefab many times without name clashes) and carrying an og.PrefabLink. A scene places one with an entity's prefab: ref('module/content-id'); the cook flattens it there.

```ts
og.load.prefab(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The prefab's canonical content id, such as `base/prefabs/crate`. |

**Returns** `Result<Asset>`: The prefab, or why not (not cooked, or not a prefab).

Binding symbol `og_load_prefab` · publish builds: kept

```ogt
fn prefab {
  doc "Load a cooked file.prefab declaration: one entity tree authored once and placed many times, docs/design.md §2.5. world.instantiate spawns it as it spawns a scene, its root under the desc's root entity, each entity unnamed (place a prefab many times without name clashes) and carrying an og.PrefabLink. A scene places one with an entity's prefab: ref('module/content-id'); the cook flattens it there."
  param world World "The world that owns the asset."
  param path String "The prefab's canonical content id, such as `base/prefabs/crate`."
  returns Result.Asset "The prefab, or why not (not cooked, or not a prefab)."
}
```

<sub>Module `load` line 264</sub>
