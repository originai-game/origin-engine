<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiSounds"></a>

### `UiSounds` (desc struct)

Sounds the UI plays through og-audio (the ui bus) on its events; null is silent.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hover` | `Sound \| null` | `null` | — | — | — | The pointer reaches a control, or keyboard or gamepad focus moves to one. |
| `press` | `Sound \| null` | `null` | — | — | — | A control is pressed: by the pointer, Enter, Space or the gamepad's A. |
| `open` | `Sound \| null` | `null` | — | — | — | A dialog opens. |
| `close` | `Sound \| null` | `null` | — | — | — | A dialog closes. |
| `success` | `Sound \| null` | `null` | — | — | — | A success toast. |
| `error` | `Sound \| null` | `null` | — | — | — | A refused press or a danger toast. |

```ogt
struct UiSounds {
  doc "Sounds the UI plays through og-audio (the ui bus) on its events; null is silent."
  desc true
  field hover Option.Sound "The pointer reaches a control, or keyboard or gamepad focus moves to one."
  field press Option.Sound "A control is pressed: by the pointer, Enter, Space or the gamepad's A."
  field open Option.Sound "A dialog opens."
  field close Option.Sound "A dialog closes."
  field success Option.Sound "A success toast."
  field error Option.Sound "A refused press or a danger toast."
}
```

<sub>Module `ui` line 486</sub>
