<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiMotionEvent"></a>

### `UiMotionEvent` (struct)

What happened in the UI's motion this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | sound, haptic, burst, state, callback, end (a play finished), dismiss (a swipe threw an element off), drop (a drag was dropped on a target), snap (a released drag chose a snap point), or tap, doubleTap and longPress (an element's ElementProps.gestures recognised one). |
| `name` | `string` | `""` | — | — | — | The sound, haptic, callback or state's name; for drop the target's path; for snap the point's index. |
| `target` | `string` | `""` | — | — | — | The element's path. |
| `play` | `number` | `0` | — | — | — | The play (0: none). |
| `index` | `number` | `0` | — | — | — | The clip's step in a cascade or stagger. |
| `playbackRate` | `number` | `0` | — | — | — | A sound's pitch: its own times 1.06 per cascade step. |

```ogt
struct UiMotionEvent {
  doc "What happened in the UI's motion this tick."
  field kind String "sound, haptic, burst, state, callback, end (a play finished), dismiss (a swipe threw an element off), drop (a drag was dropped on a target), snap (a released drag chose a snap point), or tap, doubleTap and longPress (an element's ElementProps.gestures recognised one)."
  field name String "The sound, haptic, callback or state's name; for drop the target's path; for snap the point's index."
  field target String "The element's path."
  field play u32 "The play (0: none)."
  field index u32 "The clip's step in a cascade or stagger."
  field playbackRate f32 "A sound's pitch: its own times 1.06 per cascade step."
}
```

<sub>Module `ui` line 2942</sub>
