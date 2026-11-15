<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="record-SceneChanged"></a>

### `og.SceneChanged` (event)

A scene change was put in place: sent as the tick starts, before any system runs, with the previous scene's entities despawned and the new one's spawned.

Token `og.SceneChanged`, registered under the schema name `og.SceneChanged`; its value type is `SceneChanged`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `Asset` | `0` | — | — | — | The scene now in place. |
| `previous` | `Asset` | `0` | — | — | — | The scene it replaced (0: the boot scene). |
| `root` | `Entity` | `0` | — | — | — | The root the new scene's entities are under. |
| `entities` | `number` | `0` | — | — | — | How many entities it spawned. |

```ogt
event SceneChanged {
  doc "A scene change was put in place: sent as the tick starts, before any system runs, with the previous scene's entities despawned and the new one's spawned."
  field scene Asset "The scene now in place."
  field previous Asset "The scene it replaced (0: the boot scene)."
  field root Entity "The root the new scene's entities are under."
  field entities u32 "How many entities it spawned."
}
```

<sub>Module `scene` line 23</sub>
