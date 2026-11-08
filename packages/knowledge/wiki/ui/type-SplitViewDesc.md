<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SplitViewDesc"></a>

### `SplitViewDesc` (desc struct)

Two panes with a divider between them that a drag moves.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Direction` | `"row"` | — | — | — | Side by side (row) or one over the other (column). |
| `ratio` | `number` | `0.5` | 0 … 1 | `ratio` | — | The start pane's share of the room (keep SplitResult.ratio). |
| `minStart` | `number` | `0` | ≥ 0 | `cssPx` | — | The start pane's least size; 0 is 120. |
| `minEnd` | `number` | `0` | ≥ 0 | `cssPx` | — | The end pane's least size; 0 is 120. |
| `snaps` | `readonly number[]` | `[]` | — | — | — | The ratios a press of the focused divider steps through; empty is a third, a half, two thirds. |
| `keepOnPhone` | `boolean` | `false` | — | — | — | Stay side by side on a phone in portrait (it stacks by default). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null grows. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the divider. |

```ogt
struct SplitViewDesc {
  doc "Two panes with a divider between them that a drag moves."
  desc true
  field direction Direction "Side by side (row) or one over the other (column)."
  field ratio f32 "The start pane's share of the room (keep SplitResult.ratio)." {
    default 0.5
    min 0
    max 1
    unit ratio
  }
  field minStart f32 "The start pane's least size; 0 is 120." {
    unit cssPx
    min 0
  }
  field minEnd f32 "The end pane's least size; 0 is 120." {
    unit cssPx
    min 0
  }
  field snaps List.f32 "The ratios a press of the focused divider steps through; empty is a third, a half, two thirds."
  field keepOnPhone bool "Stay side by side on a phone in portrait (it stacks by default)."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height; null grows."
  field label String "What a screen reader calls the divider."
}
```

<sub>Module `ui` line 2039</sub>
