<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SavedScene"></a>

### `SavedScene` (struct)

The scenes written back from the world: the head is the boot scene's (or the first instantiated one's), `scenes` lists every one.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick of the world it was written from. |
| `source` | `string` | `""` | — | — | — | The first scene's source, project-relative. |
| `written` | `string` | `""` | — | — | — | Where it was written; empty when only returned. |
| `lines` | `readonly number[]` | `[]` | — | — | — | The 1-based lines of the first scene's new text that changed or were added. |
| `skipped` | `readonly string[]` | `[]` | — | — | — | Tuned fields that were not written, each with its reason: an entity no scene made (without `spawned`), a resource no scene sets where no boot scene takes it. |
| `text` | `string` | `""` | — | — | — | The first scene's new text. |
| `spawned` | `number` | `0` | — | — | — | Entities written under the boot scene's spawned section (0 without `spawned`). |
| `scenes` | `readonly SavedSceneFile[]` | `[]` | — | — | — | Every scene written back: the boot scene's first, then each og.world.instantiate made, once each. |

```ogt
struct SavedScene {
  doc "The scenes written back from the world: the head is the boot scene's (or the first instantiated one's), `scenes` lists every one."
  field tick u32 "The tick of the world it was written from."
  field source String "The first scene's source, project-relative."
  field written String "Where it was written; empty when only returned."
  field lines List.u32 "The 1-based lines of the first scene's new text that changed or were added."
  field skipped List.String "Tuned fields that were not written, each with its reason: an entity no scene made (without `spawned`), a resource no scene sets where no boot scene takes it."
  field text String "The first scene's new text."
  field spawned u32 "Entities written under the boot scene's spawned section (0 without `spawned`)."
  field scenes List.SavedSceneFile "Every scene written back: the boot scene's first, then each og.world.instantiate made, once each."
}
```

<sub>`api.ogt` line 24477</sub>
