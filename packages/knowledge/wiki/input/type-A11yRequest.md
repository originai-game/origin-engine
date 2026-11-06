<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-A11yRequest"></a>

### `A11yRequest` (struct)

A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. Raises an A11yAction event; the UI acts on it at the start of the next frame, against the last frame's controls (a path naming no focusable control of the topmost dialog is ignored; increment and decrement also take a stepper's spinbutton, whose buttons are the focusable controls).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `A11yActionKind` | `"focus"` | — | — | — | What is asked. |
| `path` | `string` | `""` | — | — | — | The control's path, as og.ui.a11y() names it. |

```ogt
struct A11yRequest {
  doc "A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. Raises an A11yAction event; the UI acts on it at the start of the next frame, against the last frame's controls (a path naming no focusable control of the topmost dialog is ignored; increment and decrement also take a stepper's spinbutton, whose buttons are the focusable controls)."
  field action A11yActionKind "What is asked."
  field path String "The control's path, as og.ui.a11y() names it."
}
```

<sub>`api.ogt` line 2700</sub>
