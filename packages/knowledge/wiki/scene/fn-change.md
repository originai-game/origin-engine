<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="fn-change"></a>

### `og.scene.change`

Replace the current scene with this one as the next tick starts: the current scene's root and every entity under it are despawned (the boot scene's own entities, with their descendants, for the first change), then the new scene spawns under a new root and og.SceneChanged is sent. Entities the game spawned outside a scene stay. It preloads the scene, and the previous scene's preload is released. Systems may call it.

Also known as: change scene, switch level, load level, Unity SceneManager.LoadScene, Godot change_scene_to_file

```ts
og.scene.change(world: World, scene: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `scene` | `string` | no | A GameDesc.scenes name or the scene's asset id. |

**Returns** `Result<Asset>`: The scene's handle, or why not: an unknown scene. A scene whose entity names clash with entities that stay is not put in place: the current one stays, with a warning in the engine log.

Binding symbol `og_scene_change` · publish builds: kept

```ogt
fn change {
  doc "Replace the current scene with this one as the next tick starts: the current scene's root and every entity under it are despawned (the boot scene's own entities, with their descendants, for the first change), then the new scene spawns under a new root and og.SceneChanged is sent. Entities the game spawned outside a scene stay. It preloads the scene, and the previous scene's preload is released. Systems may call it."
  aliases "change scene, switch level, load level, Unity SceneManager.LoadScene, Godot change_scene_to_file"
  param world World "The world."
  param scene String "A GameDesc.scenes name or the scene's asset id."
  returns Result.Asset "The scene's handle, or why not: an unknown scene. A scene whose entity names clash with entities that stay is not put in place: the current one stays, with a warning in the engine log."
}
```

<sub>Module `scene` line 72</sub>
