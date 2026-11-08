<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DialogResult"></a>

### `DialogResult` (struct)

What a dialog answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string \| null` | `null` | — | — | — | The id of the action pressed. |
| `dismissed` | `boolean` | `false` | — | — | — | Back or the backdrop asked to close it. |
| `visible` | `boolean` | `false` | — | — | — | It is on screen (open, or leaving). |

```ogt
struct DialogResult {
  doc "What a dialog answered this tick."
  field action Option.String "The id of the action pressed."
  field dismissed bool "Back or the backdrop asked to close it."
  field visible bool "It is on screen (open, or leaving)."
}
```

<sub>Module `ui` line 2105</sub>
