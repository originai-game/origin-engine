<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MenuEntry"></a>

### `MenuEntry` (desc struct)

An entry of a menu. An entry whose `parent` names another's id sits in that entry's submenu.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The id answered when it is chosen; also its probe id. |
| `label` | `string` | `""` | — | — | — | What it says. |
| `kind` | `MenuEntryKind` | `"item"` | — | — | — | What it is. |
| `icon` | `string` | `""` | — | — | — | An icon before the label; empty is none. |
| `shortcut` | `string` | `""` | — | — | — | A shortcut shown at the end (`Ctrl+S`). |
| `checked` | `boolean` | `false` | — | — | — | A check or radio entry is on. |
| `danger` | `boolean` | `false` | — | — | — | A destructive command: the danger ink. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `parent` | `string` | `""` | — | — | — | The id of the entry whose submenu it is in; empty is the top level. |

```ogt
struct MenuEntry {
  doc "An entry of a menu. An entry whose `parent` names another's id sits in that entry's submenu."
  desc true
  field id String "The id answered when it is chosen; also its probe id."
  field label String "What it says."
  field kind MenuEntryKind "What it is."
  field icon String "An icon before the label; empty is none."
  field shortcut String "A shortcut shown at the end (`Ctrl+S`)."
  field checked bool "A check or radio entry is on."
  field danger bool "A destructive command: the danger ink."
  field disabled bool "Shown but inert."
  field parent String "The id of the entry whose submenu it is in; empty is the top level."
}
```

<sub>`api.ogt` line 17942</sub>
