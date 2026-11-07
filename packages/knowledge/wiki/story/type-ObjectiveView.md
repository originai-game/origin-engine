<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-ObjectiveView"></a>

### `ObjectiveView` (struct)

An objective of a quest's current stage.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `text` | `string` | `""` | — | — | — | Its text, resolved in the active locale with `count` and `target` among its arguments. |
| `count` | `number` | `0` | — | — | — | Counted so far. |
| `target` | `number` | `0` | — | — | — | Counts needed. |
| `done` | `boolean` | `false` | — | — | — | count reached target. |
| `failed` | `boolean` | `false` | — | — | — | Left undone when the quest failed. |
| `optional` | `boolean` | `false` | — | — | — | Not needed to finish the stage. |
| `hidden` | `boolean` | `false` | — | — | — | Not revealed yet: declared hidden and not yet counted (the log leaves it out or shows ???). |

```ogt
struct ObjectiveView {
  doc "An objective of a quest's current stage."
  field id String "Its id."
  field text String "Its text, resolved in the active locale with `count` and `target` among its arguments."
  field count u32 "Counted so far."
  field target u32 "Counts needed."
  field done bool "count reached target."
  field failed bool "Left undone when the quest failed."
  field optional bool "Not needed to finish the stage."
  field hidden bool "Not revealed yet: declared hidden and not yet counted (the log leaves it out or shows ???)."
}
```

<sub>Module `story` line 249</sub>
