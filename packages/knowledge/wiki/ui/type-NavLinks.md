<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NavLinks"></a>

### `NavLinks` (desc struct)

Explicit focus moves from an element (docs/ui.md §2.1), over the spatial pick: each a path (a full one, `Hud/Bag`, or a sibling's id); empty leaves that move to the layout. A path to a focus group lands on its remembered member (or its first); one to any other container, on its first focusable control.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `up` | `string` | `""` | — | — | — | Where Up goes. |
| `down` | `string` | `""` | — | — | — | Where Down goes. |
| `left` | `string` | `""` | — | — | — | Where Left goes (the screen's left, also under a right-to-left layout). |
| `right` | `string` | `""` | — | — | — | Where Right goes. |
| `next` | `string` | `""` | — | — | — | Where Tab goes. |
| `prev` | `string` | `""` | — | — | — | Where Shift+Tab goes. |

```ogt
struct NavLinks {
  doc "Explicit focus moves from an element (docs/ui.md §2.1), over the spatial pick: each a path (a full one, `Hud/Bag`, or a sibling's id); empty leaves that move to the layout. A path to a focus group lands on its remembered member (or its first); one to any other container, on its first focusable control."
  desc true
  field up String "Where Up goes."
  field down String "Where Down goes."
  field left String "Where Left goes (the screen's left, also under a right-to-left layout)."
  field right String "Where Right goes."
  field next String "Where Tab goes."
  field prev String "Where Shift+Tab goes."
}
```

<sub>Module `ui` line 2397</sub>
