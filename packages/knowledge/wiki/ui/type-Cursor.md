<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Cursor"></a>

### `Cursor` (enum)

The pointer's shape over an element (CSS cursor, docs/ui.md §2), inherited by its subtree.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Its parent's; with none set above it, a text control's I-beam, a draggable element's open hand (closed while dragged), a link's hand, else the arrow. |
| `"default"` | The arrow. |
| `"pointer"` | The hand (a link). |
| `"text"` | The I-beam. |
| `"grab"` | An open hand. |
| `"grabbing"` | A closed hand. |
| `"move"` | The move arrows. |
| `"notAllowed"` | The slashed circle. |
| `"crosshair"` | The crosshair. |
| `"help"` | The question mark. |
| `"wait"` | Busy. |
| `"progress"` | Busy, still taking input. |
| `"resizeEw"` | Left and right. |
| `"resizeNs"` | Up and down. |
| `"resizeNesw"` | Up right and down left. |
| `"resizeNwse"` | Up left and down right. |
| `"zoomIn"` | The magnifier with a plus. |
| `"zoomOut"` | The magnifier with a minus. |
| `"none"` | Hidden (a game drawing its own). |

```ogt
enum Cursor {
  doc "The pointer's shape over an element (CSS cursor, docs/ui.md §2), inherited by its subtree."
  variant auto "Its parent's; with none set above it, a text control's I-beam, a draggable element's open hand (closed while dragged), a link's hand, else the arrow."
  variant default "The arrow."
  variant pointer "The hand (a link)."
  variant text "The I-beam."
  variant grab "An open hand."
  variant grabbing "A closed hand."
  variant move "The move arrows."
  variant notAllowed "The slashed circle."
  variant crosshair "The crosshair."
  variant help "The question mark."
  variant wait "Busy."
  variant progress "Busy, still taking input."
  variant resizeEw "Left and right."
  variant resizeNs "Up and down."
  variant resizeNesw "Up right and down left."
  variant resizeNwse "Up left and down right."
  variant zoomIn "The magnifier with a plus."
  variant zoomOut "The magnifier with a minus."
  variant none "Hidden (a game drawing its own)."
}
```

<sub>Module `ui` line 4766</sub>
