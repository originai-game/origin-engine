<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GestureSpec"></a>

### `GestureSpec` (desc struct)

Tap, double tap and long press on any element (docs/ui.md §8.5), answered by Response (tapped, doubleTapped, longPressed, longPress), og.ui.gesture and the frame's motion events. Nested elements compete for one press: of those that declare a gesture, the highest priority wins it, the innermost of equals. A tap waits for a double tap only where one is declared; a long press that fires takes the press, so its release clicks nothing; travel past the slop (a drag, a scroll) fails all three. The zero value recognises nothing.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tap` | `boolean` | `false` | — | — | — | Recognise a tap. |
| `doubleTap` | `boolean` | `false` | — | — | — | Recognise a double tap (its tap then waits for the double tap's window). |
| `longPress` | `boolean` | `false` | — | — | — | Recognise a long press (the touch tooltip, a context menu). |
| `longPressMs` | `number` | `0` | ≥ 0 | `ms` | — | How long a long press is held; 0 is 500. |
| `doubleTapMs` | `number` | `0` | ≥ 0 | `ms` | — | The longest gap between a double tap's taps; 0 is 300. |
| `slop` | `number` | `0` | ≥ 0 | `cssPx` | — | Pointer travel that fails a tap and a long press; 0 is 10. |
| `priority` | `number` | `0` | — | — | — | Among nested elements' recognisers of one press: higher wins (ties: the innermost). |

```ogt
struct GestureSpec {
  doc "Tap, double tap and long press on any element (docs/ui.md §8.5), answered by Response (tapped, doubleTapped, longPressed, longPress), og.ui.gesture and the frame's motion events. Nested elements compete for one press: of those that declare a gesture, the highest priority wins it, the innermost of equals. A tap waits for a double tap only where one is declared; a long press that fires takes the press, so its release clicks nothing; travel past the slop (a drag, a scroll) fails all three. The zero value recognises nothing."
  desc true
  field tap bool "Recognise a tap."
  field doubleTap bool "Recognise a double tap (its tap then waits for the double tap's window)."
  field longPress bool "Recognise a long press (the touch tooltip, a context menu)."
  field longPressMs f32 "How long a long press is held; 0 is 500." {
    unit ms
    min 0
  }
  field doubleTapMs f32 "The longest gap between a double tap's taps; 0 is 300." {
    unit ms
    min 0
  }
  field slop f32 "Pointer travel that fails a tap and a long press; 0 is 10." {
    unit cssPx
    min 0
  }
  field priority i32 "Among nested elements' recognisers of one press: higher wins (ties: the innermost)."
}
```

<sub>Module `ui` line 4949</sub>
