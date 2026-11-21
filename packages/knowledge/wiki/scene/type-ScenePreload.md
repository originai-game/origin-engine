<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="type-ScenePreload"></a>

### `ScenePreload` (struct)

How far a scene's streaming groups have arrived and whether it is prepared (og.scene.progress). Presentation, for a loading screen: never decide a tick's outcome on it in a networked game.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `string` | `""` | — | — | — | The scene's asset id. |
| `ready` | `boolean` | `false` | — | — | — | Every group it declares is resident (true for a scene without groups). |
| `prepared` | `boolean` | `false` | — | — | — | Placed now, the scene is drawn whole: every group is resident and, after og.scene.preload, everything it spawns with is loaded (its models and their textures in the renderer, its sub-scenes and prefabs) with no pipeline of it still compiling. False for a scene og.scene.preload was not asked for. |
| `fraction` | `number` | `0` | 0 … 1 | — | — | The bytes received over the bytes of its groups' packs, 0 to 1 (1 without groups). |
| `received` | `number` | `0` | — | `bytes` | — | Bytes of its packs received so far. |
| `bytes` | `number` | `0` | — | `bytes` | — | Bytes of its packs in all. |
| `groups` | `readonly string[]` | `[]` | — | — | — | The streaming groups it declares, by name. |
| `waiting` | `readonly string[]` | `[]` | — | — | — | Those not resident yet. |
| `failed` | `string` | `""` | — | — | — | Why a group failed (its og.stream.status reason, naming the group); empty otherwise. |

```ogt
struct ScenePreload {
  doc "How far a scene's streaming groups have arrived and whether it is prepared (og.scene.progress). Presentation, for a loading screen: never decide a tick's outcome on it in a networked game."
  field scene String "The scene's asset id."
  field ready bool "Every group it declares is resident (true for a scene without groups)."
  field prepared bool "Placed now, the scene is drawn whole: every group is resident and, after og.scene.preload, everything it spawns with is loaded (its models and their textures in the renderer, its sub-scenes and prefabs) with no pipeline of it still compiling. False for a scene og.scene.preload was not asked for."
  field fraction f32 "The bytes received over the bytes of its groups' packs, 0 to 1 (1 without groups)." {
    min 0
    max 1
  }
  field received f64 "Bytes of its packs received so far." {
    unit bytes
  }
  field bytes f64 "Bytes of its packs in all." {
    unit bytes
  }
  field groups List.String "The streaming groups it declares, by name."
  field waiting List.String "Those not resident yet."
  field failed String "Why a group failed (its og.stream.status reason, naming the group); empty otherwise."
}
```

<sub>Module `scene` line 31</sub>
