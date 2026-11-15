<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="type-LoadingStageDesc"></a>

### `LoadingStageDesc` (desc struct)

A stage of the game's own loading (og.loading.stage).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The stage's id, such as `assets` or `level`; `done` is og.loading.done. |
| `fraction` | `number` | `0` | 0 … 1 | — | — | How far the stage is, 0 to 1. |
| `label` | `string` | `""` | — | — | — | Text for a loader to show, in the player's language (empty: the page's own words). |

```ogt
struct LoadingStageDesc {
  doc "A stage of the game's own loading (og.loading.stage)."
  desc true
  field id String "The stage's id, such as `assets` or `level`; `done` is og.loading.done."
  field fraction f32 "How far the stage is, 0 to 1." {
    min 0
    max 1
  }
  field label String "Text for a loader to show, in the player's language (empty: the page's own words)."
}
```

<sub>Module `loading` line 13</sub>
