<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GestureInfo"></a>

### `GestureInfo` (struct)

What a drag or pinch on an element is doing (last tick's answer, as responses are).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `held` | `boolean` | `false` | — | — | — | A press is held on it. |
| `dragging` | `boolean` | `false` | — | — | — | It moves with the pointer (the press travelled past the threshold). |
| `offsetX` | `number` | `0` | — | `cssPx` | — | Its offset from where it was laid out. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | Its offset down. |
| `velocityX` | `number` | `0` | — | — | — | CSS px per second. |
| `velocityY` | `number` | `0` | — | — | — | CSS px per second. |
| `scale` | `number` | `0` | — | — | — | A pinch's scale (1: none). |
| `rotation` | `number` | `0` | — | — | — | A pinch's turn, radians. |
| `snapped` | `number` | `0` | — | — | — | The snap point the last release chose (-1: none). |
| `dismissed` | `boolean` | `false` | — | — | — | A swipe threw it off. |
| `over` | `string` | `""` | — | — | — | The drop target under the drag (empty: none). |
| `droppedOn` | `string` | `""` | — | — | — | The drop target it was dropped on this tick (empty: none). |
| `hovering` | `boolean` | `false` | — | — | — | For a drop target: a drag is over it. |
| `received` | `string` | `""` | — | — | — | For a drop target: the element dropped on it this tick (empty: none). |
| `tapped` | `boolean` | `false` | — | — | — | Its tap this tick (ElementProps.gestures), as Response.tapped. |
| `doubleTapped` | `boolean` | `false` | — | — | — | Its double tap this tick. |
| `longPressed` | `boolean` | `false` | — | — | — | Its long press this tick. |
| `longPress` | `number` | `0` | — | `ratio` | — | How far into its long press the held press is, 0 to 1. |

```ogt
struct GestureInfo {
  doc "What a drag or pinch on an element is doing (last tick's answer, as responses are)."
  field held bool "A press is held on it."
  field dragging bool "It moves with the pointer (the press travelled past the threshold)."
  field offsetX f32 "Its offset from where it was laid out." {
    unit cssPx
  }
  field offsetY f32 "Its offset down." {
    unit cssPx
  }
  field velocityX f32 "CSS px per second."
  field velocityY f32 "CSS px per second."
  field scale f32 "A pinch's scale (1: none)."
  field rotation f32 "A pinch's turn, radians."
  field snapped i32 "The snap point the last release chose (-1: none)."
  field dismissed bool "A swipe threw it off."
  field over String "The drop target under the drag (empty: none)."
  field droppedOn String "The drop target it was dropped on this tick (empty: none)."
  field hovering bool "For a drop target: a drag is over it."
  field received String "For a drop target: the element dropped on it this tick (empty: none)."
  field tapped bool "Its tap this tick (ElementProps.gestures), as Response.tapped."
  field doubleTapped bool "Its double tap this tick."
  field longPressed bool "Its long press this tick."
  field longPress f32 "How far into its long press the held press is, 0 to 1." {
    unit ratio
  }
}
```

<sub>Module `ui` line 2897</sub>
