<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-GamepadInput"></a>

### `GamepadInput` (struct)

A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `0` | ≤ 7 | — | optional | Which pad, 0 to 7; omitted is 0. |
| `connected` | `boolean` | `false` | — | — | optional | Whether the pad is connected (omitted is true); a disconnected pad holds nothing. |
| `buttons` | `readonly GamepadButton[]` | `[]` | — | — | optional | The buttons held (omitted is none). |
| `axes` | `readonly number[]` | `[]` | — | — | optional | Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0. |
| `triggers` | `readonly number[]` | `[]` | — | — | optional | The left and right analog triggers, 0 to 1. Missing values are 0. |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | optional | The pad's family as the host tells it from the pad's id (xbox, playstation, switch, or gamepad for a pad it cannot place); omitted keeps what was said. Glyphs follow it. |

```ogt
struct GamepadInput {
  doc "A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own."
  field pad u32 "Which pad, 0 to 7; omitted is 0." {
    max 7
    optional true
  }
  field connected bool "Whether the pad is connected (omitted is true); a disconnected pad holds nothing." {
    optional true
  }
  field buttons List.GamepadButton "The buttons held (omitted is none)." {
    optional true
  }
  field axes List.f32 "Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0." {
    optional true
  }
  field triggers List.f32 "The left and right analog triggers, 0 to 1. Missing values are 0." {
    optional true
  }
  field device InputDevice "The pad's family as the host tells it from the pad's id (xbox, playstation, switch, or gamepad for a pad it cannot place); omitted keeps what was said. Glyphs follow it." {
    optional true
  }
}
```

<sub>Module `input` line 690</sub>
