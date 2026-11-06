<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Position"></a>

### `Position` (enum)

How an element is placed.

A string: one of the variants below. The zero value is `"flow"`.

| Variant | Doc |
|---|---|
| `"flow"` | In its parent's flow (the default). |
| `"absolute"` | Out of the flow, placed by its insets inside its parent's padding box. |
| `"screen"` | Out of the flow, placed by its insets in the whole surface rather than its parent's box: under the notch, the status bar and the home bar. A screen document's full-bleed art or backdrop, whose layer keeps its content in the safe area. |
| `"anchored"` | Out of the flow, its anchorSelf point on the anchorAt point of the element at anchorTo, moved by its start and top insets. |

```ogt
enum Position {
  doc "How an element is placed."
  variant flow "In its parent's flow (the default)."
  variant absolute "Out of the flow, placed by its insets inside its parent's padding box."
  variant screen "Out of the flow, placed by its insets in the whole surface rather than its parent's box: under the notch, the status bar and the home bar. A screen document's full-bleed art or backdrop, whose layer keeps its content in the safe area."
  variant anchored "Out of the flow, its anchorSelf point on the anchorAt point of the element at anchorTo, moved by its start and top insets."
}
```

<sub>`api.ogt` line 20212</sub>
