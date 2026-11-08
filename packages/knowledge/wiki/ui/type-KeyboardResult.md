<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyboardResult"></a>

### `KeyboardResult` (struct)

What an on-screen keyboard did this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `string` | `""` | — | — | — | The text, to keep and pass back. |
| `composition` | `string` | `""` | — | — | — | What is being composed and not yet committed (show it, and look up candidates for it). |
| `changed` | `boolean` | `false` | — | — | — | The text changed this tick. |
| `submitted` | `boolean` | `false` | — | — | — | Done was pressed: take the text. |
| `closed` | `boolean` | `false` | — | — | — | It was closed without submitting (Back, the close key, a press outside). |
| `layout` | `KeyboardLayout` | `"letters"` | — | — | — | The keys shown now. |

```ogt
struct KeyboardResult {
  doc "What an on-screen keyboard did this tick."
  field value String "The text, to keep and pass back."
  field composition String "What is being composed and not yet committed (show it, and look up candidates for it)."
  field changed bool "The text changed this tick."
  field submitted bool "Done was pressed: take the text."
  field closed bool "It was closed without submitting (Back, the close key, a press outside)."
  field layout KeyboardLayout "The keys shown now."
}
```

<sub>Module `ui` line 4163</sub>
