<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiHighlight"></a>

### `UiHighlight` (desc struct)

A custom highlight (CSS Custom Highlight API) for og.ui.setHighlight: ranges of text that `::highlight(<name>)` rules style. The zero value covers nothing, at priority 0.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ranges` | `readonly UiHighlightRange[]` | `[]` | — | — | — | Its ranges. Overlapping ranges paint once, as their union, and a collapsed range paints nothing. |
| `priority` | `number` | `0` | — | — | — | Where its overlay stacks: higher priorities paint above, and of equal ones the highlight registered later is above. Every custom highlight paints under ::target-text and the selection. |

```ogt
struct UiHighlight {
  doc "A custom highlight (CSS Custom Highlight API) for og.ui.setHighlight: ranges of text that `::highlight(<name>)` rules style. The zero value covers nothing, at priority 0."
  desc true
  field ranges List.UiHighlightRange "Its ranges. Overlapping ranges paint once, as their union, and a collapsed range paints nothing."
  field priority i32 "Where its overlay stacks: higher priorities paint above, and of equal ones the highlight registered later is above. Every custom highlight paints under ::target-text and the selection."
}
```

<sub>Module `ui` line 528</sub>
