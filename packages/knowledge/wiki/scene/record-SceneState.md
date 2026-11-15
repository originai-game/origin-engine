<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="record-SceneState"></a>

### `og.SceneState` (resource)

The scene og.scene.change put in place and the one it waits to put in place: world state, saved, restored and hashed. Registered at the first og.scene.change; a world that never changes scene keeps its type table, saves and hash.

Token `og.SceneState`, registered under the schema name `og.SceneState`; its value type is `SceneState`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `current` | `string` | `""` | — | — | `str(256)`: at most 256 bytes of UTF-8, read only | The asset id of the scene in place (empty: the boot scene, GameDesc.scene, which spawned at the top level). |
| `pending` | `string` | `""` | — | — | `str(256)`: at most 256 bytes of UTF-8, read only | The asset id of the scene og.scene.change asked for, put in place as the next tick starts (empty: none waits). |
| `root` | `Entity` | `0` | — | — | read only | The root the current scene's entities are under (0: the boot scene's, at the top level). |
| `changes` | `number` | `0` | — | — | read only | How many changes have been put in place. |

```ogt
resource SceneState {
  doc "The scene og.scene.change put in place and the one it waits to put in place: world state, saved, restored and hashed. Registered at the first og.scene.change; a world that never changes scene keeps its type table, saves and hash."
  field current str.256 "The asset id of the scene in place (empty: the boot scene, GameDesc.scene, which spawned at the top level)." {
    readonly true
  }
  field pending str.256 "The asset id of the scene og.scene.change asked for, put in place as the next tick starts (empty: none waits)." {
    readonly true
  }
  field root Entity "The root the current scene's entities are under (0: the boot scene's, at the top level)." {
    readonly true
  }
  field changes u32 "How many changes have been put in place." {
    readonly true
  }
}
```

<sub>Module `scene` line 7</sub>
