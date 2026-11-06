<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-PointerInput"></a>

### `PointerInput` (struct)

A pointer event at a point in CSS pixels of the play surface.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `PointerAction` | `"click"` | — | — | — | Click, move, down or up. |
| `x` | `number` | `0` | — | `cssPx` | — | Horizontal position. |
| `y` | `number` | `0` | — | `cssPx` | — | Vertical position. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button; `none` for a hover move. |
| `id` | `number` | `0` | ≤ 4 | — | optional | Which pointer: 0 the primary one (the mouse, the first touch; what protocol 5 means), 1 to 4 further touches held at the same time (an engine extension). Omitted is 0. |
| `at` | `number` | `0` | -1 … 1 | — | optional | When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5. |
| `movementX` | `number` | `0` | — | `cssPx` | optional | The mouse's relative movement since its last event (the browser's movementX), which keeps coming while the pointer is locked and x, y stand still: og.Pointers.movement sums pointer 0's over the tick for mouse-look. Omitted is 0. An engine extension of protocol 5. |
| `movementY` | `number` | `0` | — | `cssPx` | optional | The relative vertical movement (movementY, down positive). Omitted is 0. |

```ogt
struct PointerInput {
  doc "A pointer event at a point in CSS pixels of the play surface."
  field action PointerAction "Click, move, down or up."
  field x f32 "Horizontal position." {
    unit cssPx
  }
  field y f32 "Vertical position." {
    unit cssPx
  }
  field button PointerButton "The button; `none` for a hover move."
  field id u32 "Which pointer: 0 the primary one (the mouse, the first touch; what protocol 5 means), 1 to 4 further touches held at the same time (an engine extension). Omitted is 0." {
    max 4
    optional true
  }
  field at f32 "When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5." {
    min -1
    max 1
    optional true
  }
  field movementX f32 "The mouse's relative movement since its last event (the browser's movementX), which keeps coming while the pointer is locked and x, y stand still: og.Pointers.movement sums pointer 0's over the tick for mouse-look. Omitted is 0. An engine extension of protocol 5." {
    unit cssPx
    optional true
  }
  field movementY f32 "The relative vertical movement (movementY, down positive). Omitted is 0." {
    unit cssPx
    optional true
  }
}
```

<sub>`api.ogt` line 2625</sub>
