<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ConfirmResult"></a>

### `ConfirmResult` (struct)

A confirm dialog's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `confirmed` | `boolean` | `false` | — | — | — | The action was chosen. |
| `cancelled` | `boolean` | `false` | — | — | — | Cancel, Back or the backdrop. |

```ogt
struct ConfirmResult {
  doc "A confirm dialog's answer this tick."
  field confirmed bool "The action was chosen."
  field cancelled bool "Cancel, Back or the backdrop."
}
```

<sub>Module `ui` line 2009</sub>
