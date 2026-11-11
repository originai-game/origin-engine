<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-CaptureState"></a>

### `CaptureState` (struct)

Where a capture is: waiting, done with the control it took, or cancelled.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `active` | `boolean` | `false` | — | — | — | It waits for input. |
| `done` | `boolean` | `false` | — | — | — | It took a control: apply it with og.input.applyCapture (or show the conflicts first). |
| `cancelled` | `boolean` | `false` | — | — | — | The player cancelled it, or it timed out. |
| `action` | `string` | `""` | — | — | — | The action. |
| `slot` | `number` | `0` | — | — | — | The slot. |
| `part` | `string` | `""` | — | — | — | The part. |
| `control` | `string` | `""` | — | — | — | The control it took. |
| `binding` | `InputBinding` | `{}` | — | — | — | The slot's binding with that control in it. |
| `conflicts` | `readonly BindingConflict[]` | `[]` | — | — | — | What applying it would share a control with. |

```ogt
struct CaptureState {
  doc "Where a capture is: waiting, done with the control it took, or cancelled."
  field active bool "It waits for input."
  field done bool "It took a control: apply it with og.input.applyCapture (or show the conflicts first)."
  field cancelled bool "The player cancelled it, or it timed out."
  field action String "The action."
  field slot u32 "The slot."
  field part String "The part."
  field control String "The control it took."
  field binding InputBinding "The slot's binding with that control in it."
  field conflicts List.BindingConflict "What applying it would share a control with."
}
```

<sub>Module `input` line 225</sub>
