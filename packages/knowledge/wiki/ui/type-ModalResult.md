<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ModalResult"></a>

### `ModalResult` (struct)

A dialog's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean` | `false` | — | — | — | Build its content (open, or leaving). |
| `dismissed` | `boolean` | `false` | — | — | — | Back or a press on the backdrop asked to close it: set your open flag false. |

```ogt
struct ModalResult {
  doc "A dialog's answer this tick."
  field visible bool "Build its content (open, or leaving)."
  field dismissed bool "Back or a press on the backdrop asked to close it: set your open flag false."
}
```

<sub>Module `ui` line 974</sub>
