<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiHighlightRange"></a>

### `UiHighlightRange` (struct)

A range of laid-out text for og.ui.setHighlight, from a character of one text element to a character of the same element or a later one in document order.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `start` | `string` | `""` | — | — | — | The full path of the element it starts in (og.ui.focus, the probe's dump). |
| `startOffset` | `number` | `0` | — | — | — | The character it starts at, counting the element's characters over its lines joined by one break each. |
| `end` | `string` | `""` | — | — | — | The full path of the element it ends in. |
| `endOffset` | `number` | `0` | — | — | — | The character after its last, counted as startOffset is. |

```ogt
struct UiHighlightRange {
  doc "A range of laid-out text for og.ui.setHighlight, from a character of one text element to a character of the same element or a later one in document order."
  field start String "The full path of the element it starts in (og.ui.focus, the probe's dump)."
  field startOffset u32 "The character it starts at, counting the element's characters over its lines joined by one break each."
  field end String "The full path of the element it ends in."
  field endOffset u32 "The character after its last, counted as startOffset is."
}
```

<sub>Module `ui` line 520</sub>
