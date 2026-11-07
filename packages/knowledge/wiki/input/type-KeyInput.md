<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-KeyInput"></a>

### `KeyInput` (struct)

A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `action` | `KeyAction` | `"press"` | — | — | — | Press, down or up. |
| `modifiers` | `Modifiers` | `{ alt: false, control: false, platform: false, shift: false }` | — | — | optional | Held modifiers; omitted means none. |
| `at` | `number` | `0` | -1 … 1 | — | optional | When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5. |

```ogt
struct KeyInput {
  doc "A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA."
  field key String "The physical key code."
  field action KeyAction "Press, down or up."
  field modifiers Modifiers "Held modifiers; omitted means none." {
    optional true
  }
  field at f32 "When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5." {
    min -1
    max 1
    optional true
  }
}
```

<sub>Module `input` line 636</sub>
