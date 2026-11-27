<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="fn-progress"></a>

### `og.scene.progress`

How far the scene's streaming groups have arrived and whether it is prepared: answered at once from what the host has received, for a loading screen's bar. Natively a group is read at the tick boundary it is admitted, so it goes from 0 to 1.

```ts
og.scene.progress(world: World, scene: string): Result<ScenePreload>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `scene` | `string` | no | A GameDesc.scenes name or the scene's asset id. |

**Returns** `Result<ScenePreload>`: Refused for a scene that is not a GameDesc.scenes name or a cooked scene.

Binding symbol `og_scene_progress` · publish builds: kept

```ogt
fn progress {
  doc "How far the scene's streaming groups have arrived and whether it is prepared: answered at once from what the host has received, for a loading screen's bar. Natively a group is read at the tick boundary it is admitted, so it goes from 0 to 1."
  param world World "The world."
  param scene String "A GameDesc.scenes name or the scene's asset id."
  returns Result.ScenePreload "Refused for a scene that is not a GameDesc.scenes name or a cooked scene."
}
```

<sub>Module `scene` line 58</sub>
