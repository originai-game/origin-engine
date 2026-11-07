<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HudEditorResult"></a>

### `HudEditorResult` (struct)

What the HUD editor's buttons did this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `done` | `boolean` | `false` | — | — | — | Done was pressed: the edit mode is off from the next tick. |
| `changed` | `boolean` | `false` | — | — | — | A step or a reset changed the layout this tick. |

```ogt
struct HudEditorResult {
  doc "What the HUD editor's buttons did this tick."
  field done bool "Done was pressed: the edit mode is off from the next tick."
  field changed bool "A step or a reset changed the layout this tick."
}
```

<sub>Module `ui` line 1707</sub>
