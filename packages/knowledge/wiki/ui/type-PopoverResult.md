<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PopoverResult"></a>

### `PopoverResult` (struct)

What a popover answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dismissed` | `boolean` | `false` | — | — | — | Back or a press outside asked to close it. |
| `visible` | `boolean` | `false` | — | — | — | It is on screen (open, or leaving). |

```ogt
struct PopoverResult {
  doc "What a popover answered this tick."
  field dismissed bool "Back or a press outside asked to close it."
  field visible bool "It is on screen (open, or leaving)."
}
```

<sub>Module `ui` line 2063</sub>
