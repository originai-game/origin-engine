<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DragSpec"></a>

### `DragSpec` (desc struct)

The element moves with a drag that starts on it (a card, a sheet, a carousel), released onto a spring that keeps the drag's velocity.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `axis` | `DragAxis` | `"both"` | — | — | — | Which way it moves. |
| `bounds` | `UiRect \| null` | `null` | — | — | — | Bounds of its offset (x to x + width, y to y + height); null is free. |
| `rubber` | `number` | `0` | — | — | — | Past the bounds it follows with resistance, iOS's rubber band with this constant (0 stops at the bounds; 0.55 is iOS's). |
| `snap` | `readonly SnapPoint[]` | `[]` | — | — | — | Offsets a release settles on: the nearest to where its momentum would carry it. |
| `momentum` | `boolean` | `false` | — | — | — | A release projects the offset by its velocity (0.3 s of it) before choosing a snap point: a flick. |
| `returnHome` | `boolean` | `false` | — | — | — | Without snap points, a release goes back to offset 0. |
| `dismiss` | `DismissSpec \| null` | `null` | — | — | — | Swipe to dismiss; null never throws it off. |
| `threshold` | `number` | `0` | — | `cssPx` | — | Pointer travel before a press becomes a drag (0: 4). |
| `spring` | `Transition \| null` | `null` | — | — | — | The release spring; null is the style's ui spring. |
| `tag` | `string` | `""` | — | — | — | What a drop target matches in DropSpec.accept. |

```ogt
struct DragSpec {
  doc "The element moves with a drag that starts on it (a card, a sheet, a carousel), released onto a spring that keeps the drag's velocity."
  desc true
  field axis DragAxis "Which way it moves."
  field bounds Option.UiRect "Bounds of its offset (x to x + width, y to y + height); null is free."
  field rubber f32 "Past the bounds it follows with resistance, iOS's rubber band with this constant (0 stops at the bounds; 0.55 is iOS's)."
  field snap List.SnapPoint "Offsets a release settles on: the nearest to where its momentum would carry it."
  field momentum bool "A release projects the offset by its velocity (0.3 s of it) before choosing a snap point: a flick."
  field returnHome bool "Without snap points, a release goes back to offset 0."
  field dismiss Option.DismissSpec "Swipe to dismiss; null never throws it off."
  field threshold f32 "Pointer travel before a press becomes a drag (0: 4)." {
    unit cssPx
  }
  field spring Option.Transition "The release spring; null is the style's ui spring."
  field tag String "What a drop target matches in DropSpec.accept."
}
```

<sub>Module `ui` line 1559</sub>
