<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-ActionState"></a>

### `ActionState` (struct)

An action this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `down` | `boolean` | `false` | — | — | — | Its binding is past the threshold (or its toggle is on). |
| `justPressed` | `boolean` | `false` | — | — | — | It went down this tick (a press and a release between two ticks set both edges). |
| `justReleased` | `boolean` | `false` | — | — | — | It went up this tick. |
| `held` | `boolean` | `false` | — | — | — | Down for its holdMs. |
| `value` | `number` | `0` | — | — | — | A button's strength (0 to 1: a trigger's travel), an axis's value, a two-axis value's length. |
| `vector` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | A two-axis action's value (x right, y up); value and 0 for the others. |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | — | The device whose binding drove it last. |

```ogt
struct ActionState {
  doc "An action this tick."
  field down bool "Its binding is past the threshold (or its toggle is on)."
  field justPressed bool "It went down this tick (a press and a release between two ticks set both edges)."
  field justReleased bool "It went up this tick."
  field held bool "Down for its holdMs."
  field value f32 "A button's strength (0 to 1: a trigger's travel), an axis's value, a two-axis value's length."
  field vector Vector2 "A two-axis action's value (x right, y up); value and 0 for the others."
  field device InputDevice "The device whose binding drove it last."
}
```

<sub>Module `input` line 148</sub>
