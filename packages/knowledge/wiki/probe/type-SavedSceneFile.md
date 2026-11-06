<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SavedSceneFile"></a>

### `SavedSceneFile` (struct)

One scene file written back.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `source` | `string` | `""` | — | — | — | Its source, project-relative. |
| `written` | `string` | `""` | — | — | — | Where it was written; empty when only returned. |
| `lines` | `readonly number[]` | `[]` | — | — | — | The 1-based lines of its new text that changed or were added. |
| `text` | `string` | `""` | — | — | — | Its new text. |

```ogt
struct SavedSceneFile {
  doc "One scene file written back."
  field source String "Its source, project-relative."
  field written String "Where it was written; empty when only returned."
  field lines List.u32 "The 1-based lines of its new text that changed or were added."
  field text String "Its new text."
}
```

<sub>`api.ogt` line 24591</sub>
