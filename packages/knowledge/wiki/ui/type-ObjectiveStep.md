<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ObjectiveStep"></a>

### `ObjectiveStep` (desc struct)

A step of an objective.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key. |
| `text` | `string` | `""` | — | — | — | What to do. |
| `done` | `boolean` | `false` | — | — | — | Done: ticked and dimmed. |
| `failed` | `boolean` | `false` | — | — | — | Failed: crossed in danger. |
| `count` | `number` | `0` | — | — | — | How many so far (3 of 5). |
| `target` | `number` | `0` | — | — | — | How many it needs; 0 shows no count. |
| `progress` | `number \| null` | `null` | — | `ratio` | — | A bar under it, 0 to 1. |
| `optional` | `boolean` | `false` | — | — | — | Optional: in the muted ink with (Optional). |

```ogt
struct ObjectiveStep {
  doc "A step of an objective."
  desc true
  field key String "Its key."
  field text String "What to do."
  field done bool "Done: ticked and dimmed."
  field failed bool "Failed: crossed in danger."
  field count u32 "How many so far (3 of 5)."
  field target u32 "How many it needs; 0 shows no count."
  field progress Option.f32 "A bar under it, 0 to 1." {
    unit ratio
  }
  field optional bool "Optional: in the muted ink with (Optional)."
}
```

<sub>`api.ogt` line 22071</sub>
