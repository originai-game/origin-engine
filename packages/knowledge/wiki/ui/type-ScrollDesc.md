<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScrollDesc"></a>

### `ScrollDesc` (desc struct)

A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `axes` | `ScrollAxes` | `"vertical"` | — | — | — | Which ways it scrolls. |
| `hideBars` | `boolean` | `false` | — | — | — | Draws no scrollbars: a row of chips or tabs that shows its overflow by its cut edge (a resting thumb under it reads as a selection underline). It still scrolls by the wheel, a drag and focus. |
| `fade` | `number` | `0` | ≥ 0 | `cssPx` | — | How far the content fades into `fadeInto` over an edge where more of it lies beyond, as strong as the content past that edge reaches this length (a log's top row cut mid-glyph reads as "there is more"); 0 draws no fade. |
| `fadeInto` | `ThemeColor \| null` | `null` | — | — | — | The surface the content fades into; null is panel. |

```ogt
struct ScrollDesc {
  doc "A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it."
  desc true
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Insets "Space inside."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field axes ScrollAxes "Which ways it scrolls."
  field hideBars bool "Draws no scrollbars: a row of chips or tabs that shows its overflow by its cut edge (a resting thumb under it reads as a selection underline). It still scrolls by the wheel, a drag and focus."
  field fade f32 "How far the content fades into `fadeInto` over an edge where more of it lies beyond, as strong as the content past that edge reaches this length (a log's top row cut mid-glyph reads as \"there is more\"); 0 draws no fade." {
    unit cssPx
    min 0
  }
  field fadeInto Option.ThemeColor "The surface the content fades into; null is panel."
}
```

<sub>`api.ogt` line 15943</sub>
