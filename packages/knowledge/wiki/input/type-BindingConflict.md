<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-BindingConflict"></a>

### `BindingConflict` (struct)

Another binding a rebinding would share a control with: an action of the same context (or another slot of the same action).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The other action. |
| `slot` | `number` | `0` | — | — | — | Its binding's slot. |
| `control` | `string` | `""` | — | — | — | The shared control, as the new binding names it. |
| `context` | `string` | `""` | — | — | — | The context they share. |

```ogt
struct BindingConflict {
  doc "Another binding a rebinding would share a control with: an action of the same context (or another slot of the same action)."
  field action String "The other action."
  field slot u32 "Its binding's slot."
  field control String "The shared control, as the new binding names it."
  field context String "The context they share."
}
```

<sub>Module `input` line 131</sub>
