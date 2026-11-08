<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextPathSpec"></a>

### `TextPathSpec` (desc struct)

An element's text set along a path (CSS -og-text-path, SVG 2 textPath, docs/ui.md §2): each character turned to the path where it sits, its baseline on the path. One line; the element's textAlign anchors it at offset. CSS offset-path moves the element itself; this moves its text.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `string` | `""` | — | — | — | SVG path data (M L H V C S Q T A Z), CSS px from the element's content box; empty is a circle. Data that does not parse sets the text in lines. |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | With empty data: the circle's radius, centred in the content box from its top, clockwise; 0 is the box's closest side. |
| `offset` | `number` | `0` | — | — | — | Where the anchor sits along the path: CSS px, or with percent a share of its length (0 to 1). |
| `percent` | `boolean` | `false` | — | — | — | offset is a share of the path's length. |
| `reverse` | `boolean` | `false` | — | — | — | Runs the path backwards (SVG side=right): on a circle, counter-clockwise inside it. |

```ogt
struct TextPathSpec {
  doc "An element's text set along a path (CSS -og-text-path, SVG 2 textPath, docs/ui.md §2): each character turned to the path where it sits, its baseline on the path. One line; the element's textAlign anchors it at offset. CSS offset-path moves the element itself; this moves its text."
  desc true
  field data String "SVG path data (M L H V C S Q T A Z), CSS px from the element's content box; empty is a circle. Data that does not parse sets the text in lines."
  field radius f32 "With empty data: the circle's radius, centred in the content box from its top, clockwise; 0 is the box's closest side." {
    unit cssPx
    min 0
  }
  field offset f32 "Where the anchor sits along the path: CSS px, or with percent a share of its length (0 to 1)."
  field percent bool "offset is a share of the path's length."
  field reverse bool "Runs the path backwards (SVG side=right): on a circle, counter-clockwise inside it."
}
```

<sub>Module `ui` line 1955</sub>
