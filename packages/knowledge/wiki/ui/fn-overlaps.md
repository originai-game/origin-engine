<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-overlaps"></a>

### `og.ui.overlaps`

The last frame's overlaps between elements (docs/ui.md §2), with both paths: an absolute, anchored or floating element over another's drawing, and siblings in the flow over each other. What the engine places over another on purpose (a badge on its item's corner) is left out; whether any other overlap is meant is the reader's call.

```ts
og.ui.overlaps(): readonly UiOverlap[]
```

**Returns** `readonly UiOverlap[]`.

Binding symbol `og_ui_overlaps` · publish builds: kept

```ogt
fn overlaps {
  doc "The last frame's overlaps between elements (docs/ui.md §2), with both paths: an absolute, anchored or floating element over another's drawing, and siblings in the flow over each other. What the engine places over another on purpose (a badge on its item's corner) is left out; whether any other overlap is meant is the reader's call."
  returns List.UiOverlap
}
```

<sub>Module `ui` line 1252</sub>
