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
| `pressedAt` | `number` | `0` | — | `s` | — | When in the tick it went down, in seconds from the tick's start (-dt to dt): the time its key or pointer press carried, so its world time is `time.elapsed + pressedAt` and a replay gives it again. 0 when it did not go down this tick or its press carried no time (a gamepad, an on-screen control). See `input §1`. |
| `buffered` | `boolean` | `false` | — | — | — | A press og.input.consume would take now: within its bufferTicks and not taken yet. |

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
  field pressedAt f32 "When in the tick it went down, in seconds from the tick's start (-dt to dt): the time its key or pointer press carried, so its world time is `time.elapsed + pressedAt` and a replay gives it again. 0 when it did not go down this tick or its press carried no time (a gamepad, an on-screen control)." {
    guide "input §1"
    unit s
  }
  field buffered bool "A press og.input.consume would take now: within its bufferTicks and not taken yet."
}
```

<sub>Module `input` line 123</sub>
