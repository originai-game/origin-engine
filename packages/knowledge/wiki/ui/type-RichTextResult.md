<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RichTextResult"></a>

### `RichTextResult` (struct)

What a rich text's links did this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 }` | — | — | — | Its links together: hovered while one is, clicked when one was. |
| `link` | `string \| null` | `null` | — | — | — | The link clicked this tick: its `[link=…]` id. |
| `hovered` | `string \| null` | `null` | — | — | — | The link under the pointer. |

```ogt
struct RichTextResult {
  doc "What a rich text's links did this tick."
  field response Response "Its links together: hovered while one is, clicked when one was."
  field link Option.String "The link clicked this tick: its `[link=…]` id."
  field hovered Option.String "The link under the pointer."
}
```

<sub>Module `ui` line 671</sub>
