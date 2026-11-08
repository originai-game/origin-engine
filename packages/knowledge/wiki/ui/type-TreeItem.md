<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TreeItem"></a>

### `TreeItem` (desc struct)

An item of a tree; an item whose `parent` names another's id sits under it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id: answered when it is chosen, and its children's parent. |
| `parent` | `string` | `""` | — | — | — | Its parent's id; empty is the top level. |
| `label` | `string` | `""` | — | — | — | What it says. |
| `icon` | `string` | `""` | — | — | — | An icon before it; empty is none. |
| `badge` | `string` | `""` | — | — | — | A badge after it; empty is none. |
| `badgeTone` | `Tone` | `"neutral"` | — | — | — | The badge's tone. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but not choosable. |

```ogt
struct TreeItem {
  doc "An item of a tree; an item whose `parent` names another's id sits under it."
  desc true
  field id String "Its id: answered when it is chosen, and its children's parent."
  field parent String "Its parent's id; empty is the top level."
  field label String "What it says."
  field icon String "An icon before it; empty is none."
  field badge String "A badge after it; empty is none."
  field badgeTone Tone "The badge's tone."
  field disabled bool "Shown but not choosable."
}
```

<sub>Module `ui` line 2492</sub>
