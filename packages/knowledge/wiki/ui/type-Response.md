<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Response"></a>

### `Response` (struct)

What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hovered` | `boolean` | `false` | — | — | — | The pointer is over it and no other element holds the press. |
| `pressed` | `boolean` | `false` | — | — | — | The press that started on it is still held. |
| `clicked` | `boolean` | `false` | — | — | — | Pressed and released inside it. |
| `changed` | `boolean` | `false` | — | — | — | Its value changed (slider, toggle, text field). |
| `focused` | `boolean` | `false` | — | — | — | It has focus: a text field being edited, or the control keyboard and gamepad navigation landed on. |
| `submitted` | `boolean` | `false` | — | — | — | Enter was pressed in it (text field). |
| `dropped` | `number \| null` | `null` | — | — | — | A list row or grid cell a press started on was released over this one: that row's or cell's index (drag and drop). |
| `pad` | `number \| null` | `null` | — | — | — | The gamepad (0 to 3) whose A pressed it this tick; null when the pointer or the keyboard did. Pads 1 to 3 press only while og.ui.setPlayers allows them (local multiplayer). |
| `focusedBy` | `number` | `0` | — | — | — | Bit k: pad k's focus is on it (bit 0 is the shared focus of the pointer, the keyboard and pad 0; bits 1 to 3 the other players' pads). |
| `adjust` | `number` | `0` | — | — | — | An adjustable element's (ElementDesc.adjustable) Left and Right this tick while it has the focus: +1 for each press of Right, -1 for Left (the screen's directions; a right-to-left screen reads Left as forward); 0 otherwise. |
| `adjustY` | `number` | `0` | — | — | — | An element claiming Up and Down (ElementDesc.arrows) this tick while it has the focus: +1 for each press of Down, -1 for Up; 0 otherwise. |
| `ring` | `boolean` | `false` | — | — | — | The focus ring is up on it: the focus is on it and keys or a pad put it there (a press of the pointer, or a focus returned by a closing dialog to a pointer's player, puts it away). With ElementDesc.ownRing the component draws its own ring and what keys show (a cursor's card lifted). |
| `tapped` | `boolean` | `false` | — | — | — | Its tap was recognised this tick (ElementProps.gestures): a press and a release inside it within the slop and, where it declares a double tap, once that double tap's window has passed without a second tap. Keys and pads tap it by activating it. |
| `doubleTapped` | `boolean` | `false` | — | — | — | The second tap of its double tap landed this tick. |
| `longPressed` | `boolean` | `false` | — | — | — | Its long press fired this tick; the press it took clicks and taps nothing when released. |
| `longPress` | `number` | `0` | — | `ratio` | — | How far into its long press the held press is, 0 to 1 (0: none): a ring that fills while a touch is held. |

```ogt
struct Response {
  doc "What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick."
  field hovered bool "The pointer is over it and no other element holds the press."
  field pressed bool "The press that started on it is still held."
  field clicked bool "Pressed and released inside it."
  field changed bool "Its value changed (slider, toggle, text field)."
  field focused bool "It has focus: a text field being edited, or the control keyboard and gamepad navigation landed on."
  field submitted bool "Enter was pressed in it (text field)."
  field dropped Option.u32 "A list row or grid cell a press started on was released over this one: that row's or cell's index (drag and drop)."
  field pad Option.u32 "The gamepad (0 to 3) whose A pressed it this tick; null when the pointer or the keyboard did. Pads 1 to 3 press only while og.ui.setPlayers allows them (local multiplayer)."
  field focusedBy u32 "Bit k: pad k's focus is on it (bit 0 is the shared focus of the pointer, the keyboard and pad 0; bits 1 to 3 the other players' pads)."
  field adjust i32 "An adjustable element's (ElementDesc.adjustable) Left and Right this tick while it has the focus: +1 for each press of Right, -1 for Left (the screen's directions; a right-to-left screen reads Left as forward); 0 otherwise."
  field adjustY i32 "An element claiming Up and Down (ElementDesc.arrows) this tick while it has the focus: +1 for each press of Down, -1 for Up; 0 otherwise."
  field ring bool "The focus ring is up on it: the focus is on it and keys or a pad put it there (a press of the pointer, or a focus returned by a closing dialog to a pointer's player, puts it away). With ElementDesc.ownRing the component draws its own ring and what keys show (a cursor's card lifted)."
  field tapped bool "Its tap was recognised this tick (ElementProps.gestures): a press and a release inside it within the slop and, where it declares a double tap, once that double tap's window has passed without a second tap. Keys and pads tap it by activating it."
  field doubleTapped bool "The second tap of its double tap landed this tick."
  field longPressed bool "Its long press fired this tick; the press it took clicks and taps nothing when released."
  field longPress f32 "How far into its long press the held press is, 0 to 1 (0: none): a ring that fills while a touch is held." {
    unit ratio
  }
}
```

<sub>Module `ui` line 388</sub>
