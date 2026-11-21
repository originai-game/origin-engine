<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="fn-preload"></a>

### `og.scene.preload`

Load a scene and what it spawns with (its models, their textures and pipelines, its sub-scenes and prefabs) without placing it, and ask for every streaming group it declares (og.stream.request) until og.scene.release, so it is fetched and prepared while the current scene plays and is drawn whole the frame it is placed. Calling it again does nothing more. Systems may call it.

See `loading §12`.

Also known as: preload level, load the next scene in the background, async scene load, Unity LoadSceneAsync, Godot ResourceLoader.load_threaded_request

```ts
og.scene.preload(world: World, scene: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `scene` | `string` | no | A GameDesc.scenes name or the scene's asset id. |

**Returns** `Result<Asset>`: The scene's handle, or why not: no such scene (the reason lists GameDesc.scenes), or not cooked.

Binding symbol `og_scene_preload` · publish builds: kept

```ogt
fn preload {
  doc "Load a scene and what it spawns with (its models, their textures and pipelines, its sub-scenes and prefabs) without placing it, and ask for every streaming group it declares (og.stream.request) until og.scene.release, so it is fetched and prepared while the current scene plays and is drawn whole the frame it is placed. Calling it again does nothing more. Systems may call it."
  guide "loading §12"
  aliases "preload level, load the next scene in the background, async scene load, Unity LoadSceneAsync, Godot ResourceLoader.load_threaded_request"
  param world World "The world."
  param scene String "A GameDesc.scenes name or the scene's asset id."
  returns Result.Asset "The scene's handle, or why not: no such scene (the reason lists GameDesc.scenes), or not cooked."
}
```

<sub>Module `scene` line 51</sub>
