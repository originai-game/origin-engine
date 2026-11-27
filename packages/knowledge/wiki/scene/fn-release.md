<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="fn-release"></a>

### `og.scene.release`

Stop asking for the streaming groups og.scene.preload asked for (each unloads unless its trigger or og.stream.request still wants it). The current scene's groups are kept while it is in place.

```ts
og.scene.release(world: World, scene: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `scene` | `string` | no | A GameDesc.scenes name or the scene's asset id. |

**Returns** `Result<void>`: Refused for an unknown scene.

Binding symbol `og_scene_release` · publish builds: kept

```ogt
fn release {
  doc "Stop asking for the streaming groups og.scene.preload asked for (each unloads unless its trigger or og.stream.request still wants it). The current scene's groups are kept while it is in place."
  param world World "The world."
  param scene String "A GameDesc.scenes name or the scene's asset id."
  returns Result.Void "Refused for an unknown scene."
}
```

<sub>Module `scene` line 65</sub>
