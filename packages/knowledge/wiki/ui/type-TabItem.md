<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TabItem"></a>

### `TabItem` (desc struct)

A tab.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | What it says. |
| `icon` | `string` | `""` | — | — | — | An icon before it; empty is none. |
| `badge` | `string` | `""` | — | — | — | A count badge after it; empty is none. |
| `badgeTone` | `Tone` | `"primary"` | — | — | — | The badge's tone. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but not selectable. |

```ogt
struct TabItem {
  doc "A tab."
  desc true
  field label String "What it says."
  field icon String "An icon before it; empty is none."
  field badge String "A count badge after it; empty is none."
  field badgeTone Tone "The badge's tone." {
    default primary
  }
  field disabled bool "Shown but not selectable."
}
```

<sub>Module `ui` line 2038</sub>
